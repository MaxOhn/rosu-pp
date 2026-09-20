use crate::{
    any::difficulty::skills::strain_decay_skill,
    catch::difficulty::{evaluators::MovementEvaluator, object::CatchDifficultyObject},
};

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

    fn strain_value_at(
        &mut self,
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
    ) -> f64 {
        let sv = self.strain_value_of(curr, diff_objects);

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
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
    ) -> f64 {
        strain_decay_skill::calculate_initial_strain(
            self.skill_current_strain,
            time,
            curr,
            diff_objects,
            Self::STRAIN_DECAY_BASE,
        )
    }

    fn strain_value_of(
        &mut self,
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
    ) -> f64 {
        MovementEvaluator::evaluate_diff_of(curr, diff_objects, self.clock_rate)
    }
}
