pub struct LinearRegression {
    pub slope: f32,
    pub intercept: f32,
}

/// Performs linear regression on the given data
pub const fn linear_regression(x: &[f32], y: &[f32]) -> LinearRegression {
    assert!(x.len() <= y.len());

    let n = x.len() as f32;

    let mut mean_x = 0.0;
    let mut mean_y = 0.0;

    let mut i = 0;

    while i < x.len() {
        mean_x += x[i];
        mean_y += y[i];
        i += 1;
    }

    mean_x /= n;
    mean_y /= n;

    let mut cov = 0.0f32;
    let mut var_x = 0.0f32;

    i = 0;

    while i < x.len() {
        let dx = x[i] - mean_x;
        cov += dx * (y[i] - mean_y);
        var_x += dx * dx;
        i += 1;
    }

    let slope = if var_x.abs() < f32::EPSILON {
        0.0
    } else {
        cov / var_x
    };

    let intercept = mean_y - slope * mean_x;
    LinearRegression { slope, intercept }
}

/// Finds the local minima in the given data range.
///
/// # Panics
///
/// - `max_idx >= data.len()`
/// - `min_idx > max_idx`
pub fn find_local_minima(data: &[f32], min_idx: usize, max_idx: usize) -> Vec<usize> {
    debug_assert!(
        max_idx < data.len(),
        "max_idx ({max_idx}) must be less than the length of the data ({})",
        data.len()
    );
    debug_assert!(
        min_idx <= max_idx,
        "min_idx ({min_idx}) must be less than or equal to max_idx ({max_idx})"
    );

    let last_idx = data.len() - 1;

    let mut minima = Vec::new();

    for i in min_idx..=max_idx {
        // SAFETY: out-of-bounds cases are handled before any unsafe indexing occurs
        // REVIEW: performance differences between safe and unsafe indexing (it's probably not significant but whatever)
        unsafe {
            let curr = *data.get_unchecked(i);
            let lower_than_prev = i == 0 || curr < *data.get_unchecked(i - 1);
            let lower_than_next = i == last_idx || curr < *data.get_unchecked(i + 1);
            if lower_than_prev && lower_than_next {
                minima.push(i);
            }
        }
    }

    minima
}

/// Finds the local maxima in the given data range.
///
/// # Panics
///
/// - `max_idx >= data.len()`
/// - `min_idx > max_idx`
pub fn find_local_maxima(data: &[f32], min_idx: usize, max_idx: usize) -> Vec<usize> {
    debug_assert!(
        max_idx < data.len(),
        "max_idx ({max_idx}) must be less than the length of the data ({})",
        data.len()
    );
    debug_assert!(
        min_idx <= max_idx,
        "min_idx ({min_idx}) must be less than or equal to max_idx ({max_idx})"
    );

    let last_idx = data.len() - 1;

    let mut minima = Vec::new();

    for i in min_idx..=max_idx {
        // SAFETY: out-of-bounds cases are handled before any unsafe indexing occurs
        // REVIEW: refer to previous REVIEW comment
        unsafe {
            let curr = *data.get_unchecked(i);
            let greater_than_prev = i == 0 || curr > *data.get_unchecked(i - 1);
            let greater_than_next = i == last_idx || curr > *data.get_unchecked(i + 1);
            if greater_than_prev && greater_than_next {
                minima.push(i);
            }
        }
    }

    minima
}

/// Performs [parabolic interpolation](https://en.wikipedia.org/wiki/Successive_parabolic_interpolation) on the given peak index.
pub const fn parabolic_interpolate(data: &[f32], peak_idx: usize) -> f32 {
    if peak_idx == 0 || peak_idx == data.len() - 1 {
        return peak_idx as f32;
    }
    let (y_minus, y0, y_plus) = (data[peak_idx - 1], data[peak_idx], data[peak_idx + 1]);
    let denom = y_minus - 2.0 * y0 + y_plus;
    if denom.abs() < f32::EPSILON {
        return peak_idx as f32;
    }
    let offset = 0.5 * (y_minus - y_plus) / denom;
    peak_idx as f32 + offset
}
