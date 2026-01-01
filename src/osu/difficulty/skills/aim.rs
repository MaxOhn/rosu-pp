use crate::{
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::{StrainSkill, strain_decay},
    },
    osu::difficulty::{evaluators::AimEvaluator, object::OsuDifficultyObject},
    util::{float_ext::FloatExt, strains_vec::StrainsVec},
};

use super::strain::OsuStrainSkill;

define_skill! {
    #[derive(Clone)]
    pub struct Aim: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        include_sliders: bool,
        current_strain: f64 = 0.0,
        slider_strains: Vec<f64> = Vec::with_capacity(64), // TODO: use `StrainsVec`?
    }
}

impl Aim {
    const SKILL_MULTIPLIER: f64 = 26.0;
    const STRAIN_DECAY_BASE: f64 = 0.15;

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.delta_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += AimEvaluator::evaluate_diff_of(curr, objects, self.include_sliders)
            * Self::SKILL_MULTIPLIER;

        if curr.base.is_slider() {
            self.slider_strains.push(self.current_strain);
        }

        self.current_strain
    }

    pub fn get_difficult_sliders(&self) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }

        let max_slider_strain = self.slider_strains.iter().copied().fold(0.0, f64::max);

        if FloatExt::eq(max_slider_strain, 0.0) {
            return 0.0;
        }

        self.slider_strains
            .iter()
            .copied()
            .map(|strain| 1.0 / (1.0 + f64::exp(-(strain / max_slider_strain * 12.0 - 6.0))))
            .sum()
    }

    pub fn slider_strains(&self) -> &[f64] {
        &self.slider_strains
    }

    // From `OsuStrainSkill`; native rather than trait function so that it has
    // priority over `StrainSkill::difficulty_value`
    fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
        super::strain::difficulty_value(
            current_strain_peaks,
            Self::REDUCED_SECTION_COUNT,
            Self::REDUCED_STRAIN_BASELINE,
            Self::DECAY_WEIGHT,
        )
    }
}

impl OsuStrainSkill for Aim {}
