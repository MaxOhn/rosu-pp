use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::taiko::{
    colours::{AlternatingMonoPattern, MonoStreak, RepeatingHitPatterns},
    difficulty_object::{ObjectLists, TaikoDifficultyObject},
};

use super::{Skill, StrainDecaySkill, StrainSkill};

#[derive(Clone, Debug)]
pub(crate) struct Colour {
    curr_strain: f64,
    curr_section_peak: f64,
    curr_section_end: f64,
    pub(crate) strain_peaks: Vec<f64>,
}

impl Colour {
    pub(crate) fn new() -> Self {
        Self {
            curr_strain: 0.0,
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            strain_peaks: Vec::new(),
        }
    }
}

impl Skill for Colour {
    #[inline]
    fn process(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) {
        <Self as StrainSkill>::process(self, curr, hit_objects)
    }

    #[inline]
    fn difficulty_value(self) -> f64 {
        <Self as StrainSkill>::difficulty_value(self)
    }
}

impl StrainSkill for Colour {
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

impl StrainDecaySkill for Colour {
    const SKILL_MULTIPLIER: f64 = 0.12;
    const STRAIN_DECAY_BASE: f64 = 0.8;

    #[inline]
    fn curr_strain(&self) -> f64 {
        self.curr_strain
    }

    #[inline]
    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.curr_strain
    }

    #[inline]
    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject, _: &ObjectLists) -> f64 {
        ColourEvaluator::evaluate_diff_of(curr)
    }
}

struct ColourEvaluator;

impl ColourEvaluator {
    fn sigmoid(val: f64, center: f64, width: f64, middle: f64, height: f64) -> f64 {
        let sigmoid = (std::f64::consts::E * -(val - center) / width).tanh();

        sigmoid * (height / 2.0) + middle
    }

    fn evaluate_diff_of_mono_streak(mono_streak: Rc<RefCell<MonoStreak>>) -> f64 {
        let mono_streak = mono_streak.borrow();

        let parent_eval = mono_streak
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .map_or(1.0, Self::evaluate_diff_of_alternating_mono_pattern);

        Self::sigmoid(mono_streak.idx as f64, 2.0, 2.0, 0.5, 1.0) * parent_eval * 0.5
    }

    fn evaluate_diff_of_alternating_mono_pattern(
        alternating_mono_pattern: Rc<RefCell<AlternatingMonoPattern>>,
    ) -> f64 {
        let alternating_mono_pattern = alternating_mono_pattern.borrow();

        let parent_eval = alternating_mono_pattern
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .map_or(1.0, Self::evaluate_diff_of_repeating_hit_patterns);

        Self::sigmoid(alternating_mono_pattern.idx as f64, 2.0, 2.0, 0.5, 1.0) * parent_eval
    }

    fn evaluate_diff_of_repeating_hit_patterns(
        repeating_hit_patterns: Rc<RefCell<RepeatingHitPatterns>>,
    ) -> f64 {
        let repetition_interval = repeating_hit_patterns.borrow().repetition_interval as f64;

        2.0 * (1.0 - Self::sigmoid(repetition_interval, 2.0, 2.0, 0.5, 1.0))
    }

    fn evaluate_diff_of(hit_object: &TaikoDifficultyObject) -> f64 {
        let colour = &hit_object.colour;
        let mut difficulty = 0.0;

        // * Difficulty for MonoStreak
        if let Some(mono_streak) = colour.mono_streak.as_ref().and_then(Weak::upgrade) {
            difficulty += Self::evaluate_diff_of_mono_streak(mono_streak);
        }

        // * Difficulty for AlternatingMonoPattern
        if let Some(alternating_mono_pattern) = colour
            .alternating_mono_pattern
            .as_ref()
            .and_then(Weak::upgrade)
        {
            difficulty += Self::evaluate_diff_of_alternating_mono_pattern(alternating_mono_pattern);
        }

        // * Difficulty for RepeatingHitPattern
        if let Some(repeating_hit_patterns) = colour.repeating_hit_patterns.as_ref().map(Rc::clone)
        {
            difficulty += Self::evaluate_diff_of_repeating_hit_patterns(repeating_hit_patterns);
        }

        difficulty
    }
}
