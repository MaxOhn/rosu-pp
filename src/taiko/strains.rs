use crate::{taiko::difficulty::DifficultyValues, Difficulty};

use super::convert::TaikoBeatmap;

/// The result of calculating the strains on a osu!taiko map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub struct TaikoStrains {
    /// Strain peaks of the color skill.
    pub color: Vec<f64>,
    /// Strain peaks of the rhythm skill.
    pub rhythm: Vec<f64>,
    /// Strain peaks of the stamina skill.
    pub stamina: Vec<f64>,
}

impl TaikoStrains {
    /// Time between two strains in ms.
    pub const SECTION_LEN: f64 = 400.0;
}

pub fn strains(difficulty: &Difficulty, converted: &TaikoBeatmap<'_>) -> TaikoStrains {
    let values = DifficultyValues::calculate(difficulty, converted);

    TaikoStrains {
        color: values.skills.color.get_curr_strain_peaks().into_vec(),
        rhythm: values.skills.rhythm.get_curr_strain_peaks().into_vec(),
        stamina: values.skills.stamina.get_curr_strain_peaks().into_vec(),
    }
}
