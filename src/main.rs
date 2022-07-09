use creak::{Decoder, DecoderError};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

const TEMP_WAV_PATH: &str = "__shiv_temp.wav";
const DEFAULT_OUTPUT_PATH: &str = "__out";

fn main() {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let path = env::current_dir().expect("Could not find current directory");
    assert!(path.is_dir());
    println!("Hello, world! {}", path.display());
    if let Some(path) = env::args().nth(1) {
        env::set_current_dir(path).unwrap();
    }
    let mut writer = hound::WavWriter::create(TEMP_WAV_PATH, spec).unwrap();
    for path in path.read_dir().expect("Could not read files") {
        let res: io::Result<()> = (|| {
            let path = path?.path();
            if path.ends_with(TEMP_WAV_PATH) {
                return Ok(());
            }
            println!("- {:?}", path);
            if path.is_file() {
                let file = match Decoder::open(path) {
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
                for sample in file.into_samples().unwrap() {
                    let sample = sample.unwrap();
                    writer.write_sample(sample).unwrap();
                }
            }
            Ok(())
        })();
        if let Err(e) = res {
            eprintln!("Error: {:?}", e);
        }
    }
    writer.finalize().unwrap();
    //let default_out_path = &OsString::from(DEFAULT_OUTPUT_PATH);
    let out_path = path.clone();
    let out_path = out_path
        .file_name()
        .unwrap_or_else(|| OsStr::new(DEFAULT_OUTPUT_PATH));
    let mut out_path = out_path.to_owned();
    out_path.push(".mp3");
    assert!(!Path::new(&out_path).exists());
    Command::new("ffmpeg")
        .arg("-i")
        .arg(TEMP_WAV_PATH)
        .arg(out_path)
        .spawn()
        .expect("Failed to run ffmpeg")
        .wait()
        .unwrap();
    fs::remove_file(TEMP_WAV_PATH).unwrap();
}
