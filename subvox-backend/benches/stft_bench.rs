use std::{hint::black_box, sync::Arc};

use criterion::{Criterion, criterion_group, criterion_main};
use subvox_backend::{FourierTransformer, load_audio_file};

fn bench_stft(c: &mut Criterion) {
    let file = load_audio_file("../assets/growl.wav").unwrap();
    let frames = file.to_f32_vec();
    let mut transformer = FourierTransformer::new();

    let mut group = c.benchmark_group("stft");

    group.sampling_mode(criterion::SamplingMode::Flat);

    for window_size in (10..16).map(|i| 2usize.pow(i)).into_iter() {
        for hop_size in [window_size / 4, window_size / 2, window_size] {
            group.bench_function(format!("{window_size}_window_{hop_size}_hop"), |b| {
                b.iter(|| transformer.stft(black_box(&frames), window_size, hop_size))
            });
        }
    }

    group.finish();
}

fn bench_par_stft(c: &mut Criterion) {
    let file = load_audio_file("../assets/growl.wav").unwrap();
    let frames = file.to_f32_vec();
    let mut planner = realfft::RealFftPlanner::<f32>::new();
    let mut group = c.benchmark_group("stft");

    group.sampling_mode(criterion::SamplingMode::Flat);

    for window_size in (10..16).map(|i| 2usize.pow(i)).into_iter() {
        let r2c = planner.plan_fft_forward(window_size);
        for hop_size in [window_size / 4, window_size / 2, window_size] {
            group.bench_function(format!("rayon {window_size}_window_{hop_size}_hop"), |b| {
                b.iter(|| {
                    subvox_backend::par_stft(
                        black_box(&frames),
                        window_size,
                        hop_size,
                        Arc::clone(&r2c),
                    )
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_stft, bench_par_stft);
criterion_main!(benches);
