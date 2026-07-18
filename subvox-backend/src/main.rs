use realfft::RealFftPlanner;
use subvox_backend::{Note, load_audio_file};

// The Plan (tm)
// 1. Load file
// 2. Compute Fourier Transform
// 3. Compute Logarithm of Magnitudes
// 4. Compute Cepstrum (Inverse Fourier Transform)

const WINDOW_SIZE: usize = 8192;
const HOP_SIZE: usize = 1024;
const LPC_ORDER: usize = 12;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let file = load_audio_file("assets/growl.wav").expect("Failed to load file");
    let frames = file.to_f32_vec();

    let mut planner = RealFftPlanner::<f32>::new();

    let r2c = planner.plan_fft_forward(WINDOW_SIZE);
    let c2r = planner.plan_fft_inverse(WINDOW_SIZE);

    // 8k window, 1k hop
    // TODO: Experiment with LPC orders. 12 is a reasonable default but could be useful? (TBD)
    // NOTE: Levinson-Durbin is O(order^2), which I will inevitably forget until I try to run a stupidly large order size and wonder why it's taking forever.

    let spectra = subvox_backend::par_stft(&frames, WINDOW_SIZE, HOP_SIZE, r2c);
    let cepstra = subvox_backend::par_cepstrum(&spectra, c2r);
    let lpc = subvox_backend::par_lpc(&frames, WINDOW_SIZE, HOP_SIZE, LPC_ORDER);

    let n = Note::new(440.0);

    println!("{n:?}");

    Ok(())
}
