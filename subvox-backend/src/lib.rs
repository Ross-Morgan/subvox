#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod audio;
mod fft;

use std::{path::Path, process::Command};

/// Load audio file, converting the source file to a .wav if it is not already
///
/// # Errors
///
/// Returns an error if ffmpeg can't be found or fails
pub fn load_audio_file<P: AsRef<Path>>(path: P) -> Result<audio::WavFile, std::io::Error> {
    let path = path.as_ref();
    let wav_path = path.with_extension("wav");

    let extension = path.extension().unwrap_or_default().to_ascii_lowercase();

    if extension != "wav" {
        Command::new("ffmpeg")
            .arg("-i")
            .arg(path)
            .arg(&wav_path)
            .output()?;
    }

    Ok(audio::load_wav(wav_path))
}

pub use fft::FourierTransformer;
