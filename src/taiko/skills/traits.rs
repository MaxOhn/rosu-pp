use std::{cmp::Ordering, mem};

use crate::taiko::{
    difficulty_object::{ObjectLists, TaikoDifficultyObject},
    SECTION_LEN,
};

pub(crate) trait Skill: Sized {
    fn process(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists);
    fn difficulty_value(self) -> f64;
}

pub(crate) trait StrainSkill: Skill {
    const DECAY_WEIGHT: f64 = 0.9;

    fn strain_peaks_mut(&mut self) -> &mut Vec<f64>;
    fn curr_section_peak(&mut self) -> &mut f64;
    fn curr_section_end(&mut self) -> &mut f64;

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) -> f64;

    fn calculate_initial_strain(&self, time: f64, curr: &TaikoDifficultyObject) -> f64;

    fn process(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) {
        // * The first object doesn't generate a strain, so we begin with an incremented section end
        if curr.idx == 0 {
            let section_len = SECTION_LEN as f64;
            *self.curr_section_end() = (curr.start_time / section_len).ceil() * section_len;
        }

        while curr.start_time > *self.curr_section_end() {
            self.save_curr_peak();

            {
                let section_end = *self.curr_section_end();
                self.start_new_section_from(section_end, curr);
            }

            *self.curr_section_end() += SECTION_LEN as f64;
        }

        *self.curr_section_peak() = self
            .strain_value_at(curr, hit_objects)
            .max(*self.curr_section_peak());
    }

    #[inline]
    fn save_curr_peak(&mut self) {
        let peak = *self.curr_section_peak();
        self.strain_peaks_mut().push(peak);
    }

    #[inline]
    fn start_new_section_from(&mut self, time: f64, curr: &TaikoDifficultyObject) {
        // * The maximum strain of the new section is not zero by default
        // * This means we need to capture the strain level at the beginning of the new section,
        // * and use that as the initial peak level.
        *self.curr_section_peak() = self.calculate_initial_strain(time, curr);
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

    #[inline]
    fn get_curr_strain_peaks(mut self) -> Vec<f64> {
        let curr_peak = *self.curr_section_peak();
        let mut strain_peaks = mem::take(self.strain_peaks_mut());
        strain_peaks.push(curr_peak);

        strain_peaks
    }
}

pub(crate) trait StrainDecaySkill: StrainSkill {
    const SKILL_MULTIPLIER: f64;
    const STRAIN_DECAY_BASE: f64;

    fn curr_strain(&self) -> f64;
    fn curr_strain_mut(&mut self) -> &mut f64;

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) -> f64;

    #[inline]
    fn calculate_initial_strain(&self, time: f64, curr: &TaikoDifficultyObject) -> f64 {
        self.curr_strain() * self.strain_decay(time - curr.prev_time)
    }

    #[inline]
    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) -> f64 {
        *self.curr_strain_mut() *= self.strain_decay(curr.delta);
        *self.curr_strain_mut() += self.strain_value_of(curr, hit_objects) * Self::SKILL_MULTIPLIER;

        self.curr_strain()
    }

    #[inline]
    fn strain_decay(&self, ms: f64) -> f64 {
        Self::STRAIN_DECAY_BASE.powf(ms / 1000.0)
    }
}
