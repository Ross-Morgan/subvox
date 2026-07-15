use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use subvox_backend::{FourierTransformer, load_audio_file};

fn bench_stft(c: &mut Criterion) {
    let file = load_audio_file("../assets/vocal-1.wav").unwrap();
    let frames = file.to_f32_vec();
    let mut transformer = FourierTransformer::new();

    // TODO: Warm up plan cache?

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

criterion_group!(benches, bench_stft);
criterion_main!(benches);
