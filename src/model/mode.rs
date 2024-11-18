use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub use rosu_map::section::general::GameMode;

use crate::Difficulty;

use super::beatmap::Beatmap;

/// A way to specify a gamemode at compile-time.
///
/// Notably, this is implemented for the marker types [`Osu`], [`Taiko`],
/// [`Catch`], and [`Mania`].
///
/// [`Osu`]: crate::osu::Osu
/// [`Taiko`]: crate::taiko::Taiko
/// [`Catch`]: crate::catch::Catch
/// [`Mania`]: crate::mania::Mania
pub trait IGameMode: Sized {
    /// The resulting type of a difficulty calculation.
    type DifficultyAttributes;

    /// The resulting type of a strain calculation.
    type Strains;

    /// The type of a performance calculator.
    type Performance<'map>;

    /// The type of a gradual difficulty calculator.
    type GradualDifficulty;

    /// The type of a gradual performance calculator.
    type GradualPerformance;

    /// Perform a difficulty calculation for a [`Beatmap`] and process the
    /// final skill values.
    fn difficulty(
        difficulty: &Difficulty,
        map: &Beatmap,
    ) -> Result<Self::DifficultyAttributes, ConvertError>;

    /// Perform a difficulty calculation for a [`Beatmap`] without processing
    /// the final skill values.
    fn strains(difficulty: &Difficulty, map: &Beatmap) -> Result<Self::Strains, ConvertError>;

    /// Create a performance calculator for a [`Beatmap`].
    fn performance(map: &Beatmap) -> Self::Performance<'_>;

    /// Create a gradual difficulty calculator for a [`Beatmap`].
    fn gradual_difficulty(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualDifficulty, ConvertError>;

    /// Create a gradual performance calculator for a [`Beatmap`].
    fn gradual_performance(
        difficulty: Difficulty,
        map: &Beatmap,
    ) -> Result<Self::GradualPerformance, ConvertError>;
}

/// Error type when failing to convert a [`Beatmap`] from one [`GameMode`] to
/// another.
#[derive(Copy, Clone, Debug)]
pub enum ConvertError {
    /// Cannot convert an already converted map
    AlreadyConverted,
    /// Cannot convert from [`GameMode`] `from` to `to`
    Convert { from: GameMode, to: GameMode },
}

impl Error for ConvertError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConvertError::AlreadyConverted => {
                f.write_str("Cannot convert an already converted map")
            }
            ConvertError::Convert { from, to } => {
                write!(f, "Cannot convert from {from:?} to {to:?}")
            }
        }
    }
}
