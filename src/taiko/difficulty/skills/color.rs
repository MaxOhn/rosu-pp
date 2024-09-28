use std::f64::consts::E;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainDecaySkill},
    },
    taiko::difficulty::{
        color::{
            alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
            repeating_hit_patterns::RepeatingHitPatterns,
        },
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
    util::{
        strains_vec::StrainsVec,
        sync::{RefCount, Weak},
    },
};

const SKILL_MULTIPLIER: f64 = 0.12;
const STRAIN_DECAY_BASE: f64 = 0.8;

#[derive(Clone, Default)]
pub struct Color {
    inner: StrainDecaySkill,
}

impl Color {
    const fn curr_strain(&self) -> f64 {
        self.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.curr_strain
    }

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.curr_strain_mut() += Self::strain_value_of(curr) * SKILL_MULTIPLIER;

        self.curr_strain()
    }

    fn strain_value_of(curr: &TaikoDifficultyObject) -> f64 {
        ColorEvaluator::evaluate_diff_of(curr)
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn as_difficulty_value(&self) -> f64 {
        self.inner
            .clone()
            .difficulty_value(StrainDecaySkill::DECAY_WEIGHT)
    }
}

impl ISkill for Color {
    type DifficultyObjects<'a> = TaikoDifficultyObjects;
}

impl Skill<'_, Color> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &TaikoDifficultyObject) -> f64 {
        let prev_start_time = curr
            .previous(0, &self.diff_objects.objects)
            .map_or(0.0, |prev| prev.get().start_time);

        self.inner.curr_strain() * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    const fn curr_section_peak(&self) -> f64 {
        self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_peak
    }

    const fn curr_section_end(&self) -> f64 {
        self.inner.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &TaikoDifficultyObject) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / StrainDecaySkill::SECTION_LEN).ceil()
                * StrainDecaySkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += StrainDecaySkill::SECTION_LEN;
        }

        let strain_value_at = self.inner.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }
}

struct ColorEvaluator;

impl ColorEvaluator {
    fn sigmoid(val: f64, center: f64, width: f64, middle: f64, height: f64) -> f64 {
        let sigmoid = (E * -(val - center) / width).tanh();

        sigmoid * (height / 2.0) + middle
    }

    fn evaluate_diff_of_mono_streak(mono_streak: &RefCount<MonoStreak>) -> f64 {
        let mono_streak = mono_streak.get();

        let parent_eval = mono_streak
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .as_ref()
            .map_or(1.0, Self::evaluate_diff_of_alternating_mono_pattern);

        Self::sigmoid(mono_streak.idx as f64, 2.0, 2.0, 0.5, 1.0) * parent_eval * 0.5
    }

    fn evaluate_diff_of_alternating_mono_pattern(
        alternating_mono_pattern: &RefCount<AlternatingMonoPattern>,
    ) -> f64 {
        let alternating_mono_pattern = alternating_mono_pattern.get();

        let parent_eval = alternating_mono_pattern
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .as_ref()
            .map_or(1.0, Self::evaluate_diff_of_repeating_hit_patterns);

        Self::sigmoid(alternating_mono_pattern.idx as f64, 2.0, 2.0, 0.5, 1.0) * parent_eval
    }

    fn evaluate_diff_of_repeating_hit_patterns(
        repeating_hit_patterns: &RefCount<RepeatingHitPatterns>,
    ) -> f64 {
        let repetition_interval = repeating_hit_patterns.get().repetition_interval as f64;

        2.0 * (1.0 - Self::sigmoid(repetition_interval, 2.0, 2.0, 0.5, 1.0))
    }

    fn evaluate_diff_of(hit_object: &TaikoDifficultyObject) -> f64 {
        let color = &hit_object.color;
        let mut difficulty = 0.0;

        if let Some(mono_streak) = color.mono_streak.as_ref().and_then(Weak::upgrade) {
            if let Some(first_hit_object) = mono_streak.get().first_hit_object() {
                if &*first_hit_object.get() == hit_object {
                    difficulty += Self::evaluate_diff_of_mono_streak(&mono_streak);
                }
            }
        }

        if let Some(alternating_mono_pattern) = color
            .alternating_mono_pattern
            .as_ref()
            .and_then(Weak::upgrade)
        {
            if let Some(first_hit_object) = alternating_mono_pattern.get().first_hit_object() {
                if &*first_hit_object.get() == hit_object {
                    difficulty +=
                        Self::evaluate_diff_of_alternating_mono_pattern(&alternating_mono_pattern);
                }
            }
        }

        if let Some(repeating_hit_patterns) = color.repeating_hit_patterns.as_ref() {
            if let Some(first_hit_object) = repeating_hit_patterns.get().first_hit_object() {
                if &*first_hit_object.get() == hit_object {
                    difficulty +=
                        Self::evaluate_diff_of_repeating_hit_patterns(repeating_hit_patterns);
                }
            }
        }

        difficulty
    }
}
