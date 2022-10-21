use crate::taiko::difficulty_object::{ObjectLists, TaikoDifficultyObject};

use super::{Skill, StrainDecaySkill, StrainSkill};

#[derive(Clone, Debug)]
pub(crate) struct Stamina {
    curr_strain: f64,
    curr_section_peak: f64,
    curr_section_end: f64,
    pub(crate) strain_peaks: Vec<f64>,
}

impl Stamina {
    pub(crate) fn new() -> Self {
        Self {
            curr_strain: 0.0,
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            strain_peaks: Vec::new(),
        }
    }
}

impl Skill for Stamina {
    #[inline]
    fn process(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) {
        <Self as StrainSkill>::process(self, curr, hit_objects)
    }

    #[inline]
    fn difficulty_value(self) -> f64 {
        <Self as StrainSkill>::difficulty_value(self)
    }
}

impl StrainSkill for Stamina {
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

impl StrainDecaySkill for Stamina {
    const SKILL_MULTIPLIER: f64 = 1.1;
    const STRAIN_DECAY_BASE: f64 = 0.4;

    #[inline]
    fn curr_strain(&self) -> f64 {
        self.curr_strain
    }

    #[inline]
    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.curr_strain
    }

    #[inline]
    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) -> f64 {
        StaminaEvaluator::evaluate_diff_of(curr, hit_objects)
    }
}

struct StaminaEvaluator;

impl StaminaEvaluator {
    fn speed_bonus(mut interval: f64) -> f64 {
        // * Cap to 600bpm 1/4, 25ms note interval, 50ms key interval
        // * Interval will be capped at a very small value to avoid infinite/negative speed bonuses.
        // * TODO - This is a temporary measure as we need to implement methods of detecting playstyle-abuse of SpeedBonus.
        interval = interval.max(50.0);

        30.0 / interval
    }

    fn evaluate_diff_of(hit_object: &TaikoDifficultyObject, hit_objects: &ObjectLists) -> f64 {
        if !hit_object.base.is_hit {
            return 0.0;
        }

        // * Find the previous hit object hit by the current key, which is two notes of the same colour prior.
        let curr = hit_object;
        let key_prev = hit_objects.prev_mono(curr.idx, 1);

        if let Some(key_prev) = key_prev {
            // * Add a base strain to all objects
            0.5 + Self::speed_bonus(curr.start_time - key_prev.borrow().start_time)
        } else {
            // * There is no previous hit object hit by the current key
            0.0
        }
    }
}
