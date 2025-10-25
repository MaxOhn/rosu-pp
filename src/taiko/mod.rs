use rosu_map::section::general::GameMode;

use crate::{
    Difficulty,
    model::{
        beatmap::Beatmap,
        mode::{ConvertError, IGameMode},
    },
};

pub use self::{
    attributes::{TaikoDifficultyAttributes, TaikoPerformanceAttributes},
    difficulty::gradual::TaikoGradualDifficulty,
    performance::{TaikoPerformance, gradual::TaikoGradualPerformance},
    score_state::TaikoScoreState,
    strains::TaikoStrains,
};

mod attributes;
mod convert;
mod difficulty;
mod object;
mod performance;
mod score_state;
mod strains;

/// Marker type for [`GameMode::Taiko`].
///
/// [`GameMode::Taiko`]: rosu_map::section::general::GameMode::Taiko
pub struct Taiko;

impl Taiko {
    pub fn convert(map: &mut Beatmap) {
        debug_assert!(!map.is_convert && map.mode == GameMode::Osu);
        convert::convert(map);
    }
}

impl IGameMode for Taiko {
    type DifficultyAttributes = TaikoDifficultyAttributes;
    type Strains = TaikoStrains;
    type Performance<'map> = TaikoPerformance<'map>;
    type GradualDifficulty = TaikoGradualDifficulty;
    type GradualPerformance = TaikoGradualPerformance;

    fn difficulty(
        difficulty: &Difficulty,
        map: &Beatmap,
    ) -> Result<Self::DifficultyAttributes, ConvertError> {
        difficulty::difficulty(difficulty, map)
    }

    fn strains(difficulty: &Difficulty, map: &Beatmap) -> Result<Self::Strains, ConvertError> {
        strains::strains(difficulty, map)
    }

    fn performance(map: &Beatmap) -> Self::Performance<'_> {
        TaikoPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualDifficulty, ConvertError> {
        TaikoGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualPerformance, ConvertError> {
        TaikoGradualPerformance::new(difficulty, map)
    }
}
