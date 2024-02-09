use crate::{any::ModeDifficulty, mania::difficulty::DifficultyValues};

use super::convert::ManiaBeatmap;

/// The result of calculating the strains on a osu!mania map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub struct ManiaStrains {
    /// Strain peaks of the strain skill.
    pub strains: Vec<f64>,
}

impl ManiaStrains {
    /// Time between two strains in ms.
    pub const SECTION_LEN: f64 = 400.0;
}

pub fn strains(difficulty: &ModeDifficulty, converted: &ManiaBeatmap<'_>) -> ManiaStrains {
    let values = DifficultyValues::calculate(difficulty, converted);

    ManiaStrains {
        strains: values.strain.get_curr_strain_peaks(),
    }
}
