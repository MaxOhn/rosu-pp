use crate::taiko::difficulty::{
    evaluators::ColorEvaluator,
    object::{TaikoDifficultyObject, TaikoDifficultyObjects},
};

define_skill! {
    #[derive(Clone)]
    pub struct Color: StrainDecaySkill => TaikoDifficultyObjects[TaikoDifficultyObject] {}
}

impl Color {
    const SKILL_MULTIPLIER: f64 = 0.12;
    const STRAIN_DECAY_BASE: f64 = 0.8;

    #[expect(clippy::unused_self, reason = "required by `define_skill!` macro")]
    fn strain_value_of(
        &self,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        ColorEvaluator::evaluate_difficulty_of(curr, objects)
    }
}
