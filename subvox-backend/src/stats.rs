pub struct LinearRegression {
    pub slope: f32,
    pub intercept: f32,
}

pub fn linear_regression(x: &[f32], y: &[f32]) -> LinearRegression {
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

pub fn find_local_minima(data: &[f32], min_idx: usize, max_idx: usize) -> Vec<usize> {
    let mut minima = Vec::new();

    for i in min_idx..=max_idx {
        let lower_than_prev = i == 0 || data[i] < data[i - 1];
        let lower_than_next = i == data.len() - 1 || data[i] < data[i + 1];

        if lower_than_prev && lower_than_next {
            minima.push(i);
        }
    }

    minima
}

pub fn find_local_maxima(data: &[f32], min_idx: usize, max_idx: usize) -> Vec<usize> {
    let mut peaks = Vec::new();

    for i in min_idx..=max_idx {
        let higher_than_prev = i == 0 || data[i] > data[i - 1];
        let higher_than_next = i == data.len() - 1 || data[i] > data[i + 1];

        if higher_than_prev && higher_than_next {
            peaks.push(i);
        }
    }

    peaks
}

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
