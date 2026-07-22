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

const WINDOW_SIZE: usize = 16384;
const HOP_SIZE: usize = 2048;
const LPC_ORDER: usize = 12;

fn main() {
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
}
