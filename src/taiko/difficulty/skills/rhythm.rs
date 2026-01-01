use crate::{
    taiko::difficulty::{
        evaluators::{RhythmEvaluator, StaminaEvaluator},
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
    util::difficulty::logistic,
};

define_skill! {
    #[derive(Clone)]
    pub struct Rhythm: StrainDecaySkill => TaikoDifficultyObjects[TaikoDifficultyObject] {
        great_hit_window: f64,
    }
}

impl Rhythm {
    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 0.4;

    fn strain_value_of(
        &mut self,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        let mut difficulty = RhythmEvaluator::evaluate_diff_of(curr, self.great_hit_window);

        // * To prevent abuse of exceedingly long intervals between awkward rhythms, we penalise its difficulty.
        let stamina_difficulty = StaminaEvaluator::evaluate_diff_of(curr, objects) - 0.5; // * Remove base strain
        difficulty *= logistic(stamina_difficulty, 1.0 / 15.0, 50.0, None);

        difficulty
    }
}
