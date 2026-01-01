use crate::{
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::{StrainSkill, strain_decay},
    },
    osu::difficulty::{
        evaluators::{RhythmEvaluator, SpeedEvaluator},
        object::OsuDifficultyObject,
    },
    util::strains_vec::StrainsVec,
};

use super::strain::OsuStrainSkill;

define_skill! {
    #[derive(Clone)]
    pub struct Speed: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64 = 0.0,
        current_rhythm: f64 = 0.0,
        hit_window: f64,
        has_autopilot_mod: bool,
        slider_strains: Vec<f64> = Vec::with_capacity(64),
    }
}

impl Speed {
    const SKILL_MULTIPLIER: f64 = 1.47;
    const STRAIN_DECAY_BASE: f64 = 0.3;
    const REDUCED_SECTION_COUNT: usize = 5;

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        (self.current_strain * self.current_rhythm)
            * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.adjusted_delta_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += SpeedEvaluator::evaluate_diff_of(
            curr,
            objects,
            self.hit_window,
            self.has_autopilot_mod,
        ) * Self::SKILL_MULTIPLIER;
        self.current_rhythm = RhythmEvaluator::evaluate_diff_of(curr, objects, self.hit_window);

        let total_strain = self.current_strain * self.current_rhythm;

        if curr.base.is_slider() {
            self.slider_strains.push(total_strain);
        }

        total_strain
    }

    pub fn relevant_note_count(&self) -> f64 {
        self.strain_skill_object_strains
            .iter()
            .copied()
            .max_by(f64::total_cmp)
            .filter(|&n| n > 0.0)
            .map_or(0.0, |max_strain| {
                self.strain_skill_object_strains
                    .iter()
                    .fold(0.0, |sum, strain| {
                        sum + (1.0 + f64::exp(-(strain / max_strain * 12.0 - 6.0))).recip()
                    })
            })
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

impl OsuStrainSkill for Speed {}
