mod pp;
mod versions;

pub use pp::*;
pub use versions::*;

#[derive(Clone, Default, Debug)]
pub struct DifficultyAttributes {
    pub stars: f32,
    pub ar: f32,
    pub od: f32,
    pub speed_strain: f32,
    pub aim_strain: f32,
    pub max_combo: usize,
    pub n_circles: usize,
    pub n_spinners: usize,
}

const HITWINDOW_OD_MIN: f32 = 80.0;
const HITWINDOW_OD_AVG: f32 = 50.0;
const HITWINDOW_OD_MAX: f32 = 20.0;

const HITWINDOW_AR_MIN: f32 = 1800.0;
const HITWINDOW_AR_AVG: f32 = 1200.0;
const HITWINDOW_AR_MAX: f32 = 450.0;

#[inline]
pub(crate) fn difficulty_range_od(od: f32) -> f32 {
    crate::difficulty_range(od, HITWINDOW_OD_MAX, HITWINDOW_OD_AVG, HITWINDOW_OD_MIN)
}

#[inline]
pub(crate) fn difficulty_range_ar(ar: f32) -> f32 {
    crate::difficulty_range(ar, HITWINDOW_AR_MAX, HITWINDOW_AR_AVG, HITWINDOW_AR_MIN)
}
