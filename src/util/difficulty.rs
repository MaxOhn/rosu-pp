use std::f64::consts::{E, PI};

use crate::util::float_ext::FloatExt;

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

pub fn erf(x: f64) -> f64 {
    #![expect(clippy::unreadable_literal, reason = "staying in-sync with lazer")]

    if FloatExt::eq(x, 0.0) {
        return 0.0;
    }

    if x.is_infinite() {
        if x.is_sign_positive() {
            return 1.0;
        }

        if x.is_sign_negative() {
            return -1.0;
        }
    }

    if x.is_nan() {
        return f64::NAN;
    }

    // * Constants for approximation (Abramowitz and Stegun formula 7.1.26)
    let t = 1.0 / (1.0 + 0.3275911 * f64::abs(x));

    let tau = t
        * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));

    let erf = 1.0 - tau * f64::exp(-x * x);

    if x >= 0.0 { erf } else { -erf }
}

pub fn erf_inv(mut x: f64) -> f64 {
    if x <= -1.0 {
        return f64::NEG_INFINITY;
    }

    if x >= 1.0 {
        return f64::INFINITY;
    }

    if FloatExt::eq(x, 0.0) {
        return 0.0;
    }

    #[expect(clippy::items_after_statements, reason = "staying in-sync with lazer")]
    const A: f64 = 0.147;
    let sgn = f64::signum(x);
    x = f64::abs(x);

    let ln = f64::ln(1.0 - x * x);
    let t1 = 2.0 / (PI * A) + ln / 2.0;
    let t2 = ln / A;
    let base_approx = f64::sqrt(t1 * t1 - t2) - t1;

    // * Correction reduces max error from -0.005 to -0.00045.
    let c = if x >= 0.85 {
        f64::powf((x - 0.85) / 0.293, 8.0)
    } else {
        0.0
    };

    sgn * (f64::sqrt(base_approx) + c)
}
