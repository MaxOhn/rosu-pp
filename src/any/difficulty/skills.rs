use crate::util::{float_ext::FloatExt, strains_vec::StrainsVec};

pub fn strain_decay(ms: f64, strain_decay_base: f64) -> f64 {
    strain_decay_base.powf(ms / 1000.0)
}

/// Wrapper around a difficulty skill that carries a list of all difficulty
/// objects.
pub struct Skill<'a, S: ISkill> {
    pub inner: &'a mut S,
    pub diff_objects: &'a S::DifficultyObjects<'a>,
}

impl<'a, S: ISkill> Skill<'a, S> {
    pub fn new(skill: &'a mut S, diff_objects: &'a S::DifficultyObjects<'a>) -> Self {
        Self {
            inner: skill,
            diff_objects,
        }
    }
}

/// Trait required for [`Skill`].
pub trait ISkill {
    type DifficultyObjects<'a>: ?Sized;
}

#[derive(Clone)]
pub struct StrainSkill {
    pub curr_section_peak: f64,
    pub curr_section_end: f64,
    pub strain_peaks: StrainsVec,
    // * Store individual strains
    // Note: osu!lazer pushes the current strain on each `StrainSkill.process`
    // call for all implementors of `StrainSkill` which is overkill for us
    // considering `object_strains` is only relevant for osu!standard's aim and
    // speed skills so we only use it for those. This does warrant greater care
    // in case of future changes.
    pub object_strains: Vec<f64>, // TODO: should this use `StrainsVec`?
}

impl Default for StrainSkill {
    fn default() -> Self {
        Self {
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            // mean=386.81 | median=279
            strain_peaks: StrainsVec::with_capacity(256),
            // averages not checked but 256 should be decent
            object_strains: Vec::with_capacity(256),
        }
    }
}

impl StrainSkill {
    pub const DECAY_WEIGHT: f64 = 0.9;
    pub const SECTION_LEN: f64 = 400.0;

    pub fn save_curr_peak(&mut self) {
        self.strain_peaks.push(self.curr_section_peak);
    }

    pub fn start_new_section_from(&mut self, initial_strain: f64) {
        self.curr_section_peak = initial_strain;
    }

    pub fn get_curr_strain_peaks(self) -> UsedStrainSkills<StrainsVec> {
        let mut strain_peaks = self.strain_peaks;
        strain_peaks.push(self.curr_section_peak);

        UsedStrainSkills {
            value: strain_peaks,
            object_strains: self.object_strains,
        }
    }

    pub fn difficulty_value(self, decay_weight: f64) -> UsedStrainSkills<DifficultyValue> {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        let UsedStrainSkills {
            value: mut peaks,
            object_strains,
        } = self.get_curr_strain_peaks();

        for strain in peaks.sorted_non_zero_iter() {
            difficulty += strain * weight;
            weight *= decay_weight;
        }

        UsedStrainSkills {
            value: DifficultyValue(difficulty),
            object_strains,
        }
    }
}

/// The method [`StrainSkills::get_curr_strain_peaks`] requires ownership of
/// [`StrainSkill`] as to not clone the inner strains. That means the field
/// `StrainSkills::object_strains` won't be accessible anymore.
///
/// To handle this, we make [`StrainSkills::get_curr_strain_peaks`] return
/// a [`UsedStrainSkills`] which contains the processed value, as well as the
/// object strains.
///
/// The same goes for [`StrainSkills::difficulty_value`].
pub struct UsedStrainSkills<T> {
    pub value: T,
    pub object_strains: Vec<f64>,
}

pub struct DifficultyValue(pub f64);

impl UsedStrainSkills<DifficultyValue> {
    pub const fn difficulty_value(&self) -> f64 {
        self.value.0
    }

    pub fn count_top_weighted_strains(&self) -> f64 {
        Self::static_count_top_weighted_strains(&self.object_strains, self.value.0)
    }

    pub fn static_count_top_weighted_strains(object_strains: &[f64], difficulty_value: f64) -> f64 {
        if object_strains.is_empty() {
            return 0.0;
        }

        // * What would the top strain be if all strain values were identical
        let consistent_top_strain = difficulty_value / 10.0;

        if consistent_top_strain.eq(0.0) {
            return object_strains.len() as f64;
        }

        // * Use a weighted sum of all strains. Constants are arbitrary and give nice values.
        object_strains
            .iter()
            .map(|s| 1.1 / (1.0 + f64::exp(-10.0 * (s / consistent_top_strain - 0.88))))
            .sum()
    }
}

impl UsedStrainSkills<StrainsVec> {
    pub fn into_strains(self) -> StrainsVec {
        self.value
    }
}

#[derive(Clone, Default)]
pub struct StrainDecaySkill {
    pub inner: StrainSkill,
    pub curr_strain: f64,
}

impl StrainDecaySkill {
    pub const DECAY_WEIGHT: f64 = StrainSkill::DECAY_WEIGHT;
    pub const SECTION_LEN: f64 = StrainSkill::SECTION_LEN;

    pub fn save_curr_peak(&mut self) {
        self.inner.save_curr_peak();
    }

    pub fn start_new_section_from(&mut self, initial_strain: f64) {
        self.inner.start_new_section_from(initial_strain);
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks().value
    }

    pub fn difficulty_value(self, decay_weight: f64) -> UsedStrainSkills<DifficultyValue> {
        self.inner.difficulty_value(decay_weight)
    }
}
