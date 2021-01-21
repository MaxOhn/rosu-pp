#[cfg(feature = "all_included")]
pub mod all_included;

#[cfg(feature = "no_leniency")]
pub mod no_leniency;

#[cfg(feature = "no_sliders_no_leniency")]
pub mod no_sliders_no_leniency;

const OSU_OD_MAX: f32 = 20.0;
const OSU_OD_AVG: f32 = 50.0;
const OSU_OD_MIN: f32 = 80.0;

#[inline]
fn difficulty_range(od: f32) -> f32 {
    super::super::difficulty_range(od, OSU_OD_MAX, OSU_OD_AVG, OSU_OD_MIN)
}
