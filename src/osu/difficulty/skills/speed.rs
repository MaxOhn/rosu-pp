use crate::{
    GameMods,
    any::difficulty::skills_new::strain_decay_base,
    osu::difficulty::{
        evaluators::{RhythmEvaluator, SpeedEvaluator},
        object::OsuDifficultyObject,
    },
    util::{difficulty::logistic, float_ext::FloatExt},
};

define_new_skill! {
    #[derive(Clone)]
    pub struct Speed: HarmonicSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        slider_strains: Vec<f64> = Vec::with_capacity(64),
        current_strain: f64 = 0.0,
        mods: GameMods,
        hit_window: f64,
    }
}

impl Speed {
    const SKILL_MULTIPLIER: f64 = 1.16;
    const HARMONIC_SCALE: f64 = 20.0;

    fn strain_decay(ms: f64) -> f64 {
        strain_decay_base(ms, 0.3)
    }

    fn object_difficulty_of<'a>(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        let decay = Self::strain_decay(curr.adjusted_delta_time);

        self.current_strain *= decay;
        self.current_strain += self.calculate_adjusted_difficulty(curr, objects)
            * (1.0 - decay)
            * Self::SKILL_MULTIPLIER;

        let curr_rhythm = RhythmEvaluator::evaluate_diff_of(curr, objects, self.hit_window);

        let total_strain = self.current_strain * curr_rhythm;

        if curr.base.is_slider() {
            self.slider_strains.push(total_strain);
        }

        total_strain
    }

    fn calculate_adjusted_difficulty<'a>(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        let mut difficulty = SpeedEvaluator::evaluate_diff_of(curr, objects, self.hit_window);

        if self.mods.ap() {
            difficulty *= 0.5;
        }

        difficulty
    }

    pub fn relevant_object_count(&self) -> f64 {
        if self.skill_object_difficulties.is_empty() {
            return 0.0;
        }

        let max_strain = self
            .skill_object_difficulties
            .iter()
            .copied()
            .fold(0.0, f64::max);

        if FloatExt::eq(max_strain, 0.0) {
            return 0.0;
        }

        self.skill_object_difficulties
            .iter()
            .map(|s| logistic(s / max_strain, 0.5, 12.0, None))
            .sum()
    }

    pub fn count_top_weighted_sliders(&self, difficulty_value: f64, object_weight_sum: f64) -> f64 {
        if self.slider_strains.is_empty() || FloatExt::eq(object_weight_sum, 0.0) {
            return 0.0;
        }

        // * What would the top note be if all note values were identical
        let consistent_top_object = difficulty_value / object_weight_sum;

        super::count_top_weighted_sliders(&self.slider_strains, consistent_top_object)
    }
}
