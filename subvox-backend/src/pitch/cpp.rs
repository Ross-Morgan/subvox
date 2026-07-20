//! Cepstral Peak-Picking
//!
//! NOTE: Useful up to ~500Hz

use std::fmt::Debug;

use rayon::prelude::*;

// TODO: Tweak
const MIN_PEAK_SEPARATION: usize = 3;

use crate::{
    Note,
    cepstrum::CepstrumResult,
    stats::{LinearRegression, find_local_maxima, linear_regression, parabolic_interpolate},
};

pub struct CepstralPitchCandidate {
    pub frequency: f32,
    pub quefrency: usize,
    // Measure of confidence
    pub prominence: f32,
}

impl Debug for CepstralPitchCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note = Note::new(self.frequency);
        note.fmt(f)
    }
}

pub struct CepstralPitchResult {
    // TODO: Remove double indirection
    pub candidates: Vec<Vec<CepstralPitchCandidate>>,
}

pub fn cpp_pitch_candidates(
    cepstrum: &CepstrumResult,
    sample_rate: f32,
    min_pitch_hz: f32,
    max_pitch_hz: f32,
    max_candidates: usize,
) -> CepstralPitchResult {
    let q = cepstrum.quefrencies;

    let min_quefrency = (sample_rate / max_pitch_hz).round() as usize;
    let max_quefrency = ((sample_rate / min_pitch_hz).round() as usize).min(q - 1);

    assert!(
        min_quefrency < max_quefrency,
        "invalid pitch range for given sample_rate/quefrency count"
    );

    let candidates = cepstrum
        .data
        .par_chunks(q)
        .map(|frame| {
            let log_cepstrum: Vec<f32> = frame
                .iter()
                .map(|&v| 20.0 * v.abs().max(1e-10).log10())
                .collect();

            let x: Vec<f32> = (min_quefrency..=max_quefrency).map(|i| i as f32).collect();
            let LinearRegression { slope, intercept } = linear_regression(&x, &log_cepstrum);

            let mut peaks = find_local_maxima(&log_cepstrum, min_quefrency, max_quefrency);

            peaks = merge_close_peaks(peaks, &log_cepstrum, MIN_PEAK_SEPARATION);

            peaks.sort_by(|&a, &b| {
                let prom_a = log_cepstrum[a] - (slope * a as f32 + intercept);
                let prom_b = log_cepstrum[b] - (slope * b as f32 + intercept);
                prom_b.partial_cmp(&prom_a).unwrap()
            });
            peaks.truncate(max_candidates);

            peaks
                .into_iter()
                .map(|quefrency| {
                    let refined_quefrency = parabolic_interpolate(&log_cepstrum, quefrency);
                    let prominence =
                        log_cepstrum[quefrency] - (slope * quefrency as f32 + intercept);
                    CepstralPitchCandidate {
                        frequency: sample_rate / refined_quefrency as f32,
                        quefrency,
                        prominence,
                    }
                })
                .collect()
        })
        .collect();

    CepstralPitchResult { candidates }
}

fn merge_close_peaks(mut peaks: Vec<usize>, data: &[f32], min_separation: usize) -> Vec<usize> {
    peaks.sort_unstable();
    let mut result: Vec<usize> = Vec::new();

    for peak in peaks {
        if let Some(&last) = result.last() {
            if peak - last < min_separation {
                if data[peak] > data[last] {
                    result.pop();
                    result.push(peak);
                }
                continue;
            }
        }
        result.push(peak);
    }

    result
}
