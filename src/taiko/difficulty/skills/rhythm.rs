use std::cmp;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainDecaySkill},
    },
    taiko::{
        difficulty::{
            object::{TaikoDifficultyObject, TaikoDifficultyObjects},
            rhythm::HitObjectRhythm,
        },
        object::HitType,
    },
    util::{compact_zeros::CompactZerosVec, float_ext::FloatExt, limited_queue::LimitedQueue},
};

const SKILL_MULTIPLIER: f64 = 10.0;
const STRAIN_DECAY_BASE: f64 = 0.0;

const STRAIN_DECAY: f64 = 0.96;
const RHYTHM_HISTORY_MAX_LEN: usize = 8;

#[allow(clippy::struct_field_names)]
#[derive(Clone, Default)]
pub struct Rhythm {
    inner: StrainDecaySkill,
    rhythm_history: LimitedQueue<RhythmHistoryElement, RHYTHM_HISTORY_MAX_LEN>,
    curr_strain: f64,
    notes_since_rhythm_change: usize,
}

impl Rhythm {
    fn repetition_penalties(&mut self, hit_object: &TaikoDifficultyObject) -> f64 {
        let mut penalty = 1.0;

        self.rhythm_history
            .push(RhythmHistoryElement::new(hit_object));

        for most_recent_patterns_to_compare in
            2..=cmp::min(RHYTHM_HISTORY_MAX_LEN / 2, self.rhythm_history.len())
        {
            for start in (0..self.rhythm_history.len() - most_recent_patterns_to_compare).rev() {
                if !self.same_pattern(start, most_recent_patterns_to_compare) {
                    continue;
                }

                let notes_since = hit_object.idx - self.rhythm_history[start].idx;
                penalty *= Self::repetition_penalty(notes_since);

                break;
            }
        }

        penalty
    }

    fn same_pattern(&self, start: usize, most_recent_patterns_to_compare: usize) -> bool {
        let start = self.rhythm_history.iter().skip(start);

        let most_recent_patterns_to_compare = self
            .rhythm_history
            .iter()
            .skip(self.rhythm_history.len() - most_recent_patterns_to_compare);

        start
            .zip(most_recent_patterns_to_compare)
            .all(|(a, b)| a.rhythm == b.rhythm)
    }

    fn repetition_penalty(notes_since: usize) -> f64 {
        (0.032 * notes_since as f64).min(1.0)
    }

    fn pattern_len_penalty(pattern_len: usize) -> f64 {
        let pattern_len = pattern_len as f64;
        let short_pattern_penalty = (0.15 * pattern_len).min(1.0);
        let long_pattern_penalty = (2.5 - 0.15 * pattern_len).clamp(0.0, 1.0);

        short_pattern_penalty.min(long_pattern_penalty)
    }

    fn speed_penalty(&mut self, delta: f64) -> f64 {
        if delta < 80.0 {
            return 1.0;
        } else if delta < 210.0 {
            return (1.4 - 0.005 * delta).max(0.0);
        }

        self.reset_rhythm_and_strain();

        0.0
    }

    fn reset_rhythm_and_strain(&mut self) {
        self.curr_strain = 0.0;
        self.notes_since_rhythm_change = 0;
    }

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        // * drum rolls and swells are exempt.
        if matches!(curr.base_hit_type, HitType::NonHit) {
            self.reset_rhythm_and_strain();

            return 0.0;
        }

        self.curr_strain *= STRAIN_DECAY;
        self.notes_since_rhythm_change += 1;

        // * rhythm difficulty zero (due to rhythm not changing) => no rhythm strain.
        if curr.rhythm.difficulty.eq(0.0) {
            return 0.0;
        }

        let mut obj_strain = curr.rhythm.difficulty;

        obj_strain *= self.repetition_penalties(curr);
        obj_strain *= Self::pattern_len_penalty(self.notes_since_rhythm_change);
        obj_strain *= self.speed_penalty(curr.delta_time);

        // * careful - needs to be done here since calls above read this value
        self.notes_since_rhythm_change = 0;

        self.curr_strain += obj_strain;

        self.curr_strain
    }

    const fn curr_strain(&self) -> f64 {
        self.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.curr_strain
    }

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.curr_strain()
    }

    pub fn get_curr_strain_peaks(self) -> CompactZerosVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn as_difficulty_value(&self) -> f64 {
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

        let strain_value_at = self.inner.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }
}

#[derive(Copy, Clone)]
struct RhythmHistoryElement {
    idx: usize,
    rhythm: &'static HitObjectRhythm,
}

impl RhythmHistoryElement {
    const fn new(difficulty_object: &TaikoDifficultyObject) -> Self {
        Self {
            idx: difficulty_object.idx,
            rhythm: difficulty_object.rhythm,
        }
    }
}

impl Default for RhythmHistoryElement {
    fn default() -> Self {
        Self {
            idx: 0,
            rhythm: HitObjectRhythm::static_ref(),
        }
    }
}
