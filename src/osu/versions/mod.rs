#[cfg(feature = "all_included")]
#[cfg_attr(docsrs, doc(cfg(feature = "all_included")))]
mod all_included;

#[cfg(feature = "all_included")]
pub use all_included::*;

#[cfg(feature = "no_leniency")]
#[cfg_attr(docsrs, doc(cfg(feature = "no_leniency")))]
mod no_leniency;

#[cfg(feature = "no_leniency")]
pub use no_leniency::*;

#[cfg(feature = "no_sliders_no_leniency")]
#[cfg_attr(docsrs, doc(cfg(feature = "no_sliders_no_leniency")))]
mod no_sliders_no_leniency;

#[cfg(feature = "no_sliders_no_leniency")]
pub use no_sliders_no_leniency::*;

#[inline]
fn difficulty_range_od(od: f32) -> f32 {
    super::super::difficulty_range(od, 20.0, 50.0, 80.0)
}
