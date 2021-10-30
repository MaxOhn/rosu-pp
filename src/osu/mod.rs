#![cfg(feature = "osu")]

mod pp;
mod versions;

pub use pp::*;
pub use versions::*;

/// Various data created through the star calculation.
/// This data is necessary to calculate PP.
#[derive(Clone, Debug, Default)]
pub struct DifficultyAttributes {
    pub aim_strain: f32,
    pub speed_strain: f32,
    pub flashlight_rating: f32,
    pub ar: f32,
    pub od: f32,
    pub hp: f32,
    pub n_circles: usize,
    pub n_sliders: usize,
    pub n_spinners: usize,
    pub stars: f32,
    pub max_combo: usize,
}
