use crate::math_util;

use super::{skill_kind::calculate_speed_rhythm_bonus, DifficultyObject, SkillKind};

use std::cmp::Ordering;

const REDUCED_STRAIN_BASELINE: f32 = 0.75;

pub(crate) struct Skill {
    curr_strain: f32,
    curr_section_peak: f32,

    kind: SkillKind,
    pub(crate) strain_peaks: Vec<f32>,

    prev_time: Option<f32>,
}

impl Skill {
    #[inline]
    pub(crate) fn new(kind: SkillKind) -> Self {
        Self {
            curr_strain: 1.0,
            curr_section_peak: 0.0,

            kind,
            strain_peaks: Vec::with_capacity(128),

            prev_time: None,
        }
    }

    pub(crate) fn process(&mut self, curr: &DifficultyObject) {
        self.kind.pre_process();
        self.curr_section_peak = self.strain_value_at(curr).max(self.curr_section_peak);
        self.prev_time = Some(curr.base.time);
        self.kind.post_process(curr);
    }

    #[inline]
    pub(crate) fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.curr_section_peak);
    }

    #[inline]
    pub(crate) fn start_new_section_from(&mut self, time: f32) {
        // The maximum strain of the new section is not zero by default
        self.curr_section_peak = self.calculate_initial_strain(time);
    }

    pub(crate) fn difficulty_value(&mut self) -> f32 {
        // TODO: Remove
        // for (i, strain) in self.strain_peaks.iter().enumerate() {
        //     println!("[{}] {}", i, strain);
        // }

        let mut difficulty = 0.0;
        let mut weight = 1.0;
        let decay_weight = self.kind.decay_weight();

        let (reduced_section_count, difficulty_multiplier) = self.kind.difficulty_values();
        let reduced_section_count_f32 = reduced_section_count as f32;

        self.strain_peaks
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        let peaks = self.strain_peaks.iter_mut();

        for (i, strain) in peaks.take(reduced_section_count).enumerate() {
            let clamped = (i as f32 / reduced_section_count_f32).clamp(0.0, 1.0);
            let scale = (math_util::lerp(1.0, 10.0, clamped)).log10();
            *strain *= math_util::lerp(REDUCED_STRAIN_BASELINE, 1.0, scale);
        }

        self.strain_peaks
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in &self.strain_peaks {
            difficulty += strain * weight;
            weight *= decay_weight;
        }

        difficulty * difficulty_multiplier
    }

    pub(crate) fn calculate_initial_strain(&self, time: f32) -> f32 {
        let prev_time = self.prev_time.unwrap_or(0.0);
        let decayed_strain = self.curr_strain * self.kind.strain_decay(time - prev_time);

        match &self.kind {
            SkillKind::Aim | SkillKind::Flashlight { .. } => decayed_strain,
            SkillKind::Speed { curr_rhythm, .. } => curr_rhythm * decayed_strain,
        }
    }

    pub(crate) fn strain_value_at(&mut self, curr: &DifficultyObject) -> f32 {
        self.curr_strain *= self.kind.strain_decay(curr.delta);
        self.curr_strain += self.kind.strain_value_of(curr) * self.kind.skill_multiplier();

        match &mut self.kind {
            SkillKind::Aim | SkillKind::Flashlight { .. } => self.curr_strain,
            SkillKind::Speed {
                curr_rhythm,
                history,
                hit_window,
            } => {
                *curr_rhythm = calculate_speed_rhythm_bonus(curr, history, *hit_window);

                self.curr_strain * *curr_rhythm
            }
        }
    }
}
