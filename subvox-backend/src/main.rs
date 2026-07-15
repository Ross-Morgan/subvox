use subvox_backend::{FourierTransformer, load_audio_file};

// The Plan (tm)
// 1. Load file
// 2. Compute Fourier Transform
// 3. Compute Logarithm of Magnitudes
// 4. Compute Cepstrum (Inverse Fourier Transform)

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let file = load_audio_file("assets/vocal-1.wav").expect("Failed to load file");
    let frames = file.to_f32_vec();

    println!("{}", frames.len());

    let mut transformer = FourierTransformer::new();

    transformer.stft(&frames, 16384, 4096);

    Ok(())
}
