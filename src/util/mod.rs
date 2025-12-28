pub mod difficulty;
pub mod float_ext;
pub mod hint;
pub mod interval_grouping;
pub mod limited_queue;
pub mod map_or_attrs;
pub mod random;
pub mod ruleset_ext;
pub mod sort;
pub mod special_functions;
pub mod strains_vec;
pub mod sync;

#[macro_use]
mod macros;

pub fn get_precision_adjusted_beat_len(slider_velocity_multiplier: f64, beat_len: f64) -> f64 {
    let slider_velocity_as_beat_len = -100.0 / slider_velocity_multiplier;

    let bpm_multiplier = if slider_velocity_as_beat_len < 0.0 {
        f64::from(((-slider_velocity_as_beat_len) as f32).clamp(10.0, 10_000.0)) / 100.0
    } else {
        1.0
    };

    beat_len * bpm_multiplier
}
