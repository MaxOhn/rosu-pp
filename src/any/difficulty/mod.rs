use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    num::NonZeroU64,
};

use rosu_map::section::general::GameMode;

use crate::{
    GradualDifficulty, GradualPerformance,
    catch::Catch,
    mania::Mania,
    model::{beatmap::Beatmap, mode::ConvertError, mods::GameMods},
    osu::Osu,
    taiko::Taiko,
};

use super::{InspectDifficulty, Strains, attributes::DifficultyAttributes};

pub mod gradual;
pub mod inspect;
pub mod object;
pub mod skills;

use crate::model::mode::IGameMode;

/// Difficulty calculator on maps of any mode.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty, any::DifficultyAttributes};
///
/// let map = Beatmap::from_path("./resources/2118524.osu").unwrap();
///
/// let attrs: DifficultyAttributes = Difficulty::new()
///     .mods(8 + 1024) // HDFL
///     .calculate(&map);
/// ```
#[derive(Clone, PartialEq)]
#[must_use]
pub struct Difficulty {
    mods: GameMods,
    passed_objects: Option<u32>,
    /// Clock rate will be clamped internally between 0.01 and 100.0.
    ///
    /// Since its minimum value is 0.01, its bits are never zero.
    ///
    /// This allows for an optimization to reduce the struct size by storing its
    /// bits as a [`NonZeroU64`].
    clock_rate: Option<NonZeroU64>,
    ar: Option<ModsDependent>,
    cs: Option<ModsDependent>,
    hp: Option<ModsDependent>,
    od: Option<ModsDependent>,
    hardrock_offsets: Option<bool>,
    lazer: Option<bool>,
}

/// Wrapper for beatmap attributes in [`Difficulty`].
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ModsDependent {
    /// Value of the beatmap attribute.
    pub value: f32,
    /// Whether `value` should be used as is or modified based on mods.
    ///
    /// `true` means "value already considers mods" i.e. use as is;
    /// `false` means modify with mods.
    pub with_mods: bool,
}

impl ModsDependent {
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            with_mods: false,
        }
    }
}

impl Difficulty {
    /// Create a new difficulty calculator.
    pub const fn new() -> Self {
        Self {
            mods: GameMods::DEFAULT,
            passed_objects: None,
            clock_rate: None,
            ar: None,
            cs: None,
            hp: None,
            od: None,
            hardrock_offsets: None,
            lazer: None,
        }
    }

    /// Turn this [`Difficulty`] into a [`InspectDifficulty`] to inspect its
    /// configured values.
    pub fn inspect(self) -> InspectDifficulty {
        let Self {
            mods,
            passed_objects,
            clock_rate,
            ar,
            cs,
            hp,
            od,
            hardrock_offsets,
            lazer,
        } = self;

        InspectDifficulty {
            mods,
            passed_objects,
            clock_rate: clock_rate.map(non_zero_u64_to_f64),
            ar,
            cs,
            hp,
            od,
            hardrock_offsets,
            lazer,
        }
    }

    /// Specify mods.
    ///
    /// Accepted types are
    /// - `u32`
    /// - [`rosu_mods::GameModsLegacy`]
    /// - [`rosu_mods::GameMods`]
    /// - [`rosu_mods::GameModsIntermode`]
    /// - [`&rosu_mods::GameModsIntermode`](rosu_mods::GameModsIntermode)
    ///
    /// See <https://github.com/ppy/osu-api/wiki#mods>
    pub fn mods(self, mods: impl Into<GameMods>) -> Self {
        Self {
            mods: mods.into(),
            ..self
        }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    pub const fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | 0.01    | 100     |
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        let clock_rate = clock_rate.clamp(0.01, 100.0).to_bits();

        // SAFETY: The minimum value is 0.01 so its bits can never be fully
        // zero.
        let non_zero = unsafe { NonZeroU64::new_unchecked(clock_rate) };

        Self {
            clock_rate: Some(non_zero),
            ..self
        }
    }

