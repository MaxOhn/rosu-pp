use crate::{
    any::difficulty::skills::strain_decay_skill,
    taiko::difficulty::{
        evaluators::{RhythmEvaluator, StaminaEvaluator},
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
    util::difficulty as diff_utils,
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

    fn strain_value_at(
        &mut self,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        let sv = self.strain_value_of(curr, objects);

        strain_decay_skill::strain_value_at(
            &mut self.skill_current_strain,
            curr,
            sv,
            Self::STRAIN_DECAY_BASE,
            Self::SKILL_MULTIPLIER,
        )
    }

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        strain_decay_skill::calculate_initial_strain(
            self.skill_current_strain,
            time,
            curr,
            objects,
            Self::STRAIN_DECAY_BASE,
        )
    }

    fn strain_value_of(
        &mut self,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        let mut difficulty = RhythmEvaluator::evaluate_diff_of(curr, self.great_hit_window);

        // * To prevent abuse of exceedingly long intervals between awkward rhythms, we penalise its difficulty.
        let stamina_difficulty = StaminaEvaluator::evaluate_diff_of(curr, objects) - 0.5; // * Remove base strain
        difficulty *= diff_utils::logistic(stamina_difficulty, 1.0 / 15.0, 50.0, None);

        difficulty
    }
}
