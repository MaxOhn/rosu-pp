use super::{DifficultyObject, SkillKind};

use std::cmp::Ordering;

const DECAY_WEIGHT: f32 = 0.9;

const COLOR_SKILL_MULTIPLIER: f32 = 1.0;
const COLOR_STRAIN_DECAY_BASE: f32 = 0.4;

const RHYTHM_SKILL_MULTIPLIER: f32 = 10.0;
const RHYTHM_STRAIN_DECAY_BASE: f32 = 0.0;

const STAMINA_SKILL_MULTIPLIER: f32 = 1.0;
const STAMINA_STRAIN_DECAY_BASE: f32 = 0.4;

pub(crate) struct Skill {
    pub current_strain: f32,
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
    pub(crate) fn process(&mut self, current: &DifficultyObject<'_>, cheese: &[bool]) {
        self.current_strain *= self.strain_decay(current.delta);
        self.current_strain += self.kind.strain_value_of(current, cheese) * self.skill_multiplier();
        self.current_section_peak = self.current_section_peak.max(self.current_strain);
        self.prev_time.replace(current.start_time);
    }

    #[inline]
    pub(crate) fn difficulty_value(&self, buf: &mut [f32]) -> f32 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        buf.copy_from_slice(&self.strain_peaks);
        buf.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in buf.iter() {
            difficulty += strain * weight;
            weight *= DECAY_WEIGHT;
        }

        difficulty
    }

    #[inline]
    fn skill_multiplier(&self) -> f32 {
        match self.kind {
            SkillKind::Color { .. } => COLOR_SKILL_MULTIPLIER,
            SkillKind::Rhythm { .. } => RHYTHM_SKILL_MULTIPLIER,
            SkillKind::Stamina { .. } => STAMINA_SKILL_MULTIPLIER,
        }
    }

    #[inline]
    fn strain_decay_base(&self) -> f32 {
        match self.kind {
            SkillKind::Color { .. } => COLOR_STRAIN_DECAY_BASE,
            SkillKind::Rhythm { .. } => RHYTHM_STRAIN_DECAY_BASE,
            SkillKind::Stamina { .. } => STAMINA_STRAIN_DECAY_BASE,
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
