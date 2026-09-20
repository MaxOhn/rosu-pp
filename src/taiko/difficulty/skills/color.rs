use crate::{
    any::difficulty::skills::strain_decay_skill,
    taiko::difficulty::{
        evaluators::ColorEvaluator,
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
};

define_skill! {
    #[derive(Clone)]
    pub struct Color: StrainDecaySkill => TaikoDifficultyObjects[TaikoDifficultyObject] {}
}

impl Color {
    const SKILL_MULTIPLIER: f64 = 0.12;
    const STRAIN_DECAY_BASE: f64 = 0.8;

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

    #[expect(clippy::unused_self, reason = "consistency with other skills")]
    fn strain_value_of(
        &mut self,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        ColorEvaluator::evaluate_difficulty_of(curr, objects)
    }
}
