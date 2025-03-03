use std::f64::consts::E;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{
            strain_decay, DifficultyValue, ISkill, Skill, StrainDecaySkill, UsedStrainSkills,
        },
    },
    taiko::difficulty::{
        color::data::{
            alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
            repeating_hit_patterns::RepeatingHitPatterns,
        },
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
    util::{
        difficulty::logistic_exp,
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

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn as_difficulty_value(&self) -> UsedStrainSkills<DifficultyValue> {
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

        let strain_value_at = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
        self.inner.inner.inner.object_strains.push(strain_value_at);
    }

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.inner.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.inner.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.inner.curr_strain()
    }

    fn strain_value_of(&self, curr: &TaikoDifficultyObject) -> f64 {
        ColorEvaluator::evaluate_diff_of(curr, self.diff_objects)
    }
}

struct ColorEvaluator;

impl ColorEvaluator {
    fn consistent_ratio_penalty(
        hit_object: &TaikoDifficultyObject,
        diff_objects: &TaikoDifficultyObjects,
        threshold: Option<f64>,
        max_objects_to_check: Option<usize>,
    ) -> f64 {
        let threshold = threshold.unwrap_or(0.01);
        let max_objects_to_check = max_objects_to_check.unwrap_or(64);

        let curr = hit_object;

        let mut consistent_ratio_count = 0;
        let mut total_ratio_count = 0.0;

        let prev_objects =
            &diff_objects.objects[curr.idx.saturating_sub(max_objects_to_check)..curr.idx];

        for window in prev_objects.windows(2).rev() {
            let [prev, curr] = window else { unreachable!() };

            let curr = curr.get();
            let prev = prev.get();

            let curr_ratio = curr.rhythm_data.ratio;
            let prev_ratio = prev.rhythm_data.ratio;

            // * A consistent interval is defined as the percentage difference between the two rhythmic ratios with the margin of error.
            if f64::abs(1.0 - curr_ratio / prev_ratio) <= threshold {
                consistent_ratio_count += 1;
                total_ratio_count += curr_ratio;

                break;
            }
        }

        // * Ensure no division by zero
        1.0 - total_ratio_count / f64::from(consistent_ratio_count + 1) * 0.8
    }

    fn evaluate_diff_of(
        hit_object: &TaikoDifficultyObject,
        diff_objects: &TaikoDifficultyObjects,
    ) -> f64 {
        let color_data = &hit_object.color_data;
        let mut difficulty = 0.0;

        if let Some(mono_streak) = color_data.mono_streak.as_ref().and_then(Weak::upgrade) {
            if let Some(first_hit_object) = mono_streak.get().first_hit_object() {
                if &*first_hit_object.get() == hit_object {
                    difficulty += Self::eval_mono_streak_diff(&mono_streak);
                }
            }
        }

        if let Some(alternating_mono_pattern) = color_data
            .alternating_mono_pattern
            .as_ref()
            .and_then(Weak::upgrade)
        {
            if let Some(first_hit_object) = alternating_mono_pattern.get().first_hit_object() {
                if &*first_hit_object.get() == hit_object {
                    difficulty +=
                        Self::eval_alternating_mono_pattern_diff(&alternating_mono_pattern);
                }
            }
        }

        if let Some(repeating_hit_patterns) = color_data.repeating_hit_patterns.as_ref() {
            if let Some(first_hit_object) = repeating_hit_patterns.get().first_hit_object() {
                if &*first_hit_object.get() == hit_object {
                    difficulty += Self::eval_repeating_hit_patterns_diff(repeating_hit_patterns);
                }
            }
        }

        let consistency_penalty =
            Self::consistent_ratio_penalty(hit_object, diff_objects, None, None);
        difficulty *= consistency_penalty;

        difficulty
    }

    fn eval_mono_streak_diff(mono_streak: &RefCount<MonoStreak>) -> f64 {
        let mono_streak = mono_streak.get();

        let parent_eval = mono_streak
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .as_ref()
            .map_or(1.0, Self::eval_alternating_mono_pattern_diff);

        logistic_exp(E * mono_streak.idx as f64 - 2.0 * E, None) * parent_eval * 0.5
    }

    fn eval_alternating_mono_pattern_diff(
        alternating_mono_pattern: &RefCount<AlternatingMonoPattern>,
    ) -> f64 {
        let alternating_mono_pattern = alternating_mono_pattern.get();

        let parent_eval = alternating_mono_pattern
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .as_ref()
            .map_or(1.0, Self::eval_repeating_hit_patterns_diff);

        logistic_exp(E * alternating_mono_pattern.idx as f64 - 2.0 * E, None) * parent_eval
    }

    fn eval_repeating_hit_patterns_diff(
        repeating_hit_patterns: &RefCount<RepeatingHitPatterns>,
    ) -> f64 {
        let repetition_interval = repeating_hit_patterns.get().repetition_interval as f64;

        2.0 * (1.0 - logistic_exp(E * repetition_interval - 2.0 * E, None))
    }
}
