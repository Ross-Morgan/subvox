use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

static HANN_WINDOW_CACHE: LazyLock<Mutex<HashMap<usize, Arc<[f32]>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn compute_hann_window(window_size: usize) -> Vec<f32> {
    (0..window_size)
        .map(|n| {
            (std::f32::consts::PI * n as f32 / (window_size as f32 - 1.0))
                .sin()
                .powi(2)
        })
        .collect()
}

/// Returns a shared reference to the cached Hann window of the given size
///
/// The window is inserted into the cache if it is not already present
pub fn get_hann_window(window_size: usize) -> Arc<[f32]> {
    let mut cache = HANN_WINDOW_CACHE.lock().unwrap();

    Arc::clone(
        &cache
            .entry(window_size)
            .or_insert_with(|| compute_hann_window(window_size).into()),
    )
}
