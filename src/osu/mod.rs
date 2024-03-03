use rosu_map::util::Pos;

use crate::{
    model::{
        beatmap::Beatmap,
        mode::{ConvertStatus, IGameMode},
    },
    Difficulty,
};

pub use self::{
    attributes::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    convert::OsuBeatmap,
    difficulty::gradual::OsuGradualDifficulty,
    performance::{gradual::OsuGradualPerformance, OsuPerformance},
    score_state::OsuScoreState,
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

    fn check_convert(map: &Beatmap) -> ConvertStatus {
        convert::check_convert(map)
    }

    fn try_convert(map: &mut Beatmap) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &Difficulty,
        converted: &OsuBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &Difficulty, converted: &OsuBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }

    fn performance(map: OsuBeatmap<'_>) -> Self::Performance<'_> {
        OsuPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: &Difficulty,
        map: &OsuBeatmap<'_>,
    ) -> Self::GradualDifficulty {
        OsuGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: &Difficulty,
        map: &OsuBeatmap<'_>,
    ) -> Self::GradualPerformance {
        OsuGradualPerformance::new(difficulty, map)
    }
}
