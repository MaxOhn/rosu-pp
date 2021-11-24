use super::{DifficultyObject, SkillKind};

use std::cmp::Ordering;

const DECAY_WEIGHT: f64 = 0.9;

const COLOR_SKILL_MULTIPLIER: f64 = 1.0;
const COLOR_STRAIN_DECAY_BASE: f64 = 0.4;

const RHYTHM_SKILL_MULTIPLIER: f64 = 10.0;
const RHYTHM_STRAIN_DECAY_BASE: f64 = 0.0;

const STAMINA_SKILL_MULTIPLIER: f64 = 1.0;
const STAMINA_STRAIN_DECAY_BASE: f64 = 0.4;

#[derive(Clone, Debug)]
pub(crate) struct Skills {
    pub(crate) color: Skill,
    pub(crate) rhythm: Skill,
    pub(crate) stamina_right: Skill,
    pub(crate) stamina_left: Skill,
}

impl Skills {
    pub(crate) fn new() -> Self {
        Self {
            color: Skill::new(SkillKind::color()),
            rhythm: Skill::new(SkillKind::rhythm()),
            stamina_right: Skill::new(SkillKind::stamina(true)),
            stamina_left: Skill::new(SkillKind::stamina(false)),
        }
    }

    pub(crate) fn save_peak_and_start_new_section(&mut self, time: f64) {
        self.color.save_current_peak();
        self.color.start_new_section_from(time);
        self.rhythm.save_current_peak();
        self.rhythm.start_new_section_from(time);
        self.stamina_right.save_current_peak();
        self.stamina_right.start_new_section_from(time);
        self.stamina_left.save_current_peak();
        self.stamina_left.start_new_section_from(time);
    }

    pub(crate) fn save_current_peak(&mut self) {
        self.color.save_current_peak();
        self.rhythm.save_current_peak();
        self.stamina_right.save_current_peak();
        self.stamina_left.save_current_peak();
    }

    pub(crate) fn process(&mut self, curr: &DifficultyObject<'_>, cheese: &[bool]) {
        self.color.process(curr, cheese);
        self.rhythm.process(curr, cheese);
        self.stamina_right.process(curr, cheese);
        self.stamina_left.process(curr, cheese);
    }

    pub(crate) fn strain_peaks_len(&self) -> usize {
        self.color.strain_peaks.len()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Skill {
    pub(crate) current_strain: f64,
    pub(crate) curr_section_peak: f64,

    kind: SkillKind,
    pub(crate) strain_peaks: Vec<f64>,

    prev_time: Option<f64>,
}

impl Skill {
    #[inline]
    pub(crate) fn new(kind: SkillKind) -> Self {
        Self {
            current_strain: 1.0,
            curr_section_peak: 1.0,

            kind,
            strain_peaks: Vec::with_capacity(128),

            prev_time: None,
        }
    }

    #[inline]
    pub(crate) fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.curr_section_peak);
    }

    #[inline]
    pub(crate) fn start_new_section_from(&mut self, time: f64) {
        self.curr_section_peak = self.peak_strain(time - self.prev_time.unwrap());
    }

    #[inline]
    pub(crate) fn process(&mut self, curr: &DifficultyObject<'_>, cheese: &[bool]) {
        self.current_strain *= self.strain_decay(curr.delta);
        self.current_strain += self.kind.strain_value_of(curr, cheese) * self.skill_multiplier();
        self.curr_section_peak = self.curr_section_peak.max(self.current_strain);
        self.prev_time.replace(curr.start_time);
    }

    pub(crate) fn copy_strain_peaks(&self, buf: &mut [f64]) {
        buf.copy_from_slice(&self.strain_peaks);
    }

    #[inline]
    pub(crate) fn difficulty_value(&self, peaks: &mut [f64]) -> f64 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in peaks.iter() {
            difficulty += strain * weight;
            weight *= DECAY_WEIGHT;
        }

        difficulty
    }

    #[inline]
    fn skill_multiplier(&self) -> f64 {
        match self.kind {
            SkillKind::Color { .. } => COLOR_SKILL_MULTIPLIER,
            SkillKind::Rhythm { .. } => RHYTHM_SKILL_MULTIPLIER,
            SkillKind::Stamina { .. } => STAMINA_SKILL_MULTIPLIER,
        }
    }

    #[inline]
    fn strain_decay_base(&self) -> f64 {
        match self.kind {
            SkillKind::Color { .. } => COLOR_STRAIN_DECAY_BASE,
            SkillKind::Rhythm { .. } => RHYTHM_STRAIN_DECAY_BASE,
            SkillKind::Stamina { .. } => STAMINA_STRAIN_DECAY_BASE,
        }
    }

    #[inline]
    fn peak_strain(&self, delta_time: f64) -> f64 {
        self.current_strain * self.strain_decay(delta_time)
    }

    #[inline]
    fn strain_decay(&self, ms: f64) -> f64 {
        self.strain_decay_base().powf(ms / 1000.0)
    }
}
