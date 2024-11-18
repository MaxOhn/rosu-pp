use crate::{any::difficulty::skills::StrainSkill, util::strains_vec::StrainsVec};

#[derive(Clone)]
pub struct OsuStrainSkill {
    pub object_strains: Vec<f64>,
    pub inner: StrainSkill,
}

impl Default for OsuStrainSkill {
    fn default() -> Self {
        Self {
            // mean=406.72 | median=307
            object_strains: Vec::with_capacity(256),
            inner: StrainSkill::default(),
        }
    }
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

    pub fn get_curr_strain_peaks(self) -> UsedOsuStrainSkills<StrainsVec> {
        UsedOsuStrainSkills {
            value: self.inner.get_curr_strain_peaks(),
            object_strains: self.object_strains,
        }
    }

    pub fn difficulty_value(
        self,
        reduced_section_count: usize,
        reduced_strain_baseline: f64,
        decay_weight: f64,
    ) -> UsedOsuStrainSkills<DifficultyValue> {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        let UsedOsuStrainSkills {
            value: mut peaks,
            object_strains,
        } = self.get_curr_strain_peaks();

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

        UsedOsuStrainSkills {
            value: DifficultyValue(difficulty),
            object_strains,
        }
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        (5.0 * (difficulty / 0.0675).max(1.0) - 4.0).powf(3.0) / 100_000.0
    }
}

fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}

pub struct DifficultyValue(f64);

pub struct UsedOsuStrainSkills<T> {
    value: T,
    object_strains: Vec<f64>,
}

impl UsedOsuStrainSkills<DifficultyValue> {
    pub const fn difficulty_value(&self) -> f64 {
        self.value.0
    }

    pub fn count_difficult_strains(&self) -> f64 {
        let DifficultyValue(diff) = self.value;

        if diff.abs() < f64::EPSILON {
            return 0.0;
        }

        // * What would the top strain be if all strain values were identical
        let consistent_top_strain = diff / 10.0;

        // * Use a weighted sum of all strains. Constants are arbitrary and give nice values
        self.object_strains
            .iter()
            .map(|s| 1.1 / (1.0 + (-10.0 * (s / consistent_top_strain - 0.88)).exp()))
            .sum()
    }
}

impl UsedOsuStrainSkills<StrainsVec> {
    pub fn strains(self) -> StrainsVec {
        self.value
    }
}
