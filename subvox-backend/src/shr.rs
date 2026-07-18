//! Subharmonic-to-Harmonic Ratio (SHR)

use rayon::prelude::*;
use realfft::num_complex::Complex32;

use crate::fft::StftResult;

pub fn par_shr(
    stft: &StftResult,
    sample_rate: f32,
    f0_estimates: &[f32],
    num_harmonics: usize,
) -> Vec<f32> {
    let bins = stft.bins;
    let window_size = (bins - 1) * 2;
    let frame_count = stft.data.len() / bins;

    assert_eq!(
        f0_estimates.len(),
        frame_count,
        "f0_estimates length must match frame count"
    );

    stft.data
        .par_chunks(bins)
        .zip(f0_estimates.par_iter())
        .map(|(spectrum, &f0)| {
            if f0 <= 0.0 || f0.is_nan() {
                return f32::NAN;
            }

            let mut harmonic_energy = 0.0f32;
            let mut subharmonic_energy = 0.0f32;

            for k in 1..=num_harmonics {
                let harmonic_freq = k as f32 * f0;
                let subharmonic_freq = (k as f32 - 0.5) * f0;

                harmonic_energy +=
                    interpolated_magnitude(spectrum, harmonic_freq, sample_rate, window_size);
                subharmonic_energy +=
                    interpolated_magnitude(spectrum, subharmonic_freq, sample_rate, window_size);
            }

            20.0 * (subharmonic_energy.max(1e-10) / harmonic_energy.max(1e-10)).log10()
        })
        .collect()
}

fn interpolated_magnitude(
    spectrum: &[Complex32],
    freq: f32,
    sample_rate: f32,
    window_size: usize,
) -> f32 {
    let bin_pos = freq * window_size as f32 / sample_rate;

    if bin_pos < 0.0 || bin_pos >= (spectrum.len() - 1) as f32 {
        return 0.0;
    }

    let lower = bin_pos.floor() as usize;
    let upper = lower + 1;
    let frac = bin_pos - lower as f32;

    let mag_lower = spectrum[lower].norm();
    let mag_upper = spectrum[upper].norm();

    mag_lower * (1.0 - frac) + mag_upper * frac
}
