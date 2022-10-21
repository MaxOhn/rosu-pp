use std::cmp::Ordering;

use crate::taiko::difficulty_object::{ObjectLists, TaikoDifficultyObject};

use super::{colour::Colour, rhythm::Rhythm, stamina::Stamina, Skill, StrainSkill};

#[derive(Clone, Debug)]
pub(crate) struct Peaks {
    colour: Colour,
    rhythm: Rhythm,
    stamina: Stamina,
}

impl Peaks {
    const RHYTHM_SKILL_MULTIPLIER: f64 = 0.2 * Self::FINAL_MULTIPLIER;
    const COLOUR_SKILL_MULTIPLIER: f64 = 0.375 * Self::FINAL_MULTIPLIER;
    const STAMINA_SKILL_MULTIPLIER: f64 = 0.375 * Self::FINAL_MULTIPLIER;

    const FINAL_MULTIPLIER: f64 = 0.0625;

    pub(crate) fn new() -> Self {
        Self {
            colour: Colour::new(),
            rhythm: Rhythm::new(),
            stamina: Stamina::new(),
        }
    }

    pub(crate) fn difficulty_values(self) -> PeaksDifficultyValues {
        let colour_rating = <Colour as StrainSkill>::difficulty_value(self.colour.clone())
            * Self::COLOUR_SKILL_MULTIPLIER;
        let rhythm_rating = <Rhythm as StrainSkill>::difficulty_value(self.rhythm.clone())
            * Self::RHYTHM_SKILL_MULTIPLIER;
        let stamina_rating = <Stamina as StrainSkill>::difficulty_value(self.stamina.clone())
            * Self::STAMINA_SKILL_MULTIPLIER;

        PeaksDifficultyValues {
            colour_rating,
            rhythm_rating,
            stamina_rating,
            combined_rating: self.difficulty_value(),
        }
    }

    pub(crate) fn into_raw(self) -> PeaksRaw {
        PeaksRaw {
            colour: self.colour.strain_peaks,
            rhythm: self.rhythm.strain_peaks,
            stamina: self.stamina.strain_peaks,
        }
    }

    fn norm(p: f64, values: impl IntoIterator<Item = f64>) -> f64 {
        values
            .into_iter()
            .fold(0.0, |sum, x| sum + x.powf(p))
            .powf(p.recip())
    }
}

impl Skill for Peaks {
    #[inline]
    fn process(&mut self, curr: &TaikoDifficultyObject, hit_objects: &ObjectLists) {
        <Colour as Skill>::process(&mut self.colour, curr, hit_objects);
        <Rhythm as Skill>::process(&mut self.rhythm, curr, hit_objects);
        <Stamina as Skill>::process(&mut self.stamina, curr, hit_objects);
    }

    fn difficulty_value(self) -> f64 {
        let mut peaks = Vec::new();

        let colour_peaks = self.colour.get_curr_strain_peaks();
        let rhythm_peaks = self.rhythm.get_curr_strain_peaks();
        let stamina_peaks = self.stamina.get_curr_strain_peaks();

        let zip = colour_peaks
            .into_iter()
            .zip(rhythm_peaks)
            .zip(stamina_peaks);

        for ((mut colour_peak, mut rhythm_peak), mut stamina_peak) in zip {
            colour_peak *= Self::COLOUR_SKILL_MULTIPLIER;
            rhythm_peak *= Self::RHYTHM_SKILL_MULTIPLIER;
            stamina_peak *= Self::STAMINA_SKILL_MULTIPLIER;

            let mut peak = Self::norm(1.5, [colour_peak, stamina_peak]);
            peak = Self::norm(2.0, [peak, rhythm_peak]);

            // * Sections with 0 strain are excluded to avoid worst-case
            // * time complexity of the following sort (e.g. /b/2351871).
            // * These sections will not contribute to the difficulty.
            if peak > 0.0 {
                peaks.push(peak);
            }
        }

        let mut difficulty = 0.0;
        let mut weight = 1.0;

        peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        for strain in peaks {
            difficulty += strain * weight;
            weight *= 0.9;
        }

        difficulty
    }
}

pub(crate) struct PeaksDifficultyValues {
    pub(crate) colour_rating: f64,
    pub(crate) rhythm_rating: f64,
    pub(crate) stamina_rating: f64,
    pub(crate) combined_rating: f64,
}

pub(crate) struct PeaksRaw {
    pub(crate) colour: Vec<f64>,
    pub(crate) rhythm: Vec<f64>,
    pub(crate) stamina: Vec<f64>,
}
