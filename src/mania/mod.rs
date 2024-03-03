use crate::{
    model::{
        beatmap::Beatmap,
        mode::{ConvertStatus, IGameMode},
    },
    Difficulty,
};

pub use self::{
    attributes::{ManiaDifficultyAttributes, ManiaPerformanceAttributes},
    convert::ManiaBeatmap,
    difficulty::gradual::ManiaGradualDifficulty,
    performance::{gradual::ManiaGradualPerformance, ManiaPerformance},
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

impl IGameMode for Mania {
    type DifficultyAttributes = ManiaDifficultyAttributes;
    type Strains = ManiaStrains;
    type Performance<'map> = ManiaPerformance<'map>;
    type GradualDifficulty = ManiaGradualDifficulty;
    type GradualPerformance = ManiaGradualPerformance;

    fn check_convert(map: &Beatmap) -> ConvertStatus {
        convert::check_convert(map)
    }

    fn try_convert(map: &mut Beatmap) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &Difficulty,
        converted: &ManiaBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &Difficulty, converted: &ManiaBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }

    fn performance(map: ManiaBeatmap<'_>) -> Self::Performance<'_> {
        ManiaPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: &Difficulty,
        map: &ManiaBeatmap<'_>,
    ) -> Self::GradualDifficulty {
        ManiaGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: &Difficulty,
        map: &ManiaBeatmap<'_>,
    ) -> Self::GradualPerformance {
        ManiaGradualPerformance::new(difficulty, map)
    }
}
