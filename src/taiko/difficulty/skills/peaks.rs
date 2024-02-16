use crate::{
    any::difficulty::skills::Skill,
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
};

use super::{color::Color, rhythm::Rhythm, stamina::Stamina};

const RHYTHM_SKILL_MULTIPLIER: f64 = 0.2 * FINAL_MULTIPLIER;
const COLOR_SKILL_MULTIPLIER: f64 = 0.375 * FINAL_MULTIPLIER;
const STAMINA_SKILL_MULTIPLIER: f64 = 0.375 * FINAL_MULTIPLIER;

const FINAL_MULTIPLIER: f64 = 0.0625;

pub struct Peaks {
    pub color: Color,
    pub rhythm: Rhythm,
    pub stamina: Stamina,
}

impl Peaks {
    pub fn new() -> Self {
        Self {
            color: Color::default(),
            rhythm: Rhythm::default(),
            stamina: Stamina::default(),
        }
    }

    pub fn color_difficulty_value(&self) -> f64 {
        self.color.as_difficulty_value() * COLOR_SKILL_MULTIPLIER
    }

    pub fn rhythm_difficulty_value(&self) -> f64 {
        self.rhythm.as_difficulty_value() * RHYTHM_SKILL_MULTIPLIER
    }

    pub fn stamina_difficulty_value(&self) -> f64 {
        self.stamina.as_difficulty_value() * STAMINA_SKILL_MULTIPLIER
    }

    fn norm(p: f64, values: impl IntoIterator<Item = f64>) -> f64 {
        values
            .into_iter()
            .fold(0.0, |sum, x| sum + x.powf(p))
            .powf(p.recip())
    }

    pub fn difficulty_value(self) -> f64 {
        let color_peaks = self.color.get_curr_strain_peaks();
        let rhythm_peaks = self.rhythm.get_curr_strain_peaks();
        let stamina_peaks = self.stamina.get_curr_strain_peaks();

        let cap = color_peaks
            .len()
            .min(rhythm_peaks.len())
            .min(stamina_peaks.len());
        let mut peaks = Vec::with_capacity(cap);

        let zip = color_peaks
            .iter()
            .copied()
            .zip(rhythm_peaks.iter().copied())
            .zip(stamina_peaks.iter().copied());

        for ((mut color_peak, mut rhythm_peak), mut stamina_peak) in zip {
            color_peak *= COLOR_SKILL_MULTIPLIER;
            rhythm_peak *= RHYTHM_SKILL_MULTIPLIER;
            stamina_peak *= STAMINA_SKILL_MULTIPLIER;

            let mut peak = Self::norm(1.5, [color_peak, stamina_peak]);
            peak = Self::norm(2.0, [peak, rhythm_peak]);

            if peak > 0.0 {
                peaks.push(peak);
            }
        }

        let mut difficulty = 0.0;
        let mut weight = 1.0;

        peaks.sort_by(|a, b| b.total_cmp(a));

        for strain in peaks {
            difficulty += strain * weight;
            weight *= 0.9;
        }

        difficulty
    }
}

impl Default for Peaks {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PeaksSkill<'a> {
    pub color: Skill<'a, Color>,
    pub rhythm: Skill<'a, Rhythm>,
    pub stamina: Skill<'a, Stamina>,
}

impl<'a> PeaksSkill<'a> {
    pub fn new(diff_objects: &'a TaikoDifficultyObjects) -> Self {
        Self {
            color: Skill::new(Color::default(), diff_objects),
            rhythm: Skill::new(Rhythm::default(), diff_objects),
            stamina: Skill::new(Stamina::default(), diff_objects),
        }
    }

    pub fn process(&mut self, curr: &TaikoDifficultyObject) {
        self.rhythm.process(curr);
        self.color.process(curr);
        self.stamina.process(curr);
    }

    pub fn into_inner(self) -> Peaks {
        Peaks {
            color: self.color.inner,
            rhythm: self.rhythm.inner,
            stamina: self.stamina.inner,
        }
    }
}
