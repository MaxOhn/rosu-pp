use super::{DifficultyObject, SkillKind};

use std::cmp::Ordering;

const SPEED_SKILL_MULTIPLIER: f32 = 1400.0;
const SPEED_STRAIN_DECAY_BASE: f32 = 0.3;

const AIM_SKILL_MULTIPLIER: f32 = 26.25;
const AIM_STRAIN_DECAY_BASE: f32 = 0.15;

const DECAY_WEIGHT: f32 = 0.9;

pub(crate) struct Skill {
    current_strain: f32,
    current_section_peak: f32,

    kind: SkillKind,
    pub(crate) strain_peaks: Vec<f32>,

    prev_time: Option<f32>,
    pub(crate) object_strains: Vec<f32>,
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
            object_strains: Vec::new(),
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
    pub(crate) fn process(&mut self, current: &DifficultyObject<'_>) {
        self.current_strain *= self.strain_decay(current.delta);
        self.current_strain += self.kind.strain_value_of(current) * self.skill_multiplier();

        self.object_strains.push(self.current_strain);

        self.current_section_peak = self.current_section_peak.max(self.current_strain);
        self.prev_time.replace(current.base.time);
    }

    pub(crate) fn difficulty_value(&mut self) -> f32 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        self.strain_peaks
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in self.strain_peaks.iter() {
            difficulty += strain * weight;
            weight *= DECAY_WEIGHT;
        }

        difficulty
    }

    pub(crate) fn count_difficult_strains(&mut self) -> f64 {
        let top_strain = self
            .object_strains
            .iter()
            .fold(f64::NEG_INFINITY, |prev, curr| prev.max(*curr as f64));

        self.object_strains
            .iter()
            .map(|strain| (strain / top_strain as f32).powi(4))
            .sum::<f32>() as f64
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
