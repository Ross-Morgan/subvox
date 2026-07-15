use std::{collections::HashMap, f32::consts::PI};

use realfft::{RealFftPlanner, num_complex::Complex32};

// pub struct FourierTransformer<T: Debug + Copy + Send + Sync + FromPrimitive + Signed + 'static> {
// planners: HashMap<u32, RealFftPlanner<T>>,
// }

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

    #[must_use]
    pub fn fft(&mut self, samples: &mut [f32]) -> Vec<Complex32> {
        let r2c = self.planner.plan_fft_forward(samples.len());

        let mut spectrum = r2c.make_output_vec();
        let mut scratch = r2c.make_scratch_vec();

        r2c.process_with_scratch(samples, &mut spectrum, &mut scratch)
            .expect("realfft forward transform failed");

        spectrum
    }

    #[must_use]
    pub fn ifft(&mut self, spectrum: &mut [Complex32]) -> Vec<f32> {
        let window_len = (spectrum.len() - 1) * 2; // assumes even N
        let c2r = self.planner.plan_fft_inverse(window_len);

        let mut output = c2r.make_output_vec();
        let mut scratch = c2r.make_scratch_vec();

        c2r.process_with_scratch(spectrum, &mut output, &mut scratch)
            .expect("realfft inverse transform failed");

        let norm = 1.0 / window_len as f32;
        for v in output.iter_mut() {
            *v *= norm;
        }

        output
    }

    pub fn stft(
        &mut self,
        samples: &[f32],
        window_size: usize,
        hop_size: usize,
    ) -> Vec<Vec<Complex32>> {
        assert!(hop_size > 0, "hop_size must be non-zero");
        assert!(
            window_size <= samples.len(),
            "window size exceeds singal length"
        );

        let window = self.hann_window(window_size).to_vec();

        let frame_count = (samples.len() - window_size) / hop_size + 1;
        let mut frames = Vec::with_capacity(frame_count);
        let mut windowed = vec![0.0f32; window_size];

        let mut start = 0;
        while start + window_size <= samples.len() {
            for i in 0..window_size {
                windowed[i] = samples[start + i] * window[i];
            }
            frames.push(self.fft(&mut windowed));
            start += hop_size;
        }

        frames
    }
}
