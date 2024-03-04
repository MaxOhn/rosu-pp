use std::{borrow::Cow, num::NonZeroU32};

use rosu_map::section::general::GameMode;

use crate::{
    catch::Catch,
    mania::Mania,
    model::beatmap::{Beatmap, Converted},
    osu::Osu,
    taiko::Taiko,
    GradualDifficulty, GradualPerformance,
};

use self::converted::ConvertedDifficulty;

use super::{attributes::DifficultyAttributes, Strains};

pub mod converted;
pub mod gradual;
pub mod object;
pub mod skills;

use crate::{model::mode::IGameMode, util::mods::Mods};

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
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct Difficulty {
    mods: u32,
    passed_objects: Option<u32>,
    /// Clock rate will be clamped internally between 0.01 and 100.0.
    ///
    /// Since its minimum value is 0.01, its bits are never zero.
    /// Additionally, values between 0.01 and 100 are represented sufficiently
    /// precise with 32 bits.
    ///
    /// This allows for an optimization to reduce the struct size by storing its
    /// bits as a [`NonZeroU32`].
    clock_rate: Option<NonZeroU32>,
    ar: Option<ModsDependent>,
    cs: Option<ModsDependent>,
    hp: Option<ModsDependent>,
    od: Option<ModsDependent>,
    hardrock_offsets: Option<bool>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ModsDependent {
    pub value: f32,
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
            mods: 0,
            passed_objects: None,
            clock_rate: None,
            ar: None,
            cs: None,
            hp: None,
            od: None,
            hardrock_offsets: None,
        }
    }

    /// Use this [`Difficulty`] as a calculator for a specific [`IGameMode`].
    ///
    /// Note that [`ConvertedDifficulty`] won't allow to further customize
    /// fields so be sure they're all set before converting to it.
    pub const fn with_mode<M: IGameMode>(&self) -> ConvertedDifficulty<'_, M> {
        ConvertedDifficulty::new(self)
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    pub const fn mods(self, mods: u32) -> Self {
        Self { mods, ..self }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    pub const fn passed_objects(self, passed_objects: u32) -> Self {
        Self {
            passed_objects: Some(passed_objects),
            ..self
        }
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
        let clock_rate = (clock_rate as f32).clamp(0.01, 100.0).to_bits();

        // SAFETY: The minimum value is 0.01 so its bits can never be fully
        // zero.
        let non_zero = unsafe { NonZeroU32::new_unchecked(clock_rate) };

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
    pub const fn hardrock_offsets(self, hardrock_offsets: bool) -> Self {
        Self {
            hardrock_offsets: Some(hardrock_offsets),
            ..self
        }
    }

    /// Perform the difficulty calculation.
    pub fn calculate(&self, map: &Beatmap) -> DifficultyAttributes {
        let map = Cow::Borrowed(map);

        match map.mode {
            GameMode::Osu => DifficultyAttributes::Osu(Osu::difficulty(self, &Converted::new(map))),
            GameMode::Taiko => {
                DifficultyAttributes::Taiko(Taiko::difficulty(self, &Converted::new(map)))
            }
            GameMode::Catch => {
                DifficultyAttributes::Catch(Catch::difficulty(self, &Converted::new(map)))
            }
            GameMode::Mania => {
                DifficultyAttributes::Mania(Mania::difficulty(self, &Converted::new(map)))
            }
        }
    }

    /// Perform the difficulty calculation but instead of evaluating the skill
    /// strains, return them as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    pub fn strains(&self, map: &Beatmap) -> Strains {
        let map = Cow::Borrowed(map);

        match map.mode {
            GameMode::Osu => Strains::Osu(Osu::strains(self, &Converted::new(map))),
            GameMode::Taiko => Strains::Taiko(Taiko::strains(self, &Converted::new(map))),
            GameMode::Catch => Strains::Catch(Catch::strains(self, &Converted::new(map))),
            GameMode::Mania => Strains::Mania(Mania::strains(self, &Converted::new(map))),
        }
    }

    /// Create a gradual difficulty calculator for a [`Beatmap`].
    pub fn gradual_difficulty(self, map: &Beatmap) -> GradualDifficulty {
        GradualDifficulty::new(self, map)
    }

    /// Create a gradual performance calculator for a [`Beatmap`].
    pub fn gradual_performance(self, map: &Beatmap) -> GradualPerformance {
        GradualPerformance::new(self, map)
    }

    pub(crate) const fn get_mods(&self) -> u32 {
        self.mods
    }

    pub(crate) fn get_clock_rate(&self) -> f64 {
        self.clock_rate
            .map(NonZeroU32::get)
            .map(u64::from)
            .map_or_else(|| self.mods.clock_rate(), f64::from_bits)
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
        self.hardrock_offsets.unwrap_or(self.mods.hr())
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::new()
    }
}
