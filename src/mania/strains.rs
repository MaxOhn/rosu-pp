use rosu_map::section::general::GameMode;

use crate::{
    Beatmap,
    any::{Difficulty, difficulty::skills::StrainSkill},
    mania::difficulty::DifficultyValues,
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
    let map = map.convert_ref(GameMode::Mania, difficulty.get_mods())?;
    let values = DifficultyValues::calculate(difficulty, &map);

    Ok(ManiaStrains {
        strains: values.strain.into_current_strain_peaks().into_vec(),
    })
}
