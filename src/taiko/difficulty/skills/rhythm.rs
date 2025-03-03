use std::f64::consts::PI;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{
            strain_decay, DifficultyValue, ISkill, Skill, StrainDecaySkill, UsedStrainSkills,
        },
    },
    taiko::difficulty::{
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
        rhythm::data::same_rhythm_hit_object_grouping::SameRhythmHitObjectGrouping,
    },
    util::{
        difficulty::{bell_curve, logistic},
        strains_vec::StrainsVec,
        sync::RefCount,
    },
};

use super::stamina::StaminaEvaluator;

const SKILL_MULTIPLIER: f64 = 1.0;
const STRAIN_DECAY_BASE: f64 = 0.4;

#[derive(Clone)]
pub struct Rhythm {
    inner: StrainDecaySkill,
    great_hit_window: f64,
}

impl Rhythm {
    pub fn new(great_hit_window: f64) -> Self {
        Self {
            inner: StrainDecaySkill::default(),
            great_hit_window,
        }
    }

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

impl ISkill for Rhythm {
    type DifficultyObjects<'a> = TaikoDifficultyObjects;
}

impl Skill<'_, Rhythm> {
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

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        let mut difficulty = RhythmEvaluator::evaluate_diff_of(curr, self.inner.great_hit_window);

        // * To prevent abuse of exceedingly long intervals between awkward rhythms, we penalise its difficulty.
        let stamina_difficulty = StaminaEvaluator::evaluate_diff_of(curr, self.diff_objects) - 0.5; // * Remove base strain
        difficulty *= logistic(stamina_difficulty, 1.0 / 15.0, 50.0, None);

        difficulty
    }

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.inner.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.inner.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.inner.curr_strain()
    }
}

struct RhythmEvaluator;

impl RhythmEvaluator {
    fn evaluate_diff_of(hit_object: &TaikoDifficultyObject, hit_window: f64) -> f64 {
        let rhythm_data = &hit_object.rhythm_data;
        let mut difficulty = 0.0;

        let mut same_rhythm = 0.0;
        let mut same_pattern = 0.0;
        let mut interval_penalty = 0.0;

        // * Difficulty for SameRhythmGroupedHitObjects
        if let Some(ref same_rhythm_grouped) = rhythm_data.same_rhythm_grouped_hit_objects {
            if same_rhythm_grouped
                .get()
                .first_hit_object()
                .is_some_and(|h| &*h.get() == hit_object)
            {
                same_rhythm += 10.0 * Self::evaluate_diff_of_(same_rhythm_grouped, hit_window);
                interval_penalty =
                    Self::repeated_interval_penalty(same_rhythm_grouped, hit_window, None);
            }
        }

        // * Difficulty for SamePatternsGroupedHitObjects
        if let Some(ref same_pattern_grouped) = rhythm_data.same_patterns_grouped_hit_objects {
            if same_pattern_grouped
                .get()
                .first_hit_object()
                .is_some_and(|h| &*h.get() == hit_object)
            {
                if let Some(interval_ratio) = same_pattern_grouped.get().interval_ratio() {
                    same_pattern += 1.15 * Self::ratio_difficulty(interval_ratio, None);
                }
            }
        }

        difficulty += f64::max(same_rhythm, same_pattern) * interval_penalty;

        difficulty
    }

