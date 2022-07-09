use creak::{Decoder, DecoderError};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const TEMP_WAV_PATH: &str = "__shiv_temp.wav";
const DEFAULT_OUTPUT_PATH: &str = "__out";
const METADATA_FILE: &str = "FFMETADATAFILE";

fn write_chapter(
    metadata_file: &mut fs::File,
    title: &str,
    starttime_in_samples: u64,
    endtime_in_samples: u64,
) -> () {
    writeln!(metadata_file, "[CHAPTER]");
    writeln!(metadata_file, "TIMEBASE=1/44100"); // FIXME: take this as an arg
    writeln!(metadata_file, "START={}", starttime_in_samples);
    writeln!(metadata_file, "END={}", endtime_in_samples);
    writeln!(metadata_file, "title={}", title);
}

fn main() -> io::Result<()> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let path = env::current_dir()?;
    assert!(path.is_dir());
    println!("Hello, world! {}", path.display());
    if let Some(path) = env::args().nth(1) {
        env::set_current_dir(path)?;
    }
    let mut writer = hound::WavWriter::create(TEMP_WAV_PATH, spec).unwrap();
    let mut metadata = fs::File::create(METADATA_FILE)?;
    let mut last_chapter_end: u64 = 0;
    writeln!(metadata, ";FFMETADATA1");
    for path in path.read_dir()? {
        let res: io::Result<()> = (|| {
            let path = path?.path();
            if path.ends_with(TEMP_WAV_PATH) {
                return Ok(());
            }
            println!("- {:?}", path);
            if path.is_file() {
                let file = match Decoder::open(&path) {
                    Ok(f) => f,
                    Err(DecoderError::NoExtension | DecoderError::UnsupportedExtension(_)) => {
                        return Ok(())
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return Ok(());
                    }
                };
                assert_eq!(file.info().sample_rate(), spec.sample_rate);
                assert_eq!(file.info().channels(), spec.channels as usize);

                let samples = file.into_samples().unwrap();
                let mut amt_interleaved_samples: u64 = 0;
                for sample in samples {
                    let sample = sample.unwrap();
                    writer.write_sample(sample).unwrap();
                    amt_interleaved_samples += 1;
                }
                let duration_in_samples = amt_interleaved_samples / spec.channels as u64;
                let duration_in_seconds =
                    duration_in_samples / spec.sample_rate as u64;
                println!(
                    "duration {}:{}",
                    duration_in_seconds / 60,
                    duration_in_seconds % 60
                );
                write_chapter(
                    &mut metadata,
                    &path.file_stem().unwrap().to_str().unwrap(),
                    last_chapter_end,
                    last_chapter_end + duration_in_samples,
                );
                last_chapter_end += duration_in_samples;
            }
            Ok(())
        })();
        if let Err(e) = res {
            eprintln!("Error: {:?}", e);
        }
    }
    writer.finalize().unwrap();
    //let default_out_path = &OsString::from(DEFAULT_OUTPUT_PATH);
    let out_path = path;
    let out_path = out_path
        .file_name()
        .unwrap_or_else(|| OsStr::new(DEFAULT_OUTPUT_PATH));
    let mut out_path = out_path.to_owned();
    out_path.push(".mka");
    assert!(!Path::new(dbg!(&out_path)).exists());
    Command::new("ffmpeg")
        .arg("-i")
        .arg(TEMP_WAV_PATH)
        .arg("-i")
        .arg(METADATA_FILE)
        .arg("-map_metadata")
        .arg("1")
        .arg("-map_chapters")
        .arg("1")
        .arg(out_path)
        .spawn()
        .expect("Failed to run ffmpeg")
        .wait()
        .unwrap();
    fs::remove_file(TEMP_WAV_PATH).unwrap();
    Ok(())
}
