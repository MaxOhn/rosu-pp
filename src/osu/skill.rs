use super::{lerp, skill_kind::calculate_speed_rhythm_bonus, DifficultyObject, SkillKind};

use std::{cmp::Ordering, fmt};

const REDUCED_STRAIN_BASELINE: f64 = 0.75;

#[derive(Debug)]
pub(crate) struct Skills {
    skills: Box<[Skill]>,
    mask: u8,
}

impl Skills {
    const RX: u8 = 1 << 0;
    const FL: u8 = 1 << 1;

    pub(crate) fn new(hit_window: f64, rx: bool, radius: f32, fl: bool) -> Self {
        let mut skills = Vec::with_capacity(2 + !rx as usize + fl as usize);

        skills.push(Skill::aim(true));
        skills.push(Skill::aim(false));

        if !rx {
            skills.push(Skill::speed(hit_window));
        }

        if fl {
            // NOTE: Instead of having `NORMALIZED_RADIUS` as dividend, it still uses 52.0.
            let scaling_factor = 52.0 / radius as f64;
            skills.push(Skill::flashlight(scaling_factor));
        }

        let mask = rx as u8 * Self::RX + fl as u8 * Self::FL;
        let skills = skills.into_boxed_slice();

        Self { skills, mask }
    }

    pub(crate) fn start_new_section_from(&mut self, curr_section_end: f64) {
        for skill in self.skills.iter_mut() {
            skill.start_new_section_from(curr_section_end);
        }
    }

    pub(crate) fn save_peak_and_start_new_section(&mut self, curr_section_end: f64) {
        for skill in self.skills.iter_mut() {
            skill.save_current_peak();
            skill.start_new_section_from(curr_section_end);
        }
    }

    pub(crate) fn save_current_peak(&mut self) {
        for skill in self.skills.iter_mut() {
            skill.save_current_peak();
        }
    }

    pub(crate) fn process(&mut self, h: &DifficultyObject<'_>) {
        for skill in self.skills.iter_mut() {
            skill.process(h);
        }
    }

    pub(crate) fn aim(&mut self) -> &mut Skill {
        &mut self.skills[0]
    }

    pub(crate) fn aim_no_sliders(&mut self) -> &mut Skill {
        &mut self.skills[1]
    }

    pub(crate) fn speed_flashlight(&mut self) -> (Option<&mut Skill>, Option<&mut Skill>) {
        match (self.mask & Self::RX, self.mask & Self::FL) {
            // only speed
            (0, 0) => (Some(&mut self.skills[2]), None),
            // both speed and flashlight
            (0, _) => {
                let (left, right) = self.skills.split_at_mut(3);

                (Some(&mut left[2]), Some(&mut right[0]))
            }
            // neither
            (_, 0) => (None, None),
            // only flashlight
            (_, _) => (None, Some(&mut self.skills[2])),
        }
    }
}

pub(crate) struct Skill {
    curr_strain: f64,
    curr_section_peak: f64,

    kind: SkillKind,
    pub(crate) strain_peaks: Vec<f64>,

    prev_time: Option<f64>,
}

impl Skill {
    #[inline]
    pub(crate) fn aim(with_sliders: bool) -> Self {
        Self::new(SkillKind::aim(with_sliders))
    }

    #[inline]
    pub(crate) fn flashlight(scaling_factor: f64) -> Self {
        Self::new(SkillKind::flashlight(scaling_factor))
    }

    #[inline]
    pub(crate) fn speed(hit_window: f64) -> Self {
        Self::new(SkillKind::speed(hit_window))
    }

    #[inline]
    fn new(kind: SkillKind) -> Self {
        Self {
            curr_strain: 0.0,
            curr_section_peak: 0.0,

            kind,
            strain_peaks: Vec::with_capacity(128),

            prev_time: None,
        }
    }

    #[inline]
    pub(crate) fn process(&mut self, curr: &DifficultyObject<'_>) {
        self.kind.pre_process();
        self.curr_section_peak = self.strain_value_at(curr).max(self.curr_section_peak);
        self.prev_time = Some(curr.base.time / curr.clock_rate);
        self.kind.post_process(curr);
    }

    #[inline]
    pub(crate) fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.curr_section_peak);
    }

    #[inline]
    pub(crate) fn start_new_section_from(&mut self, time: f64) {
        // The maximum strain of the new section is not zero by default
        self.curr_section_peak = self.calculate_initial_strain(time);
    }

    pub(crate) fn difficulty_value(strain_peaks: &mut [f64], this: &Self) -> f64 {
        // ? Common values to debug
        // println!("---");

        // for (i, strain) in self.strain_peaks.iter().enumerate() {
        //     println!("[{}] {}", i, strain);
        // }

        let mut difficulty = 0.0;
        let mut weight = 1.0;
        let decay_weight = this.kind.decay_weight();

        let (reduced_section_count, difficulty_multiplier) = this.kind.difficulty_values();
        let reduced_section_count_f64 = reduced_section_count as f64;

        strain_peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        let peaks = strain_peaks.iter_mut();

        for (i, strain) in peaks.take(reduced_section_count).enumerate() {
            let clamped = (i as f64 / reduced_section_count_f64).clamp(0.0, 1.0);
            let scale = (lerp(1.0, 10.0, clamped)).log10();
            *strain *= lerp(REDUCED_STRAIN_BASELINE, 1.0, scale);
        }

        strain_peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for &strain in strain_peaks.iter() {
            difficulty += strain * weight;
            weight *= decay_weight;
        }

        difficulty * difficulty_multiplier
    }

    pub(crate) fn calculate_initial_strain(&self, time: f64) -> f64 {
        let prev_time = self.prev_time.unwrap_or(0.0);
        let decayed_strain = self.curr_strain * self.kind.strain_decay(time - prev_time);

        match &self.kind {
            SkillKind::Aim { .. } | SkillKind::Flashlight { .. } => decayed_strain,
            SkillKind::Speed { curr_rhythm, .. } => curr_rhythm * decayed_strain,
        }
    }

    pub(crate) fn strain_value_at(&mut self, curr: &DifficultyObject<'_>) -> f64 {
        self.curr_strain *= self.kind.strain_decay(curr.delta);
        self.curr_strain += self.kind.strain_value_of(curr) * self.kind.skill_multiplier();

        match &mut self.kind {
            SkillKind::Aim { .. } | SkillKind::Flashlight { .. } => self.curr_strain,
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

impl fmt::Debug for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Skill")
            .field("curr_strain", &self.curr_strain)
            .field("curr_section_peak", &self.curr_section_peak)
            .field("kind", &self.kind)
            .field("strain_peaks_len", &self.strain_peaks.len())
            .field("prev_time", &self.prev_time)
            .finish()
    }
}
