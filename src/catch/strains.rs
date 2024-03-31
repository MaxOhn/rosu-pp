use crate::{any::Difficulty, catch::difficulty::DifficultyValues};

use super::convert::CatchBeatmap;

/// The result of calculating the strains on a osu!catch map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub struct CatchStrains {
    /// Strain peaks of the movement skill.
    pub movement: Vec<f64>,
}

impl CatchStrains {
    /// Time between two strains in ms.
    pub const SECTION_LEN: f64 = 750.0;
}

pub fn strains(difficulty: &Difficulty, converted: &CatchBeatmap<'_>) -> CatchStrains {
    let DifficultyValues { movement, .. } = DifficultyValues::calculate(difficulty, converted);

    CatchStrains {
        movement: movement.get_curr_strain_peaks().into_vec(),
    }
}
