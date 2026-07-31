use crate::util::{
    difficulty::logistic,
    float_ext::FloatExt,
    hint::unlikely,
};

pub mod harmonic_skill;
pub mod skill;
pub mod strain_skill;
pub mod strain_decay_skill;
pub mod variable_length_strain_skill;

pub fn count_top_weighted_object_difficulties(
    difficulty_value: f64,
    object_difficulties: &[f64],
    object_weight_sum: f64,
) -> f64 {
    if object_difficulties.is_empty() || FloatExt::eq(object_weight_sum, 0.0) {
        return 0.0;
    }

    // * What would the top difficulty be if all object difficulties were identical
    let consistent_top_obj = difficulty_value / object_weight_sum;

    if consistent_top_obj == 0.0 {
        0.0
    } else {
        object_difficulties
            .iter()
            .fold(0.0, |acc, od| acc + logistic(od / consistent_top_obj, 0.88, 10.0, Some(1.1)))
    }
}

pub fn count_top_weighted_strains(
    object_difficulties: &[f64],
    difficulty_value: f64,
    decay_weight: f64,
) -> f64 {
    if unlikely(object_difficulties.is_empty()) {
        return 0.0;
    }

    // * What would the top strain be if all strain values were identical
    let consistent_top_strain = difficulty_value * (1.0 - decay_weight);

    if unlikely(FloatExt::eq(consistent_top_strain, 0.0)) {
        return object_difficulties.len() as f64;
    }

    // * Use a weighted sum of all strains. Constants are arbitrary and give nice values
    object_difficulties
        .iter()
        .map(|s| logistic(s / consistent_top_strain, 0.88, 10.0, Some(1.1)))
        .sum()
}

pub fn strain_decay_base(ms: f64, strain_decay_base: f64) -> f64 {
    f64::powf(strain_decay_base, ms / 1000.0)
}