use std::time::Instant;

use subvox_backend::{FourierTransformer, load_audio_file};

// The Plan (tm)
// 1. Load file
// 2. Compute Fourier Transform
// 3. Compute Logarithm of Magnitudes
// 4. Compute Cepstrum (Inverse Fourier Transform)

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let file = load_audio_file("assets/growl.wav").expect("Failed to load file");
    let format = file.format();
    let frames = file.to_f32_vec();

    let mut transformer = FourierTransformer::new();

    let start = Instant::now();

    let transformed = transformer.stft(&frames, 2048, 1024);

    let end = Instant::now();

    println!(
        "2048 window, 1024 hop: {}us",
        end.duration_since(start).as_micros()
    );

    let start = Instant::now();

    let transformed = transformer.stft(&frames, 16384, 4096);

    let end = Instant::now();

    println!(
        "16384 window, 4096 hop: {}us",
        end.duration_since(start).as_micros()
    );

    dbg!(format.sample_rate);
    dbg!(frames.len());
    dbg!(transformed.len());
    println!("{:#?}", &transformed[0][0..10]);

    Ok(())
}
