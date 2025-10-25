use rosu_map::section::general::GameMode;

use crate::{
    Difficulty,
    model::{
        beatmap::Beatmap,
        mode::{ConvertError, IGameMode},
    },
};

pub use self::{
    attributes::{CatchDifficultyAttributes, CatchPerformanceAttributes},
    difficulty::gradual::CatchGradualDifficulty,
    performance::{CatchPerformance, gradual::CatchGradualPerformance},
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

impl Catch {
    pub fn convert(map: &mut Beatmap) {
        debug_assert!(!map.is_convert && map.mode == GameMode::Osu);
        convert::convert(map);
    }
}

impl IGameMode for Catch {
    type DifficultyAttributes = CatchDifficultyAttributes;
    type Strains = CatchStrains;
    type Performance<'map> = CatchPerformance<'map>;
    type GradualDifficulty = CatchGradualDifficulty;
    type GradualPerformance = CatchGradualPerformance;

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
        CatchPerformance::new(map)
    }

    fn gradual_difficulty(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualDifficulty, ConvertError> {
        CatchGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualPerformance, ConvertError> {
        CatchGradualPerformance::new(difficulty, map)
    }
}
