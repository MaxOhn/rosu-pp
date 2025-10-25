use rosu_map::section::general::GameMode;

use crate::{
    Beatmap,
    any::{Difficulty, difficulty::skills::StrainSkill},
    catch::difficulty::DifficultyValues,
    model::mode::ConvertError,
};

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

pub fn strains(difficulty: &Difficulty, map: &Beatmap) -> Result<CatchStrains, ConvertError> {
    let map = map.convert_ref(GameMode::Catch, difficulty.get_mods())?;
    let DifficultyValues { movement, .. } = DifficultyValues::calculate(difficulty, &map);

    Ok(CatchStrains {
        movement: movement.into_current_strain_peaks().into_vec(),
    })
}
