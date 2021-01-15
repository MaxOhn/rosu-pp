pub mod fruits;
pub mod mania;
pub mod osu;
pub mod parse;
pub mod taiko;

mod curve;
mod math_util;

pub use parse::*;

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
