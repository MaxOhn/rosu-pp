use crate::{
    taiko::difficulty::{
        evaluators::ReadingEvaluator,
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
    util::{difficulty::logistic, sync::Weak},
};

define_skill! {
    #[derive(Clone)]
    pub struct Reading: StrainDecaySkill => TaikoDifficultyObjects[TaikoDifficultyObject] {
        current_strain: f64 = 0.0,
    }
}

impl Reading {
    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 0.4;

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject, _: &TaikoDifficultyObjects) -> f64 {
        // * Drum Rolls and Swells are exempt.
        if !curr.base_hit_type.is_hit() {
            return 0.0;
        }

        let index = curr
            .color_data
            .mono_streak
            .as_ref()
            .and_then(Weak::upgrade)
            .and_then(|mono| {
                mono.get().hit_objects.iter().position(|h| {
                    let Some(h) = h.upgrade() else { return false };
                    let h = h.get();

                    h.idx == curr.idx
                })
            })
            .unwrap_or(0) as isize;

        self.current_strain *= logistic(index as f64, 4.0, -1.0 / 25.0, Some(0.5)) + 0.5;
        self.current_strain *= Self::STRAIN_DECAY_BASE;
        self.current_strain += ReadingEvaluator::evaluate_diff_of(curr) * Self::SKILL_MULTIPLIER;

        self.current_strain
    }
}