    fn evaluate_diff_of_(
        same_rhythm_grouped_hit_objects: &RefCount<SameRhythmHitObjectGrouping>,
        hit_window: f64,
    ) -> f64 {
        let mut interval_diff = Self::ratio_difficulty(
            same_rhythm_grouped_hit_objects
                .get()
                .hit_object_interval_ratio,
            None,
        );
        let prev_interval = same_rhythm_grouped_hit_objects
            .get()
            .upgraded_previous()
            .and_then(|h| h.get().hit_object_interval);

        interval_diff *=
            Self::repeated_interval_penalty(same_rhythm_grouped_hit_objects, hit_window, None);

        let borrowed = same_rhythm_grouped_hit_objects.get();
        let duration = borrowed.duration();

        // * If a previous interval exists and there are multiple hit objects in the sequence:
        if let Some(prev_interval) = prev_interval.filter(|_| borrowed.hit_objects.len() > 1) {
            if let Some(duration) = duration {
                let expected_duration_from_prev = prev_interval * borrowed.hit_objects.len() as f64;
                let duration_diff = duration - expected_duration_from_prev;

                if duration_diff > 0.0 {
                    interval_diff *= logistic(duration_diff / hit_window, 0.7, 1.0, Some(1.0));
                }
            }
        }

        // Penalise patterns that can be hit within a single hit window.
        if let Some(duration) = duration {
            interval_diff *= logistic(duration / hit_window, 0.6, 1.0, Some(1.0));
        }

        f64::powf(interval_diff, 0.75)
    }

    fn repeated_interval_penalty(
        same_rhythm_grouped_hit_objects: &RefCount<SameRhythmHitObjectGrouping>,
        hit_window: f64,
        threshold: Option<f64>,
    ) -> f64 {
        let threshold = threshold.unwrap_or(0.1);

        let same_interval =
            |start_object: RefCount<SameRhythmHitObjectGrouping>, interval_count: usize| -> f64 {
                let mut intervals = Vec::new();
                let mut curr_object = Some(start_object);

                let mut i = 0;

                while let Some(curr) = curr_object.filter(|_| i < interval_count) {
                    let curr = curr.get();

                    if let Some(interval) = curr.hit_object_interval {
                        intervals.push(interval);
                    }

                    curr_object = curr.upgraded_previous();
                    i += 1;
                }

                if intervals.len() < interval_count {
                    return 1.0; // * No penalty if there aren't enough valid intervals.
                }

                for i in 0..intervals.len() {
                    for j in i + 1..intervals.len() {
                        let ratio = intervals[i] / intervals[j];

                        // * If any two intervals are similar, apply a penalty.
                        if f64::abs(1.0 - ratio) <= threshold {
                            return 0.8;
                        }
                    }
                }

                // * No penalty if all intervals are different.
                1.0
            };

        let long_interval_penalty =
            same_interval(RefCount::clone(same_rhythm_grouped_hit_objects), 3);

        let short_interval_penalty = if same_rhythm_grouped_hit_objects.get().hit_objects.len() < 6
        {
            same_interval(RefCount::clone(same_rhythm_grouped_hit_objects), 4)
        } else {
            // * Returns a non-penalty if there are 6 or more notes within an interval.
            1.0
        };

        // * The duration penalty is based on hit object duration relative to hitWindow.
        let duration_penalty = same_rhythm_grouped_hit_objects
            .get()
            .duration()
            .map_or(0.5, |duration| {
                f64::max(1.0 - duration * 2.0 / hit_window, 0.5)
            });

        f64::min(long_interval_penalty, short_interval_penalty) * duration_penalty
    }

    fn ratio_difficulty(mut ratio: f64, terms: Option<i32>) -> f64 {
        let terms = terms.unwrap_or(8);
        let mut difficulty = 0.0;

        // * Validate the ratio by ensuring it is a normal number in cases where maps breach regular mapping conditions.
        ratio = if ratio.is_normal() { ratio } else { 0.0 };

        for i in 1..=terms {
            difficulty += Self::term_penalty(ratio, i, 4.0, 1.0);
        }

        difficulty += f64::from(terms) / (1.0 + ratio);

        // * Give bonus to near-1 ratios
        difficulty += bell_curve(ratio, 1.0, 0.5, None);

        // * Penalize ratios that are VERY near 1
        difficulty -= bell_curve(ratio, 1.0, 0.3, None);

        difficulty = f64::max(difficulty, 0.0);
        difficulty /= f64::sqrt(8.0);

        difficulty
    }

    fn term_penalty(ratio: f64, denominator: i32, power: f64, multiplier: f64) -> f64 {
        -multiplier * f64::powf(f64::cos(f64::from(denominator) * PI * ratio), power)
    }
}
