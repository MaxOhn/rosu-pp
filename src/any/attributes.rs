use crate::{
    catch::{Catch, CatchDifficultyAttributes, CatchPerformanceAttributes},
    mania::{Mania, ManiaDifficultyAttributes, ManiaPerformanceAttributes},
    model::mode::IGameMode,
    osu::{Osu, OsuDifficultyAttributes, OsuPerformanceAttributes},
    taiko::{Taiko, TaikoDifficultyAttributes, TaikoPerformanceAttributes},
};

use super::performance::Performance;

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
        self.into()
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
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait AttributeProvider {
    /// Provide the actual difficulty attributes.
    fn attributes(self) -> DifficultyAttributes;
}

impl AttributeProvider for DifficultyAttributes {
    fn attributes(self) -> DifficultyAttributes {
        self
    }
}

impl AttributeProvider for PerformanceAttributes {
    fn attributes(self) -> DifficultyAttributes {
        match self {
            Self::Osu(attrs) => DifficultyAttributes::Osu(attrs.difficulty),
            Self::Taiko(attrs) => DifficultyAttributes::Taiko(attrs.difficulty),
            Self::Catch(attrs) => DifficultyAttributes::Catch(attrs.difficulty),
            Self::Mania(attrs) => DifficultyAttributes::Mania(attrs.difficulty),
        }
    }
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait ModeAttributeProvider<M: IGameMode> {
    /// Provide the actual difficulty attributes.
    fn attributes(self) -> Option<M::DifficultyAttributes>;
}

impl<M: IGameMode> ModeAttributeProvider<M> for M::DifficultyAttributes {
    fn attributes(self) -> Option<M::DifficultyAttributes> {
        Some(self)
    }
}

macro_rules! impl_attr_provider {
    ( $mode:ident: $difficulty:ident, $performance:ident ) => {
        impl AttributeProvider for $difficulty {
            fn attributes(self) -> DifficultyAttributes {
                DifficultyAttributes::$mode(self)
            }
        }

        impl AttributeProvider for $performance {
            fn attributes(self) -> DifficultyAttributes {
                DifficultyAttributes::$mode(self.difficulty)
            }
        }

        impl ModeAttributeProvider<$mode> for $performance {
            fn attributes(self) -> Option<<$mode as IGameMode>::DifficultyAttributes> {
                Some(self.difficulty)
            }
        }

        impl ModeAttributeProvider<$mode> for DifficultyAttributes {
            fn attributes(self) -> Option<<$mode as IGameMode>::DifficultyAttributes> {
                if let Self::$mode(attrs) = self {
                    Some(attrs)
                } else {
                    None
                }
            }
        }

        impl ModeAttributeProvider<$mode> for PerformanceAttributes {
            fn attributes(self) -> Option<<$mode as IGameMode>::DifficultyAttributes> {
                if let Self::$mode(attrs) = self {
                    Some(attrs.difficulty)
                } else {
                    None
                }
            }
        }
    };
}

impl_attr_provider!(Catch: CatchDifficultyAttributes, CatchPerformanceAttributes);
impl_attr_provider!(Mania: ManiaDifficultyAttributes, ManiaPerformanceAttributes);
impl_attr_provider!(Osu: OsuDifficultyAttributes, OsuPerformanceAttributes);
impl_attr_provider!(Taiko: TaikoDifficultyAttributes, TaikoPerformanceAttributes);
