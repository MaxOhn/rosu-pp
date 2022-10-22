use std::{cmp::Ordering, mem};

use crate::mania::{difficulty_object::ManiaDifficultyObject, SECTION_LEN};

pub(crate) trait Skill {
    fn process(&mut self, curr: &ManiaDifficultyObject, diff_objects: &[ManiaDifficultyObject]);
    fn difficulty_value(self) -> f64;
}

pub(crate) trait StrainSkill: Sized + Skill {
    const DECAY_WEIGHT: f64 = 0.9;

    fn curr_section_end(&self) -> f64;
    fn curr_section_end_mut(&mut self) -> &mut f64;

    fn curr_section_peak(&self) -> f64;
    fn curr_section_peak_mut(&mut self) -> &mut f64;

    fn strain_peaks_mut(&mut self) -> &mut Vec<f64>;

    fn strain_value_at(&mut self, curr: &ManiaDifficultyObject) -> f64;

    fn process(&mut self, curr: &ManiaDifficultyObject, diff_objects: &[ManiaDifficultyObject]) {
        // * The first object doesn't generate a strain, so we begin with an incremented section end
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / SECTION_LEN).ceil() * SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.save_curr_peak();
            self.start_new_section_from(self.curr_section_end(), curr, diff_objects);
            *self.curr_section_end_mut() += SECTION_LEN;
        }

        *self.curr_section_peak_mut() = self.strain_value_at(curr).max(self.curr_section_peak());
    }

    fn save_curr_peak(&mut self) {
        let curr_section_peak = self.curr_section_peak();
        self.strain_peaks_mut().push(curr_section_peak);
    }

    fn start_new_section_from(
        &mut self,
        time: f64,
        curr: &ManiaDifficultyObject,
        diff_objects: &[ManiaDifficultyObject],
    ) {
        *self.curr_section_peak_mut() = self.calculate_initial_strain(time, curr, diff_objects);
    }

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &ManiaDifficultyObject,
        diff_objects: &[ManiaDifficultyObject],
    ) -> f64;

    fn get_curr_strain_peaks(mut self) -> Vec<f64> {
        let mut peaks = mem::take(self.strain_peaks_mut());
        peaks.push(self.curr_section_peak());

        peaks
    }

    fn difficulty_value(self) -> f64 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
        // * These sections will not contribute to the difficulty.
        let mut peaks = self.get_curr_strain_peaks();
        peaks.retain(|&peak| peak > 0.0);
        peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        // * Difficulty is the weighted sum of the highest strains from every section.
        // * We're sorting from highest to lowest strain.
        for strain in peaks {
            difficulty += strain * weight;
            weight *= Self::DECAY_WEIGHT;
        }

        difficulty
    }
}

pub(crate) trait StrainDecaySkill: StrainSkill {
    const SKILL_MULTIPLIER: f64;
    const STRAIN_DECAY_BASE: f64;

    fn curr_strain(&self) -> f64;
    fn curr_strain_mut(&mut self) -> &mut f64;

    fn strain_value_of(&mut self, curr: &ManiaDifficultyObject) -> f64;

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &ManiaDifficultyObject,
        diff_objects: &[ManiaDifficultyObject],
    ) -> f64;

    fn strain_value_at(&mut self, curr: &ManiaDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= self.strain_decay(curr.delta_time);
        *self.curr_strain_mut() += self.strain_value_of(curr) * Self::SKILL_MULTIPLIER;

        self.curr_strain()
    }

    fn strain_decay(&self, ms: f64) -> f64 {
        Self::STRAIN_DECAY_BASE.powf(ms / 1000.0)
    }
}
