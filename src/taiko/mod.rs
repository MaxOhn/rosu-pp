use crate::{
    any::ModeDifficulty,
    model::{
        beatmap::Beatmap,
        mode::{ConvertStatus, IGameMode},
    },
};

pub use self::{
    attributes::{TaikoDifficultyAttributes, TaikoPerformanceAttributes},
    convert::TaikoBeatmap,
    difficulty::gradual::TaikoGradualDifficulty,
    performance::{gradual::TaikoGradualPerformance, TaikoPerformance},
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

impl IGameMode for Taiko {
    type DifficultyAttributes = TaikoDifficultyAttributes;
    type Strains = TaikoStrains;
    type Performance<'map> = TaikoPerformance<'map>;
    type GradualDifficulty = TaikoGradualDifficulty;
    type GradualPerformance = TaikoGradualPerformance;

    fn check_convert(map: &Beatmap) -> ConvertStatus {
        convert::check_convert(map)
    }

    fn try_convert(map: &mut Beatmap) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &ModeDifficulty,
        converted: &TaikoBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &ModeDifficulty, converted: &TaikoBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }

    fn performance(map: TaikoBeatmap<'_>) -> Self::Performance<'_> {
        TaikoPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: &ModeDifficulty,
        map: &TaikoBeatmap<'_>,
    ) -> Self::GradualDifficulty {
        TaikoGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: &ModeDifficulty,
        map: &TaikoBeatmap<'_>,
    ) -> Self::GradualPerformance {
        TaikoGradualPerformance::new(difficulty, map)
    }
}
