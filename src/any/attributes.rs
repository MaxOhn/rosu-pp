use crate::{
    catch::{CatchDifficultyAttributes, CatchPerformanceAttributes},
    mania::{ManiaDifficultyAttributes, ManiaPerformanceAttributes},
    osu::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    taiko::{TaikoDifficultyAttributes, TaikoPerformanceAttributes},
};

use super::performance::{Performance, into::IntoPerformance};

/// The result of a difficulty calculation based on the mode.
#[derive(Clone, Debug, PartialEq)]
pub enum DifficultyAttributes {
    /// osu!standard difficulty calculation result.
    Osu(OsuDifficultyAttributes),
    /// osu!taiko difficulty calculation result.
    Taiko(TaikoDifficultyAttributes),
    /// osu!catch difficulty calculation result.
    Catch(CatchDifficultyAttributes),
    /// osu!mania difficulty calculation result.
    Mania(ManiaDifficultyAttributes),
}

impl DifficultyAttributes {
    /// The star value.
    pub const fn stars(&self) -> f64 {
        match self {
            Self::Osu(attrs) => attrs.stars,
            Self::Taiko(attrs) => attrs.stars,
            Self::Catch(attrs) => attrs.stars,
            Self::Mania(attrs) => attrs.stars,
        }
    }

    /// The maximum combo of the map.
    pub const fn max_combo(&self) -> u32 {
        match self {
            Self::Osu(attrs) => attrs.max_combo,
            Self::Taiko(attrs) => attrs.max_combo,
            Self::Catch(attrs) => attrs.max_combo(),
            Self::Mania(attrs) => attrs.max_combo,
        }
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> Performance<'a> {
        self.into_performance()
    }
}

/// The result of a performance calculation based on the mode.
#[derive(Clone, Debug, PartialEq)]
pub enum PerformanceAttributes {
    /// osu!standard performance calculation result.
    Osu(OsuPerformanceAttributes),
    /// osu!taiko performance calculation result.
    Taiko(TaikoPerformanceAttributes),
    /// osu!catch performance calculation result.
    Catch(CatchPerformanceAttributes),
    /// osu!mania performance calculation result.
    Mania(ManiaPerformanceAttributes),
}

impl PerformanceAttributes {
    /// The pp value.
    pub const fn pp(&self) -> f64 {
        match self {
            Self::Osu(attrs) => attrs.pp,
            Self::Taiko(attrs) => attrs.pp,
            Self::Catch(attrs) => attrs.pp,
            Self::Mania(attrs) => attrs.pp,
        }
    }

    /// The star value.
    pub const fn stars(&self) -> f64 {
        match self {
            Self::Osu(attrs) => attrs.stars(),
            Self::Taiko(attrs) => attrs.stars(),
            Self::Catch(attrs) => attrs.stars(),
            Self::Mania(attrs) => attrs.stars(),
        }
    }

    /// Difficulty attributes that were used for the performance calculation.
    pub fn difficulty_attributes(&self) -> DifficultyAttributes {
        match self {
            Self::Osu(attrs) => DifficultyAttributes::Osu(attrs.difficulty.clone()),
            Self::Taiko(attrs) => DifficultyAttributes::Taiko(attrs.difficulty.clone()),
            Self::Catch(attrs) => DifficultyAttributes::Catch(attrs.difficulty.clone()),
            Self::Mania(attrs) => DifficultyAttributes::Mania(attrs.difficulty.clone()),
        }
    }

    /// The maximum combo of the map.
    pub const fn max_combo(&self) -> u32 {
        match self {
            Self::Osu(attrs) => attrs.difficulty.max_combo,
            Self::Taiko(attrs) => attrs.difficulty.max_combo,
            Self::Catch(attrs) => attrs.difficulty.max_combo(),
            Self::Mania(attrs) => attrs.difficulty.max_combo,
        }
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> Performance<'a> {
        self.into_performance()
    }
}

impl From<PerformanceAttributes> for DifficultyAttributes {
    fn from(attrs: PerformanceAttributes) -> Self {
        attrs.difficulty_attributes()
    }
}
