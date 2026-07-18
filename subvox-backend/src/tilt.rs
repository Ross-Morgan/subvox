use rayon::prelude::*;

use crate::fft::StftResult;

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

            linear_regression_slope(&log_frequencies, &log_magnitudes)
        })
        .collect::<Vec<_>>()
}

// Hell yeah, statistics
fn linear_regression_slope(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len() as f32;
    let mean_x = x.iter().sum::<f32>() / n;
    let mean_y = y.iter().sum::<f32>() / n;

    let mut cov = 0.0f32;
    let mut var_x = 0.0f32;
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        cov += dx * (yi - mean_y);
        var_x += dx * dx;
    }

    if var_x.abs() < f32::EPSILON {
        0.0
    } else {
        cov / var_x
    }
}
