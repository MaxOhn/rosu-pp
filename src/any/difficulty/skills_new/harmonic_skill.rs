use crate::{
    any::difficulty::skills_new::skill::Skill,
    util::traits::{IEnumerable, IOrderedEnumerable},
};

pub trait HarmonicSkill: Skill {
    const HARMONIC_SCALE: f64 = 1.0;
    const DECAY_EXPONENT: f64 = 0.9;

    fn object_difficulty_of<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    fn process_internal<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64 {
        self.object_difficulty_of(curr, objects)
    }

    #[expect(unused_mut, reason = "staying in-sync with lazer")]
    fn get_transformed_difficulties(&self, mut difficulties: Vec<f64>) -> Vec<f64> {
        difficulties
    }

    fn difficulty_value(
        transformed_object_difficulties: Vec<f64>,
        object_weight_sum: &mut f64,
    ) -> f64;

    fn into_difficulty_value(self) -> f64;

    fn cloned_difficulty_value(&mut self) -> f64;

    fn count_top_weighted_object_difficulties(&self, difficulty_value: f64) -> f64;

    fn difficulty_to_performance(difficulty: f64) -> f64 {
        4.0 * difficulty.powf(3.0)
    }
}

pub fn harmonic_skill_difficulty_value(
    transformed_object_difficulties: &[f64],
    object_weight_sum: &mut f64,
    harmonic_scale: f64,
    decay_exponent: f64,
) -> f64 {
    *object_weight_sum = 0.0;

    if transformed_object_difficulties.is_empty() {
        return 0.0;
    }

    let mut difficulty = 0.0;
    let mut index = 0;

    // * Objects with 0 difficulty are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
    // * These objects will not contribute to the difficulty.
    for obj in transformed_object_difficulties
        .to_vec()
        .cs_order_descending()
        .cs_where(|v| *v > 0.0)
    {
        // * Use a harmonic sum that considers each object of the map according to a predefined weight.
        let weight = (1.0 + (harmonic_scale / f64::from(1 + index)))
            / (f64::from(index).powf(decay_exponent)
                + 1.0
                + (harmonic_scale / f64::from(1 + index)));

        *object_weight_sum += weight;

        difficulty += obj * weight;
        index += 1;
    }

    difficulty
}
