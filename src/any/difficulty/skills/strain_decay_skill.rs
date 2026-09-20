use crate::any::difficulty::{
    object::{HasStartTime, IDifficultyObject},
    skills::{strain_decay_base, strain_skill::StrainSkill},
};

pub trait StrainDecaySkill: StrainSkill {
    #[expect(dead_code, reason = "provided by skill's inherent impl")]
    fn strain_value_of<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) -> f64;
}

/// C# `StrainDecaySkill.StrainValueAt`.
///
/// The skill's inherent `strain_value_at` should call this:
/// ```
/// let sv = self.strain_value_of(curr, objects);
/// strain_decay_skill_strain_value_at(
///     &mut self.skill_current_strain,
///     curr, sv,
///     Self::STRAIN_DECAY_BASE,
///     Self::SKILL_MULTIPLIER,
/// )
/// ```
pub fn strain_value_at<O: IDifficultyObject>(
    current_strain: &mut f64,
    curr: &O,
    strain_value: f64,
    decay_base: f64,
    skill_multiplier: f64,
) -> f64 {
    *current_strain *= strain_decay_base(curr.delta_time(), decay_base);
    *current_strain += strain_value * skill_multiplier;

    *current_strain
}

/// C# `StrainDecaySkill.CalculateInitialStrain`.
///
/// The skill's inherent `calculate_initial_strain` should call this:
/// ```text
/// strain_decay_skill_calculate_initial_strain(
///     self.skill_current_strain,
///     time, curr, objects,
///     Self::STRAIN_DECAY_BASE,
/// )
/// ```
pub fn calculate_initial_strain<O: IDifficultyObject>(
    current_strain: f64,
    time: f64,
    curr: &O,
    objects: &O::DifficultyObjects,
    decay_base: f64,
) -> f64 {
    let prev_start = curr
        .previous(0, objects)
        .map_or(0.0, HasStartTime::start_time);

    current_strain * strain_decay_base(time - prev_start, decay_base)
}
