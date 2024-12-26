pub const fn bpm_to_milliseconds(bpm: f64, delimiter: Option<i32>) -> f64 {
    60_000.0 / i32_unwrap_or(delimiter, 4) as f64 / bpm
}

pub const fn milliseconds_to_bpm(ms: f64, delimiter: Option<i32>) -> f64 {
    60_000.0 / (ms * i32_unwrap_or(delimiter, 4) as f64)
}

// `Option::unwrap_or` is not const
const fn i32_unwrap_or(option: Option<i32>, default: i32) -> i32 {
    match option {
        Some(value) => value,
        None => default,
    }
}

// `f64::exp` is not const
pub fn logistic(x: f64, midpoint_offset: f64, multiplier: f64, max_value: Option<f64>) -> f64 {
    max_value.unwrap_or(1.0) / (1.0 + f64::exp(multiplier * (midpoint_offset - x)))
}

// `f64::exp` is not const
pub fn logistic_exp(exp: f64, max_value: Option<f64>) -> f64 {
    max_value.unwrap_or(1.0) / (1.0 + f64::exp(exp))
}
