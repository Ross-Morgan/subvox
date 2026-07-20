use std::hint::black_box;

use realfft::RealFftPlanner;
use subvox_backend::load_audio_file;

// The Plan (tm)
// 1. Load file
// 2. Compute Fourier Transform
// 3. Compute Logarithm of Magnitudes
// 4. Compute Cepstrum (Inverse Fourier Transform)
// 5. Derive metrics from different transforms
// 6. Analyse and process
// 7. Profit???

const WINDOW_SIZE: usize = 2usize.pow(18);
const HOP_SIZE: usize = 1024;
// const LPC_ORDER: usize = 12;

fn main() -> color_eyre::Result<()> {
    // color_eyre::install()?;

    #[cfg(debug_assertions)]
    println!("Loading audio file...");

    let file = load_audio_file("assets/sixteen-tons.wav").expect("Failed to load file");
    let frames = file.to_f32_vec();

    // let sample_rate = file.format().sample_rate;

    println!("Frames: {}", frames.len());
    println!("Bytes: {}", frames.len() * std::mem::size_of::<f32>());

    let mut planner = RealFftPlanner::<f32>::new();

    let r2c = planner.plan_fft_forward(WINDOW_SIZE);
    // let c2r = planner.plan_fft_inverse(WINDOW_SIZE);

    // 8k window, 1k hop
    // TODO: Experiment with LPC orders. 12 is a reasonable default but could be useful? (TBD)
    // NOTE: Levinson-Durbin is O(order^2), which I will inevitably forget until I try to run a stupidly large order size and wonder why it's taking forever.

    #[cfg(debug_assertions)]
    println!("Computing STFT...");

    let start_stft = std::time::Instant::now();

    let _spectra = black_box(subvox_backend::par_stft(
        &frames,
        WINDOW_SIZE,
        HOP_SIZE,
        r2c,
    ));

    let end_stft = std::time::Instant::now();

    // #[cfg(debug_assertions)]
    // println!("Computing cepstrum...");

    // let cepstrum_start = std::time::Instant::now();

    // let cepstra = black_box(subvox_backend::par_cepstrum(&spectra, c2r));

    // let cepstrum_end = std::time::Instant::now();

    // #[cfg(debug_assertions)]
    // println!("Computing LPC...");

    // let lpc_start = std::time::Instant::now();

    // let _lpc = black_box(subvox_backend::par_lpc(
    //     &frames,
    //     WINDOW_SIZE,
    //     HOP_SIZE,
    //     LPC_ORDER,
    // ));

    // let lpc_end = std::time::Instant::now();

    // let algorithm_distribution = subvox_backend::pitch::PitchAlgorithmCandidateDistribution {
    //     cpp: 3,
    //     hps: 3,
    //     yin: 3,
    // };

    // #[cfg(debug_assertions)]
    // println!("Computing pitch estimate...");

    // let pitch_start = std::time::Instant::now();

    // let notes = black_box(subvox_backend::pitch::combined_pitch_estimate(
    //     &frames,
    //     &spectra,
    //     &cepstra,
    //     sample_rate,
    //     WINDOW_SIZE,
    //     HOP_SIZE,
    //     65.0,
    //     550.0,
    //     algorithm_distribution,
    // ));

    // let pitch_end = std::time::Instant::now();

    let stft_duration = end_stft.duration_since(start_stft);
    // let cepstrum_duration = cepstrum_end.duration_since(cepstrum_start);
    // let lpc_duration = lpc_end.duration_since(lpc_start);
    // let pitch_duration = pitch_end.duration_since(pitch_start);

    println!("STFT duration: {}us", stft_duration.as_micros());
    // println!("Cepstrum duration: {}us", cepstrum_duration.as_micros());
    // println!("LPC duration: {}us", lpc_duration.as_micros());
    // println!("Pitch duration: {}us", pitch_duration.as_micros());

    // black_box(notes);

    Ok(())
}
