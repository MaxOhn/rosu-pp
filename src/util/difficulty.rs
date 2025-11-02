use std::f64::consts::E;

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

pub fn norm<const N: usize>(p: f64, values: [f64; N]) -> f64 {
    values
        .into_iter()
        .map(|x| f64::powf(x, p))
        .sum::<f64>()
        .powf(p.recip())
}

pub fn bell_curve(x: f64, mean: f64, width: f64, multiplier: Option<f64>) -> f64 {
    multiplier.unwrap_or(1.0) * f64::exp(E * -(f64::powf(x - mean, 2.0) / f64::powf(width, 2.0)))
}

pub fn smoothstep_bell_curve(x: f64, mean: f64, width: f64) -> f64 {
    let mut new_x = x;

    new_x -= mean;
    new_x = if new_x > 0.0 {
        width - new_x
    } else {
        width + new_x
    };

    smoothstep(new_x, 0.0, width)
}

pub const fn smoothstep(x: f64, start: f64, end: f64) -> f64 {
    let x = reverse_lerp(x, start, end);

    x * x * (3.0 - 2.0 * x)
}

pub const fn smootherstep(x: f64, start: f64, end: f64) -> f64 {
    let x = reverse_lerp(x, start, end);

    x * x * x * (x * (6.0 * x - 15.0) + 10.0)
}

pub const fn reverse_lerp(x: f64, start: f64, end: f64) -> f64 {
    f64::clamp((x - start) / (end - start), 0.0, 1.0)
}