    /// Override a beatmap's set AR.
    ///
    /// Only relevant for osu! and osu!catch.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn ar(self, ar: f32, with_mods: bool) -> Self {
        Self {
            ar: Some(ModsDependent {
                value: ar.clamp(-20.0, 20.0),
                with_mods,
            }),
            ..self
        }
    }

    /// Override a beatmap's set CS.
    ///
    /// Only relevant for osu! and osu!catch.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn cs(self, cs: f32, with_mods: bool) -> Self {
        Self {
            cs: Some(ModsDependent {
                value: cs.clamp(-20.0, 20.0),
                with_mods,
            }),
            ..self
        }
    }

    /// Override a beatmap's set HP.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn hp(self, hp: f32, with_mods: bool) -> Self {
        Self {
            hp: Some(ModsDependent {
                value: hp.clamp(-20.0, 20.0),
                with_mods,
            }),
            ..self
        }
    }

    /// Override a beatmap's set OD.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn od(self, od: f32, with_mods: bool) -> Self {
        Self {
            od: Some(ModsDependent {
                value: od.clamp(-20.0, 20.0),
                with_mods,
            }),
            ..self
        }
    }

    /// Adjust patterns as if the HR mod is enabled.
    ///
    /// Only relevant for osu!catch.
    pub const fn hardrock_offsets(mut self, hardrock_offsets: bool) -> Self {
        self.hardrock_offsets = Some(hardrock_offsets);

        self
    }

    /// Whether the calculated attributes belong to an osu!lazer or osu!stable
    /// score.
    ///
    /// Defaults to `true`.
    pub const fn lazer(mut self, lazer: bool) -> Self {
        self.lazer = Some(lazer);

        self
    }

    /// Perform the difficulty calculation.
    #[expect(clippy::missing_panics_doc, reason = "unreachable")]
    pub fn calculate(&self, map: &Beatmap) -> DifficultyAttributes {
        match map.mode {
            GameMode::Osu => DifficultyAttributes::Osu(
                Osu::difficulty(self, map).expect("no conversion required"),
            ),
            GameMode::Taiko => DifficultyAttributes::Taiko(
                Taiko::difficulty(self, map).expect("no conversion required"),
            ),
            GameMode::Catch => DifficultyAttributes::Catch(
                Catch::difficulty(self, map).expect("no conversion required"),
            ),
            GameMode::Mania => DifficultyAttributes::Mania(
                Mania::difficulty(self, map).expect("no conversion required"),
            ),
        }
    }

    /// Perform the difficulty calculation for a specific [`IGameMode`].
    pub fn calculate_for_mode<M: IGameMode>(
        &self,
        map: &Beatmap,
    ) -> Result<M::DifficultyAttributes, ConvertError> {
        M::difficulty(self, map)
    }

    /// Perform the difficulty calculation but instead of evaluating the skill
    /// strains, return them as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[expect(clippy::missing_panics_doc, reason = "unreachable")]
    pub fn strains(&self, map: &Beatmap) -> Strains {
        match map.mode {
            GameMode::Osu => Strains::Osu(Osu::strains(self, map).expect("no conversion required")),
            GameMode::Taiko => {
                Strains::Taiko(Taiko::strains(self, map).expect("no conversion required"))
            }
            GameMode::Catch => {
                Strains::Catch(Catch::strains(self, map).expect("no conversion required"))
            }
            GameMode::Mania => {
                Strains::Mania(Mania::strains(self, map).expect("no conversion required"))
            }
        }
    }

    /// Perform the strain calculation for a specific [`IGameMode`].
    pub fn strains_for_mode<M: IGameMode>(
        &self,
        map: &Beatmap,
    ) -> Result<M::Strains, ConvertError> {
        M::strains(self, map)
    }

    /// Create a gradual difficulty calculator for a [`Beatmap`].
    pub fn gradual_difficulty(self, map: &Beatmap) -> GradualDifficulty {
        GradualDifficulty::new(self, map)
    }

    /// Create a gradual difficulty calculator for a [`Beatmap`] on a specific [`IGameMode`].
    pub fn gradual_difficulty_for_mode<M: IGameMode>(
        self,
        map: &Beatmap,
    ) -> Result<M::GradualDifficulty, ConvertError> {
        M::gradual_difficulty(self, map)
    }

    /// Create a gradual performance calculator for a [`Beatmap`].
    pub fn gradual_performance(self, map: &Beatmap) -> GradualPerformance {
        GradualPerformance::new(self, map)
    }

    /// Create a gradual performance calculator for a [`Beatmap`] on a specific [`IGameMode`].
    pub fn gradual_performance_for_mode<M: IGameMode>(
        self,
        map: &Beatmap,
    ) -> Result<M::GradualPerformance, ConvertError> {
        M::gradual_performance(self, map)
    }

    pub(crate) const fn get_mods(&self) -> &GameMods {
        &self.mods
    }

    pub(crate) fn get_clock_rate(&self) -> f64 {
        self.clock_rate
            .map_or(self.mods.clock_rate(), non_zero_u64_to_f64)
    }

    pub(crate) fn get_passed_objects(&self) -> usize {
        self.passed_objects.map_or(usize::MAX, |n| n as usize)
    }

    pub(crate) const fn get_ar(&self) -> Option<ModsDependent> {
        self.ar
    }

    pub(crate) const fn get_cs(&self) -> Option<ModsDependent> {
        self.cs
    }

    pub(crate) const fn get_hp(&self) -> Option<ModsDependent> {
        self.hp
    }

    pub(crate) const fn get_od(&self) -> Option<ModsDependent> {
        self.od
    }

    pub(crate) fn get_hardrock_offsets(&self) -> bool {
        self.hardrock_offsets
            .unwrap_or_else(|| self.mods.hardrock_offsets())
    }

    pub(crate) fn get_lazer(&self) -> bool {
        self.lazer.unwrap_or(true)
    }
}

const fn non_zero_u64_to_f64(n: NonZeroU64) -> f64 {
    f64::from_bits(n.get())
}

impl Debug for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self {
            mods,
            passed_objects,
            clock_rate,
            ar,
            cs,
            hp,
            od,
            hardrock_offsets,
            lazer,
        } = self;

        f.debug_struct("Difficulty")
            .field("mods", mods)
            .field("passed_objects", passed_objects)
            .field("clock_rate", &clock_rate.map(non_zero_u64_to_f64))
            .field("ar", ar)
            .field("cs", cs)
            .field("hp", hp)
            .field("od", od)
            .field("hardrock_offsets", hardrock_offsets)
            .field("lazer", lazer)
            .finish()
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::new()
    }
}
