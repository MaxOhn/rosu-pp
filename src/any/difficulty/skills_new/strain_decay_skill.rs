use crate::any::difficulty::skills_new::strain_skill::NewStrainSkill;

pub trait NewStrainDecaySkill: NewStrainSkill {
    const SKILL_MULTIPLIER: f64 = 0.0;
    const STRAIN_DECAY_BASE: f64 = 0.0;

    fn strain_value_of<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;

    fn strain_decay(ms: f64) -> f64;
}
