use crate::{
    taiko::difficulty_object::{HitObjectRhythm, ObjectLists, TaikoDifficultyObject},
    util::LimitedQueue,
};

use super::{Skill, StrainDecaySkill, StrainSkill};

const HISTORY_MAX_LEN: usize = 8;

#[derive(Clone, Debug)]
pub(crate) struct Rhythm {
    // Equal to osu's CurrentStrain from abstract class StrainDecaySkill
    curr_decay_strain: f64,
    // Equal to osu's currentStrain from class Rhythm
    curr_strain: f64,
    notes_since_rhythm_change: usize,
    history: LimitedQueue<HistoryElement, HISTORY_MAX_LEN>,
    curr_section_peak: f64,
    curr_section_end: f64,
    pub(crate) strain_peaks: Vec<f64>,
}

impl Rhythm {
    const STRAIN_DECAY: f64 = 0.96;

    pub(crate) fn new() -> Self {
        Self {
            curr_decay_strain: 0.0,
            curr_strain: 0.0,
            notes_since_rhythm_change: 0,
            history: LimitedQueue::new(),
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            strain_peaks: Vec::new(),
        }
    }

    fn reset_rhythm_and_strain(&mut self) {
        self.curr_strain = 0.0;
        self.notes_since_rhythm_change = 0;
    }

    fn repetition_penalties(&mut self, hit_object: &TaikoDifficultyObject) -> f64 {
        let mut penalty = 1.0;

        self.history.push(HistoryElement::new(hit_object));

        for most_recent_patterns_to_compare in 2..=(HISTORY_MAX_LEN / 2).min(self.history.len()) {
            for start in (0..self.history.len() - most_recent_patterns_to_compare).rev() {
                if !self.same_pattern(start, most_recent_patterns_to_compare) {
                    continue;
                }

                let notes_since = hit_object.idx - self.history[start].idx;
                penalty *= Self::repetition_penalty(notes_since);

                break;
            }
        }

        penalty
    }

    fn same_pattern(&self, start: usize, most_recent_patterns_to_compare: usize) -> bool {
        let start = self.history.iter().skip(start);

        let most_recent_patterns_to_compare = self
            .history
            .iter()
            .skip(self.history.len() - most_recent_patterns_to_compare);

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
}

impl Skill for Rhythm {
    #[inline]
    fn process(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) {
        <Self as StrainSkill>::process(self, curr, hit_objects)
    }

    #[inline]
    fn difficulty_value(self) -> f64 {
        <Self as StrainSkill>::difficulty_value(self)
    }
}

impl StrainSkill for Rhythm {
    #[inline]
    fn strain_peaks_mut(&mut self) -> &mut Vec<f64> {
        &mut self.strain_peaks
    }

    #[inline]
    fn curr_section_peak(&mut self) -> &mut f64 {
        &mut self.curr_section_peak
    }

    #[inline]
    fn curr_section_end(&mut self) -> &mut f64 {
        &mut self.curr_section_end
    }

    #[inline]
    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) -> f64 {
        <Self as StrainDecaySkill>::strain_value_at(self, curr, hit_objects)
    }

    #[inline]
    fn calculate_initial_strain(&self, time: f64, curr: &TaikoDifficultyObject) -> f64 {
        <Self as StrainDecaySkill>::calculate_initial_strain(self, time, curr)
    }
}

impl StrainDecaySkill for Rhythm {
    const SKILL_MULTIPLIER: f64 = 10.0;
    const STRAIN_DECAY_BASE: f64 = 0.0;

    #[inline]
    fn curr_strain(&self) -> f64 {
        self.curr_decay_strain
    }

    #[inline]
    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.curr_decay_strain
    }

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject, _: &ObjectLists) -> f64 {
        let base_is_circle = curr.base.is_hit;

        // * drum rolls and swells are exempt.
        if !base_is_circle {
            self.reset_rhythm_and_strain();

            return 0.0;
        }

        self.curr_strain *= Self::STRAIN_DECAY;
        self.notes_since_rhythm_change += 1;

        // * rhythm difficulty zero (due to rhythm not changing) => no rhythm strain.
        if curr.rhythm.difficulty.abs() <= f64::EPSILON {
            return 0.0;
        }

        let mut obj_strain = curr.rhythm.difficulty;

        obj_strain *= self.repetition_penalties(curr);
        obj_strain *= Self::pattern_len_penalty(self.notes_since_rhythm_change);
        obj_strain *= self.speed_penalty(curr.delta);

        // * careful - needs to be done here since calls above read this value
        self.notes_since_rhythm_change = 0;

        self.curr_strain += obj_strain;

        self.curr_strain
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct HistoryElement {
    idx: usize,
    rhythm: &'static HitObjectRhythm,
}

impl HistoryElement {
    fn new(difficulty_object: &TaikoDifficultyObject) -> Self {
        Self {
            idx: difficulty_object.idx,
            rhythm: difficulty_object.rhythm,
        }
    }
}

impl Default for HistoryElement {
    fn default() -> Self {
        Self {
            idx: 0,
            rhythm: HitObjectRhythm::static_ref(),
        }
    }
}
