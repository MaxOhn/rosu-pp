use crate::{any::difficulty::skills::StrainSkill, util::strains_vec::StrainsVec};

#[derive(Clone, Default)]
pub struct OsuStrainSkill {
    pub inner: StrainSkill,
}

impl OsuStrainSkill {
    pub const REDUCED_SECTION_COUNT: usize = 10;
    pub const REDUCED_STRAIN_BASELINE: f64 = 0.75;

    pub const DECAY_WEIGHT: f64 = 0.9;
    pub const SECTION_LEN: f64 = 400.0;

    pub fn save_curr_peak(&mut self) {
        self.inner.save_curr_peak();
    }

    pub fn start_new_section_from(&mut self, initial_strain: f64) {
        self.inner.start_new_section_from(initial_strain);
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn difficulty_value(
        self,
        reduced_section_count: usize,
        reduced_strain_baseline: f64,
        decay_weight: f64,
    ) -> f64 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        let mut peaks = self.get_curr_strain_peaks();

        let peaks_iter = peaks.sorted_non_zero_iter_mut().take(reduced_section_count);

        for (i, strain) in peaks_iter.enumerate() {
            let clamped = f64::from((i as f32 / reduced_section_count as f32).clamp(0.0, 1.0));
            let scale = (lerp(1.0, 10.0, clamped)).log10();
            *strain *= lerp(reduced_strain_baseline, 1.0, scale);
        }

        peaks.sort_desc();

        for strain in peaks.iter() {
            difficulty += strain * weight;
            weight *= decay_weight;
        }

        difficulty
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        (5.0 * (difficulty / 0.0675).max(1.0) - 4.0).powf(3.0) / 100_000.0
    }
}

fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}
