use crate::{
    model::{
        beatmap::Beatmap,
        mode::{ConvertStatus, IGameMode},
    },
    Difficulty,
};

pub use self::{
    attributes::{CatchDifficultyAttributes, CatchPerformanceAttributes},
    convert::CatchBeatmap,
    difficulty::gradual::CatchGradualDifficulty,
    performance::{gradual::CatchGradualPerformance, CatchPerformance},
    score_state::CatchScoreState,
    strains::CatchStrains,
};

mod attributes;
mod catcher;
mod convert;
mod difficulty;
mod object;
mod performance;
mod score_state;
mod strains;

const PLAYFIELD_WIDTH: f32 = 512.0;

/// Marker type for [`GameMode::Catch`].
///
/// [`GameMode::Catch`]: rosu_map::section::general::GameMode::Catch
pub struct Catch;

impl IGameMode for Catch {
    type DifficultyAttributes = CatchDifficultyAttributes;
    type Strains = CatchStrains;
    type Performance<'map> = CatchPerformance<'map>;
    type GradualDifficulty = CatchGradualDifficulty;
    type GradualPerformance = CatchGradualPerformance;

    fn check_convert(map: &Beatmap) -> ConvertStatus {
        convert::check_convert(map)
    }

    fn try_convert(map: &mut Beatmap) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &Difficulty,
        converted: &CatchBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &Difficulty, converted: &CatchBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }

    fn performance(map: CatchBeatmap<'_>) -> Self::Performance<'_> {
        CatchPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: Difficulty,
        map: &CatchBeatmap<'_>,
    ) -> Self::GradualDifficulty {
        CatchGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: Difficulty,
        map: &CatchBeatmap<'_>,
    ) -> Self::GradualPerformance {
        CatchGradualPerformance::new(difficulty, map)
    }
}
