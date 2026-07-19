use realfft::RealFftPlanner;
use subvox_backend::load_audio_file;

// The Plan (tm)
// 1. Load file
// 2. Compute Fourier Transform
// 3. Compute Logarithm of Magnitudes
// 4. Compute Cepstrum (Inverse Fourier Transform)

const WINDOW_SIZE: usize = 16384;
const HOP_SIZE: usize = 1024;
const LPC_ORDER: usize = 12;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let file = load_audio_file("assets/sixteen-tons.wav").expect("Failed to load file");
    let frames = file.to_f32_vec();

    let sample_rate = file.format().sample_rate;

    let mut planner = RealFftPlanner::<f32>::new();

    let r2c = planner.plan_fft_forward(WINDOW_SIZE);
    let c2r = planner.plan_fft_inverse(WINDOW_SIZE);

    // 8k window, 1k hop
    // TODO: Experiment with LPC orders. 12 is a reasonable default but could be useful? (TBD)
    // NOTE: Levinson-Durbin is O(order^2), which I will inevitably forget until I try to run a stupidly large order size and wonder why it's taking forever.

    let spectra = subvox_backend::par_stft(&frames, WINDOW_SIZE, HOP_SIZE, r2c);
    let cepstra = subvox_backend::par_cepstrum(&spectra, c2r);
    let _lpc = subvox_backend::par_lpc(&frames, WINDOW_SIZE, HOP_SIZE, LPC_ORDER);

    let algorithm_distribution = subvox_backend::pitch::PitchAlgorithmCandidateDistribution {
        cpp: 3,
        hps: 3,
        yin: 3,
    };

    let mut notes = subvox_backend::pitch::combined_pitch_estimate(
        &frames,
        &spectra,
        &cepstra,
        sample_rate,
        WINDOW_SIZE,
        HOP_SIZE,
        50.0,
        550.0,
        algorithm_distribution,
    );

    notes.dedup();

    dbg!(&notes);
    dbg!(notes.len());

    Ok(())
}
