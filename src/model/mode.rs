pub use rosu_map::section::general::GameMode;

use crate::any::ModeDifficulty;

use super::beatmap::{Beatmap, Converted};

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

    /// Check whether the map's mode can be converted to the current type.
    fn check_convert(map: &Beatmap) -> ConvertStatus;

    /// Attempt to convert a beatmap.
    ///
    /// In case [`ConvertStatus::Incompatible`] is returned, the map is not
    /// modified.
    fn try_convert(map: &mut Beatmap) -> ConvertStatus;

    /// Perform a difficulty calculation for a [`Converted`] beatmap and
    /// process the final skill values.
    fn difficulty(
        difficulty: &ModeDifficulty,
        map: &Converted<'_, Self>,
    ) -> Self::DifficultyAttributes;

    /// Perform a difficulty calculation for a [`Converted`] beatmap without
    /// processing the final skill values.
    fn strains(difficulty: &ModeDifficulty, map: &Converted<'_, Self>) -> Self::Strains;

    /// Create a performance calculator for a [`Converted`] beatmap.
    fn performance(map: Converted<'_, Self>) -> Self::Performance<'_>;

    /// Create a gradual difficulty calculator for a [`Converted`] beatmap.
    fn gradual_difficulty(
        difficulty: &ModeDifficulty,
        map: &Converted<'_, Self>,
    ) -> Self::GradualDifficulty;

    /// Create a gradual performance calculator for a [`Converted`] beatmap.
    fn gradual_performance(
        difficulty: &ModeDifficulty,
        map: &Converted<'_, Self>,
    ) -> Self::GradualPerformance;
}

/// The status of a conversion through [`IGameMode::try_convert`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConvertStatus {
    /// Conversion not necessary.
    Noop,
    /// Conversion possible.
    Conversion,
    /// Conversion not possible.
    Incompatible,
}

impl ConvertStatus {
    /// Whether this [`ConvertStatus`] represents a success.
    pub const fn success(self) -> bool {
        matches!(self, Self::Noop | Self::Conversion)
    }
}
