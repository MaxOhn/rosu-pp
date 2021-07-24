use crate::math_util;

use super::{DifficultyObject, SkillKind};

use std::cmp::Ordering;

const SPEED_SKILL_MULTIPLIER: f32 = 1400.0;
const SPEED_STRAIN_DECAY_BASE: f32 = 0.3;
const REDUCED_STRAIN_BASELINE: f32 = 0.75;

const AIM_SKILL_MULTIPLIER: f32 = 26.25;
const AIM_STRAIN_DECAY_BASE: f32 = 0.15;

const DECAY_WEIGHT: f32 = 0.9;

pub(crate) struct Skill {
    current_strain: f32,
    current_section_peak: f32,

    kind: SkillKind,
    pub(crate) strain_peaks: Vec<f32>,

    prev_time: Option<f32>,
}

impl Skill {
    #[inline]
    pub(crate) fn new(kind: SkillKind) -> Self {
        Self {
            current_strain: 1.0,
            current_section_peak: 1.0,

            kind,
            strain_peaks: Vec::with_capacity(128),

            prev_time: None,
        }
    }

    #[inline]
    pub(crate) fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.current_section_peak);
    }

    #[inline]
    pub(crate) fn start_new_section_from(&mut self, time: f32) {
        self.current_section_peak = self.peak_strain(time - self.prev_time.unwrap());
    }

    #[inline]
    pub(crate) fn process(&mut self, current: &DifficultyObject) {
        self.current_strain *= self.strain_decay(current.delta);
        self.current_strain += self.kind.strain_value_of(&current) * self.skill_multiplier();
        self.current_section_peak = self.current_section_peak.max(self.current_strain);
        self.prev_time.replace(current.base.start_time);
    }

    pub(crate) fn difficulty_value(&mut self) -> f32 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        let (reduced_section_count, difficulty_multiplier) = self.kind.difficulty_values();
        let reduced_section_count_f32 = reduced_section_count as f32;

        self.strain_peaks
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for (i, strain) in self
            .strain_peaks
            .iter_mut()
            .take(reduced_section_count)
            .enumerate()
        {
            let clamped = (i as f32 / reduced_section_count_f32).clamp(0.0, 1.0);
            let scale = (math_util::lerp(1.0, 10.0, clamped)).log10();
            *strain *= math_util::lerp(REDUCED_STRAIN_BASELINE, 1.0, scale);
        }

        self.strain_peaks
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in self.strain_peaks.iter() {
            difficulty += strain * weight;
            weight *= DECAY_WEIGHT;
        }

        difficulty * difficulty_multiplier
    }

    #[inline]
    fn skill_multiplier(&self) -> f32 {
        match self.kind {
            SkillKind::Aim => AIM_SKILL_MULTIPLIER,
            SkillKind::Speed => SPEED_SKILL_MULTIPLIER,
        }
    }

    #[inline]
    fn strain_decay_base(&self) -> f32 {
        match self.kind {
            SkillKind::Aim => AIM_STRAIN_DECAY_BASE,
            SkillKind::Speed => SPEED_STRAIN_DECAY_BASE,
        }
    }

    #[inline]
    fn peak_strain(&self, delta_time: f32) -> f32 {
        self.current_strain * self.strain_decay(delta_time)
    }

    #[inline]
    fn strain_decay(&self, ms: f32) -> f32 {
        self.strain_decay_base().powf(ms / 1000.0)
    }
}
