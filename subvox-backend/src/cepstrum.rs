use std::sync::Arc;

use rayon::prelude::*;
use realfft::{ComplexToReal, num_complex::Complex32};

use crate::{fft::StftResult, stats::linear_regression_full};

// Example used
const CEPSTRUM_LOG_CONSTANT: f32 = 1e-10;

pub struct CepstrumResult {
    pub data: Vec<f32>,
    pub quefrencies: usize,
}

/// Computes real cepstrum
pub fn par_cepstrum(stft: &StftResult, c2r: Arc<dyn ComplexToReal<f32>>) -> CepstrumResult {
    let bins = stft.bins;
    let window_size = (bins - 1) * 2;
    let frame_count = stft.data.len() / bins;
    let mut cepstra = vec![0.0f32; frame_count * window_size];

    stft.data
        .par_chunks(bins)
        .zip_eq(cepstra.par_chunks_mut(window_size))
        .for_each_init(
            || {
                (
                    vec![Complex32::ZERO; bins],
                    c2r.make_scratch_vec(),
                    Arc::clone(&c2r),
                )
            },
            |(log_spectrum, scratch_buf, c2r), (spectrum_in, cepstrum_out)| {
                for (dst, src) in log_spectrum.iter_mut().zip(spectrum_in.iter()) {
                    dst.re = src.norm().max(CEPSTRUM_LOG_CONSTANT).ln();
                    dst.im = 0.0;
                }

                c2r.process_with_scratch(
                    log_spectrum.as_mut_slice(),
                    cepstrum_out,
                    scratch_buf.as_mut_slice(),
                )
                .expect("IFFT failed");

                let norm = 1.0 / window_size as f32;
                cepstrum_out.iter_mut().for_each(|v| *v *= norm);
            },
        );

    CepstrumResult {
        data: cepstra,
        quefrencies: window_size,
    }
}

/// Cepstral Peak Prominence
///
/// Higher values can mean more periodicity and clear phonation
/// Lower values can mean more aperiodicity and a breathier, rougher phonation
pub fn par_cpp(
    cepstrum: &CepstrumResult,
    sample_rate: f32,
    min_pitch_hz: f32,
    max_pitch_hz: f32,
) -> Vec<f32> {
    let q = cepstrum.quefrencies;

    // Quefrency range corresponding to the given pitch range.
    // quefrency (in samples) = sample_rate / frequency
    let min_quefrency = (sample_rate / max_pitch_hz).round() as usize;
    let max_quefrency = ((sample_rate / min_pitch_hz).round() as usize).min(q - 1);

    cepstrum
        .data
        .par_chunks(q)
        .map(|frame| {
            // Convert to log scale (dB-like) for the regression + peak comparison.
            // Cepstrum values can be negative, so we work with magnitude.
            let log_cepstrum: Vec<f32> = frame
                .iter()
                .map(|&v| 20.0 * v.abs().max(1e-10).log10())
                .collect();

            // Regression line over the full cepstrum (the "noise floor" trend).
            let x: Vec<f32> = (0..q).map(|i| i as f32).collect();
            let (slope, intercept) = linear_regression_full(&x, &log_cepstrum);

            // Find the peak within the pitch-plausible quefrency range.
            let (peak_idx, peak_val) = log_cepstrum[min_quefrency..=max_quefrency]
                .iter()
                .enumerate()
                .fold(
                    (0, f32::MIN),
                    |(bi, bv), (i, &v)| {
                        if v > bv { (i, v) } else { (bi, bv) }
                    },
                );
            let peak_idx = peak_idx + min_quefrency;

            let expected = slope * peak_idx as f32 + intercept;
            peak_val - expected
        })
        .collect()
}
