mod versions;
pub use versions::*;

mod curve;
mod difficulty_attributes;
mod math_util;
mod pp;

pub use difficulty_attributes::DifficultyAttributes;
pub use pp::*;

const HITWINDOW_OD_MIN: f32 = 80.0;
const HITWINDOW_OD_AVG: f32 = 50.0;
const HITWINDOW_OD_MAX: f32 = 20.0;

const HITWINDOW_AR_MIN: f32 = 1800.0;
const HITWINDOW_AR_AVG: f32 = 1200.0;
const HITWINDOW_AR_MAX: f32 = 450.0;

#[inline]
pub(crate) fn difficulty_range_od(od: f32) -> f32 {
    difficulty_range(od, HITWINDOW_OD_MAX, HITWINDOW_OD_AVG, HITWINDOW_OD_MIN)
}

#[inline]
pub(crate) fn difficulty_range_ar(ar: f32) -> f32 {
    difficulty_range(ar, HITWINDOW_AR_MAX, HITWINDOW_AR_AVG, HITWINDOW_AR_MIN)
}

#[inline]
fn difficulty_range(val: f32, max: f32, avg: f32, min: f32) -> f32 {
    if val > 5.0 {
        avg + (max - avg) * (val - 5.0) / 5.0
    } else if val < 5.0 {
        avg - (avg - min) * (5.0 - val) / 5.0
    } else {
        avg
    }
}
