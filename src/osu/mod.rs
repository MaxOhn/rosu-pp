use rosu_map::util::Pos;

use crate::{
    Difficulty,
    model::{
        beatmap::Beatmap,
        mode::{ConvertError, IGameMode},
    },
};

pub use self::{
    attributes::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    difficulty::gradual::OsuGradualDifficulty,
    performance::{OsuHitResultParams, OsuPerformance, gradual::OsuGradualPerformance},
    score_state::{OsuHitResults, OsuScoreOrigin, OsuScoreState},
    strains::OsuStrains,
};

mod attributes;
mod convert;
mod difficulty;
mod object;
mod performance;
mod score_state;
mod strains;

const PLAYFIELD_BASE_SIZE: Pos = Pos::new(512.0, 384.0);

/// Marker type for [`GameMode::Osu`].
///
/// [`GameMode::Osu`]: rosu_map::section::general::GameMode::Osu
pub struct Osu;

impl IGameMode for Osu {
    type DifficultyAttributes = OsuDifficultyAttributes;
    type Strains = OsuStrains;
    type Performance<'map> = OsuPerformance<'map>;
    type GradualDifficulty = OsuGradualDifficulty;
    type GradualPerformance = OsuGradualPerformance;

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
        OsuPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualDifficulty, ConvertError> {
        OsuGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualPerformance, ConvertError> {
        OsuGradualPerformance::new(difficulty, map)
    }
}
