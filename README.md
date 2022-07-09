# shiv
A command line tool for concatenating all audio files in a folder. Requires `ffmpeg` to be accessible in the terminal.

## Usage
```
shiv [output-dir]
```
Find all audio files in the current working directory, concatenate them seamlessly into a temporary `wav` file, and then use `ffmpeg` to convert the full audio into an `mp3` file which is saved to `output-dir` or the current working directory if not specified.

Accepts the following audio file formats:
```
wav
mp3
flac
ogg
```
