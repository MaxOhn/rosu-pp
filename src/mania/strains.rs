use crate::{
    Beatmap,
    any::{Difficulty, difficulty::skills::strain_skill::StrainSkill},
    mania::{convert::prepare_map, difficulty::DifficultyValues},
    model::mode::ConvertError,
};

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

pub fn strains(difficulty: &Difficulty, map: &Beatmap) -> Result<ManiaStrains, ConvertError> {
    let map = prepare_map(difficulty, map)?;
    let values = DifficultyValues::calculate(difficulty, &map);

    Ok(ManiaStrains {
        strains: values.strain.into_current_strain_peaks(),
    })
}
