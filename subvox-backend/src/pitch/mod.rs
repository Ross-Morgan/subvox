mod cpp;
mod hps;
mod yin;

use std::collections::HashMap;

pub use cpp::{CepstralPitchCandidate, CepstralPitchResult, cpp_pitch_candidates};
pub use hps::{HpsPitchCandidate, HpsPitchResult, hps_pitch_candidates};
pub use yin::{YinPitchCandidate, YinPitchResult, yin_pitch_candidates};

use crate::{Note, cepstrum::CepstrumResult, fft::StftResult, notes::NoteKey};

pub struct PitchAlgorithmCandidateDistribution {
    pub cpp: usize,
    pub hps: usize,
    pub yin: usize,
}

// TODO: Improve voting. This hardly works and doesn't normalise different algorithms' output ranges
pub fn combined_pitch_estimate(
    frames: &[f32],
    spectra: &StftResult,
    cepstra: &CepstrumResult,
    sample_rate: u32,
    window_size: usize,
    hop_size: usize,
    min_frequency: f32,
    max_frequency: f32,
    algorithm_distribution: PitchAlgorithmCandidateDistribution,
) -> Vec<NoteKey> {
    #[cfg(debug_assertions)]
    println!("Computing CPP pitch candidates...");

    let cpp_candidates = cpp_pitch_candidates(
        cepstra,
        sample_rate as f32,
        min_frequency,
        max_frequency,
        algorithm_distribution.cpp,
    );

    #[cfg(debug_assertions)]
    println!("Computing HPS pitch candidates...");

    let hps_candidates = hps_pitch_candidates(
        spectra,
        sample_rate as f32,
        min_frequency,
        max_frequency,
        5,
        algorithm_distribution.hps,
    );

    #[cfg(debug_assertions)]
    println!("Computing YIN pitch candidates...");

    let yin_candidates = yin_pitch_candidates(
        frames,
        window_size,
        hop_size,
        sample_rate as f32,
        min_frequency,
        max_frequency,
        0.1,
        algorithm_distribution.yin,
    );

    #[cfg(debug_assertions)]
    println!("Merging pitch candidates...");

    cpp_candidates
        .candidates
        .iter()
        .zip(hps_candidates.candidates.iter())
        .zip(yin_candidates.candidates.iter())
        .map(|((cpp, hps), yin)| {
            let mut map = HashMap::new();

            for cpp_candidate in cpp.iter() {
                map.entry(Note::new(cpp_candidate.frequency).key())
                    .and_modify(|p| *p += cpp_candidate.prominence)
                    .or_insert(cpp_candidate.prominence);

                dbg!(cpp_candidate.prominence);
            }

            for hps_candidate in hps.iter() {
                map.entry(Note::new(hps_candidate.frequency).key())
                    .and_modify(|p| *p += hps_candidate.prominence)
                    .or_insert(hps_candidate.prominence);

                dbg!(hps_candidate.prominence);
            }

            // TODO: No idea what range `cmnd_value` can lie in. Experiment.
            for yin_candidate in yin.iter() {
                map.entry(Note::new(yin_candidate.frequency).key())
                    .and_modify(|p| *p += yin_candidate.cmnd_value.recip())
                    .or_insert(yin_candidate.cmnd_value.recip());

                dbg!(yin_candidate.cmnd_value.recip());
            }

            *map.iter()
                .max_by(|&(_, a), &(_, b)| a.total_cmp(b))
                .map(|(key, _)| key)
                .unwrap()
        })
        .collect::<Vec<_>>()
}
