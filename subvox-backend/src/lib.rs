#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// TODO:
// 1. Unify sequential and parallel apis
// 2. Proper error handling
// 3. Probably many other things that I can't think of right now

mod audio;
mod cepstrum;
mod fft;
mod lpc;
mod notes;
mod shr;
mod tilt;
mod windowing;

// Pitch detection algorithms
pub mod pitch;

mod stats;

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

    // TODO: Use a more robust method to do this
    if extension != "wav" {
        Command::new("ffmpeg")
            .arg("-i")
            .arg(path)
            .arg(&wav_path)
            .output()?;
    }

    Ok(audio::load_wav(wav_path))
}

pub use fft::{FourierTransformer, par_stft};

// Analysis stuff
pub use cepstrum::{par_cepstrum, par_cpp};
pub use lpc::par_lpc;
pub use pitch::{cpp_pitch_candidates, hps_pitch_candidates, yin_pitch_candidates};
pub use shr::par_shr;
pub use tilt::par_tilt;

pub use notes::Note;
