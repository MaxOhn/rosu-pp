use std::borrow::Cow;

use crate::{
    any::ModeDifficulty,
    model::{
        beatmap::Beatmap,
        mode::{ConvertStatus, IGameMode},
    },
};

pub use self::{
    attributes::{ManiaDifficultyAttributes, ManiaPerformanceAttributes},
    convert::ManiaBeatmap,
    performance::ManiaPerformance,
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

    fn try_convert(map: &mut Cow<'_, Beatmap>) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &ModeDifficulty,
        converted: &ManiaBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &ModeDifficulty, converted: &ManiaBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }
}
