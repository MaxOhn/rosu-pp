use rosu_map::section::general::GameMode;

use crate::{
    Difficulty, GameMods,
    model::{
        beatmap::Beatmap,
        mode::{ConvertError, IGameMode},
    },
};

pub use self::{
    attributes::{ManiaDifficultyAttributes, ManiaPerformanceAttributes},
    difficulty::gradual::ManiaGradualDifficulty,
    performance::{ManiaPerformance, gradual::ManiaGradualPerformance},
    score_state::ManiaScoreState,
    strains::ManiaStrains,
};

mod attributes;
mod convert;
mod difficulty;
mod object;
mod performance;
mod score_state;
mod strains;

/// Marker type for [`GameMode::Mania`].
///
/// [`GameMode::Mania`]: rosu_map::section::general::GameMode::Mania
pub struct Mania;

impl Mania {
    pub(crate) fn convert(map: &mut Beatmap, mods: &GameMods) {
        debug_assert!(!map.is_convert && map.mode == GameMode::Osu);
        convert::convert(map, mods);
    }
}

impl IGameMode for Mania {
    type DifficultyAttributes = ManiaDifficultyAttributes;
    type Strains = ManiaStrains;
    type Performance<'map> = ManiaPerformance<'map>;
    type GradualDifficulty = ManiaGradualDifficulty;
    type GradualPerformance = ManiaGradualPerformance;

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
        ManiaPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualDifficulty, ConvertError> {
        ManiaGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualPerformance, ConvertError> {
        ManiaGradualPerformance::new(difficulty, map)
    }
}
