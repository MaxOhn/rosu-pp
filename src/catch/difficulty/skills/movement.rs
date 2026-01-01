use crate::catch::difficulty::{evaluators::MovementEvaluator, object::CatchDifficultyObject};

define_skill! {
    pub struct Movement: StrainDecaySkill => [CatchDifficultyObject][CatchDifficultyObject] {
        clock_rate: f64,
    }
}

impl Movement {
    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 0.2;

    const DECAY_WEIGHT: f64 = 0.94;

    const SECTION_LENGTH: f64 = 750.0;

    fn strain_value_of(
        &mut self,
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
    ) -> f64 {
        MovementEvaluator::evaluate_diff_of(curr, diff_objects, self.clock_rate)
    }
}
