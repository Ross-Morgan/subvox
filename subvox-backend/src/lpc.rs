//! Linear Predictive Coding (LPC)

use std::f32::consts::PI;

use rayon::prelude::*;

pub struct LpcResult {
    pub data: Vec<f32>,
    pub order: usize,
    pub error_energy: Vec<f32>,
}

pub fn par_lpc(samples: &[f32], window_size: usize, hop_size: usize, order: usize) -> LpcResult {
    assert!(hop_size > 0, "hop_size must be non-zero");
    assert!(
        window_size <= samples.len(),
        "window size exceeds signal length"
    );
    assert!(
        order > 0 && order < window_size,
        "order must be positive and less than window_size"
    );

    // TODO: Remove hardcoded window function
    let window = (0..window_size)
        .map(|n| (PI * n as f32 / (window_size as f32 - 1.0)).sin().powi(2))
        .collect::<Vec<_>>();

    let frame_count = (samples.len() - window_size) / hop_size + 1;

    let mut coeffs = vec![0.0f32; frame_count * order];
    let mut error_energy = vec![0.0f32; frame_count];

    coeffs
        .par_chunks_mut(order)
        .zip(error_energy.par_iter_mut())
        .enumerate()
        .for_each_init(
            || (vec![0.0f32; window_size], vec![0.0f32; order + 1]),
            |(windowed, autocorr), (frame_idx, (coeffs_out, error_out))| {
                let start = frame_idx * hop_size;

                for i in 0..window_size {
                    windowed[i] = samples[start + i] * window[i];
                }

                autocorrelate(windowed, order, autocorr);
                *error_out = levinson_durbin(autocorr, order, coeffs_out);
            },
        );

    LpcResult {
        data: coeffs,
        order,
        error_energy,
    }
}

fn autocorrelate(frame: &[f32], order: usize, out: &mut [f32]) {
    for lag in 0..=order {
        let mut sum = 0.0f32;
        for i in 0..(frame.len() - lag) {
            sum += frame[i] * frame[i + lag];
        }
        out[lag] = sum;
    }
}

// TODO: Replace with an algorithm that is faster?
// I don't know enough about this to make a nuanced decision
fn levinson_durbin(autocorr: &[f32], order: usize, coeffs: &mut [f32]) -> f32 {
    let mut error = autocorr[0];
    let mut a = vec![0.0f32; order + 1];

    if error.abs() < f32::EPSILON {
        coeffs.fill(0.0);
        return 0.0;
    }

    for i in 1..=order {
        let mut acc = autocorr[i];
        for j in 1..i {
            acc -= a[j] * autocorr[i - j];
        }
        let k = acc / error;

        let mut new_a = a.clone();
        new_a[i] = k;
        for j in 1..i {
            new_a[j] = a[j] - k * a[i - j];
        }
        a = new_a;

        error *= 1.0 - k * k;
        if error <= 0.0 {
            // Guards against numerical instability (e.g. near-silent or
            // perfectly periodic frames) driving error negative.
            error = f32::EPSILON;
        }
    }

    coeffs.copy_from_slice(&a[1..=order]);
    error
}
