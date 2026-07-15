use std::{collections::HashMap, f32::consts::PI};

use realfft::{RealFftPlanner, num_complex::Complex32};

pub struct StftResult {
    data: Vec<Complex32>,
    bins: usize,
}

impl StftResult {
    pub fn frame(&self, i: usize) -> &[Complex32] {
        &self.data[i * self.bins..(i + 1) * self.bins]
    }
    pub fn frame_count(&self) -> usize {
        self.data.len() / self.bins
    }
}

pub struct FourierTransformer {
    planner: RealFftPlanner<f32>,
    // TODO: Implement generic windowing strategy
    window_cache: HashMap<usize, Vec<f32>>,
}

impl FourierTransformer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            planner: RealFftPlanner::new(),
            window_cache: HashMap::new(),
        }
    }

    // TODO: Move function elsewhere. Put here for convenience
    fn hann_window(&mut self, size: usize) -> &[f32] {
        self.window_cache.entry(size).or_insert_with(|| {
            (0..size)
                .map(|n| (PI * n as f32 / (size as f32 - 1.0)).sin().powi(2))
                .collect()
        })
    }

    pub fn fft(
        &mut self,
        samples: &mut [f32],
        scratch: &mut [Complex32],
        output: &mut [Complex32],
    ) {
        let r2c = self.planner.plan_fft_forward(samples.len());

        r2c.process_with_scratch(samples, output, scratch)
            .expect("realfft forward transform failed");
    }

    pub fn ifft(
        &mut self,
        spectrum: &mut [Complex32],
        scratch: &mut [Complex32],
        output: &mut [f32],
    ) {
        let window_len = (spectrum.len() - 1) * 2; // assumes even N
        let c2r = self.planner.plan_fft_inverse(window_len);

        c2r.process_with_scratch(spectrum, output, scratch)
            .expect("realfft inverse transform failed");

        let norm = 1.0 / window_len as f32;
        for v in output.iter_mut() {
            *v *= norm;
        }
    }

    pub fn stft(&mut self, samples: &[f32], window_size: usize, hop_size: usize) -> StftResult {
        assert!(hop_size > 0, "hop_size must be non-zero");
        assert!(
            window_size <= samples.len(),
            "window size exceeds singal length"
        );

        let window = self.hann_window(window_size).to_vec();
        let frame_count = (samples.len() - window_size) / hop_size + 1;
        let bins = window_size / 2 + 1;

        let r2c = self.planner.plan_fft_forward(window_size);

        let mut frames = vec![Complex32::ZERO; frame_count * bins];
        let mut windowed = vec![0.0; window_size];
        let mut scratch_buf = r2c.make_scratch_vec();

        let mut start = 0;
        let mut frame_idx = 0;

        while start + window_size <= samples.len() {
            for i in 0..window_size {
                windowed[i] = samples[start + i] * window[i];
            }

            let out_slice = &mut frames[(frame_idx * bins)..((frame_idx + 1) * bins)];

            self.fft(&mut windowed, scratch_buf.as_mut_slice(), out_slice);

            start += hop_size;
            frame_idx += 1;
        }

        StftResult { data: frames, bins }
    }
}
