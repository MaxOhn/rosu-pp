use std::{cmp::Ordering, mem};

use crate::{
    osu::{difficulty_object::OsuDifficultyObject, SECTION_LEN},
    util::CompactVec,
};

pub(crate) trait Skill {
    fn process(&mut self, curr: &OsuDifficultyObject<'_>, diff_objects: &[OsuDifficultyObject<'_>]);
    fn difficulty_value(&mut self) -> f64;
}

pub(crate) trait StrainSkill: Skill + Sized {
    const DECAY_WEIGHT: f64 = 0.9;

    fn strain_peaks_mut(&mut self) -> &mut CompactVec;
    fn curr_section_peak(&mut self) -> &mut f64;
    fn curr_section_end(&mut self) -> &mut f64;

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64;

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64;

    fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        // * The first object doesn't generate a strain, so we begin with an incremented section end
        if curr.idx == 0 {
            let section_len = SECTION_LEN;
            *self.curr_section_end() = (curr.start_time / section_len).ceil() * section_len;
        }

        while curr.start_time > *self.curr_section_end() {
            self.save_curr_peak();

            {
                let section_end = *self.curr_section_end();
                self.start_new_section_from(section_end, curr, diff_objects);
            }

            *self.curr_section_end() += SECTION_LEN;

            // Optimization to finish the loop early if
            // the current peak is 0.0 i.e. it can't decay further.
            // If final values don't coincide perfectly anymore,
            // this should be looked at and maybe adjusted.
            if self.curr_section_peak().abs() <= f64::EPSILON
                && curr.start_time > *self.curr_section_end()
            {
                let remaining_time = curr.start_time - *self.curr_section_end();
                let remaining_iters = (remaining_time / SECTION_LEN).ceil();
                *self.curr_section_end() += remaining_iters * SECTION_LEN;
            }
        }

        *self.curr_section_peak() = self
            .strain_value_at(curr, diff_objects)
            .max(*self.curr_section_peak());
    }

    #[inline]
    fn save_curr_peak(&mut self) {
        let peak = *self.curr_section_peak();
        self.strain_peaks_mut().push(peak);
    }

    #[inline]
    fn start_new_section_from(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        // * The maximum strain of the new section is not zero by default
        // * This means we need to capture the strain level at the beginning of the new section,
        // * and use that as the initial peak level.
        *self.curr_section_peak() = self.calculate_initial_strain(time, curr, diff_objects);
    }

    fn difficulty_value(&mut self) -> f64;

    #[inline]
    fn get_curr_strain_peaks(&mut self) -> CompactVec {
        let curr_peak = *self.curr_section_peak();
        let mut strain_peaks = mem::take(self.strain_peaks_mut());
        strain_peaks.push(curr_peak);

        strain_peaks
    }
}

pub(crate) trait OsuStrainSkill: StrainSkill + Sized {
    const REDUCED_SECTION_COUNT: usize = 10;
    const REDUCED_STRAIN_BASELINE: f64 = 0.75;
    const DIFFICULTY_MULTIPLER: f64 = 1.06;

    fn difficulty_value(&mut self) -> f64 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
        // * These sections will not contribute to the difficulty.
        let mut peaks = self.get_curr_strain_peaks();
        peaks.retain(|peak| peak > 0.0);

        let mut peaks = peaks.to_vec();
        peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        let peak_iter = peaks.iter_mut().take(Self::REDUCED_SECTION_COUNT);

        fn lerp(start: f64, end: f64, amount: f64) -> f64 {
            start + (end - start) * amount
        }

        // * We are reducing the highest strains first to account for extreme difficulty spikes
        for (i, strain) in peak_iter.enumerate() {
            let clamped = (i as f32 / Self::REDUCED_SECTION_COUNT as f32).clamp(0.0, 1.0) as f64;
            let scale = (lerp(1.0, 10.0, clamped)).log10();
            *strain *= lerp(Self::REDUCED_STRAIN_BASELINE, 1.0, scale);
        }

        peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        // * Difficulty is the weighted sum of the highest strains from every section.
        // * We're sorting from highest to lowest strain.
        for strain in peaks {
            difficulty += strain * weight;
            weight *= Self::DECAY_WEIGHT;
        }

        difficulty * Self::DIFFICULTY_MULTIPLER
    }
}
