//! YIN

use std::fmt::Debug;

use rayon::prelude::*;

use crate::{
    Note,
    stats::{find_local_minima, parabolic_interpolate},
};

pub struct YinPitchCandidate {
    pub frequency: f32,
    /// CMND value at this lag — lower means more confidently periodic.
    /// Note this is an *inverted* confidence scale relative to HPS/cepstral
    /// prominence (where higher = better); keep this in mind when comparing
    /// or ranking across algorithms later.
    pub cmnd_value: f32,
}

impl Debug for YinPitchCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note = Note::new(self.frequency);
        note.fmt(f)
    }
}

pub struct YinPitchResult {
    pub candidates: Vec<Vec<YinPitchCandidate>>,
}

pub fn yin_pitch_candidates(
    samples: &[f32],
    window_size: usize,
    hop_size: usize,
    sample_rate: f32,
    min_pitch_hz: f32,
    max_pitch_hz: f32,
    threshold: f32,
    max_candidates: usize,
) -> YinPitchResult {
    assert!(hop_size > 0, "hop_size must be non-zero");
    assert!(
        window_size <= samples.len(),
        "window size exceeds signal length"
    );

    let max_tau = (sample_rate / min_pitch_hz).ceil() as usize;
    let min_tau = (sample_rate / max_pitch_hz).floor().max(1.0) as usize;

    assert!(
        min_tau < max_tau,
        "invalid pitch range for given sample_rate"
    );
    assert!(
        max_tau < window_size,
        "window_size must exceed max_tau (sample_rate / min_pitch_hz); \
         increase window_size or raise min_pitch_hz"
    );

    let frame_count = (samples.len() - window_size) / hop_size + 1;

    let candidates = (0..frame_count)
        .into_par_iter()
        .map_init(
            || vec![0.0f32; max_tau + 1],
            |cmnd, frame_idx| {
                let start = frame_idx * hop_size;
                let frame = &samples[start..start + window_size];

                compute_cmnd(frame, max_tau, cmnd);

                let mut minima = find_local_minima(cmnd, min_tau, max_tau);

                // Prefer the first minimum below threshold (classic YIN
                // behavior, biases toward the shortest plausible period
                // rather than an octave-down alternative); fall back to
                // whatever minima exist otherwise.
                minima.sort_by(|&a, &b| cmnd[a].partial_cmp(&cmnd[b]).unwrap());

                if let Some(pos) = minima.iter().position(|&tau| cmnd[tau] < threshold) {
                    minima.swap(0, pos);
                }

                minima.truncate(max_candidates);

                minima
                    .into_iter()
                    .map(|tau| {
                        let refined_tau = parabolic_interpolate(cmnd, tau);
                        YinPitchCandidate {
                            frequency: sample_rate / refined_tau,
                            cmnd_value: cmnd[tau],
                        }
                    })
                    .collect()
            },
        )
        .collect();

    YinPitchResult { candidates }
}

fn compute_cmnd(frame: &[f32], max_tau: usize, cmnd: &mut [f32]) {
    cmnd[0] = 1.0;
    let mut running_sum = 0.0f32;

    for tau in 1..=max_tau {
        let mut diff = 0.0f32;
        for j in 0..(frame.len() - tau) {
            let delta = frame[j] - frame[j + tau];
            diff += delta * delta;
        }

        running_sum += diff;
        cmnd[tau] = if running_sum > f32::EPSILON {
            diff * tau as f32 / running_sum
        } else {
            1.0
        };
    }
}
