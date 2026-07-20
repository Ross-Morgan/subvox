use rayon::prelude::*;

use crate::{fft::StftResult, stats::linear_regression};

const TILT_LOG_CONSTANT: f32 = 1e-10;

/// Slope of a linear fit of log(magnitude) vs log(frequency)
///
/// More negative values mean faster high frequency rolloff
/// More positive values mean a flatter spectrum
///
/// The former can mean a breathier or softer phonation
/// The latter can mean a harsher or pressed phonation
pub fn par_tilt(stft: &StftResult, sample_rate: f32) -> Vec<f32> {
    let bins = stft.bins;
    let window_size = (bins - 1) * 2;

    let log_frequencies = (1..bins)
        .map(|k| {
            let freq = k as f32 * sample_rate / window_size as f32;
            freq.ln()
        })
        .collect::<Vec<f32>>();

    stft.data
        .par_chunks(bins)
        .map(|frame| {
            let log_magnitudes = frame[1..]
                .iter()
                .map(|c| c.norm().max(TILT_LOG_CONSTANT).ln())
                .collect::<Vec<f32>>();

            linear_regression(&log_frequencies, &log_magnitudes).slope
        })
        .collect::<Vec<_>>()
}
