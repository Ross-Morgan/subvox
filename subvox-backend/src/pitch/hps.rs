//! Harmonic Product Spectrum

use std::fmt::Debug;

use rayon::prelude::*;

use crate::{
    Note,
    fft::StftResult,
    stats::{find_local_maxima, parabolic_interpolate},
};

pub struct HpsPitchCandidate {
    pub frequency: f32,
    pub prominence: f32,
}

impl Debug for HpsPitchCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Note::new(self.frequency).fmt(f)
    }
}

pub struct HpsPitchResult {
    pub candidates: Vec<Vec<HpsPitchCandidate>>,
}

pub fn hps_pitch_candidates(
    stft: &StftResult,
    sample_rate: f32,
    min_pitch_hz: f32,
    max_pitch_hz: f32,
    num_harmonics: usize,
    max_candidates: usize,
) -> HpsPitchResult {
    let bins = stft.bins;
    let window_size = (bins - 1) * 2;
    let bin_hz = sample_rate / window_size as f32;

    let min_bin = (min_pitch_hz / bin_hz).floor().max(1.0) as usize;
    let max_bin = (max_pitch_hz / bin_hz).ceil().min((bins - 1) as f32) as usize;

    assert!(
        min_bin < max_bin,
        "invalid pitch range for given sample_rate/window_size"
    );
    assert!(
        num_harmonics >= 2,
        "num_harmonics must be at least 2 for HPS to do anything"
    );

    let candidates = stft
        .data
        .par_chunks(bins)
        .map(|spectrum| {
            let magnitudes: Vec<f32> = spectrum.iter().map(|c| c.norm()).collect();

            // Product spectrum only needs to be computed up to max_bin,
            // since a fundamental can't be found above the search range
            // regardless of how the harmonics stack up.
            let mut product = vec![0.0f32; max_bin + 1];
            product[..=max_bin].copy_from_slice(&magnitudes[..=max_bin]);

            for harmonic in 2..=num_harmonics {
                for bin in min_bin..=max_bin {
                    let downsampled_idx = bin * harmonic;
                    if downsampled_idx < magnitudes.len() {
                        product[bin] *= magnitudes[downsampled_idx];
                    } else {
                        product[bin] = 0.0;
                    }
                }
            }

            let mut peaks = find_local_maxima(&product, min_bin, max_bin);
            peaks.sort_by(|&a, &b| product[b].partial_cmp(&product[a]).unwrap());
            peaks.truncate(max_candidates);

            peaks
                .into_iter()
                .map(|bin| {
                    let refined_bin = parabolic_interpolate(&product, bin);
                    HpsPitchCandidate {
                        frequency: refined_bin * bin_hz,
                        prominence: product[bin],
                    }
                })
                .collect()
        })
        .collect();

    HpsPitchResult { candidates }
}
