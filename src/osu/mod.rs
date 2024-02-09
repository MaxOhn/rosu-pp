use std::borrow::Cow;

use rosu_map::util::Pos;

use crate::{
    any::ModeDifficulty,
    model::{
        beatmap::Beatmap,
        mode::{ConvertStatus, IGameMode},
    },
};

pub use self::{
    attributes::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    convert::OsuBeatmap,
    performance::OsuPerformance,
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

    fn try_convert(map: &mut Cow<'_, Beatmap>) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &ModeDifficulty,
        converted: &OsuBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &ModeDifficulty, converted: &OsuBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }
}
