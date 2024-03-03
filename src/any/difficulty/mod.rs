use std::borrow::Cow;

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
#[derive(Clone, Debug, Default, PartialEq)]
#[must_use]
pub struct Difficulty {
    mods: u32,
    passed_objects: Option<u32>,
    clock_rate: Option<f64>,
}

impl Difficulty {
    /// Create a new difficulty calculator.
    pub const fn new() -> Self {
        Self {
            mods: 0,
            passed_objects: None,
            clock_rate: None,
        }
    }

    /// Use this [`&Difficulty`] as a calculator for a specific [`IGameMode`].
    ///
    /// [`&Difficulty`]: Difficulty
    pub const fn with_mode<M: IGameMode>(&self) -> ConvertedDifficulty<M> {
        let this = Self {
            mods: self.mods,
            passed_objects: self.passed_objects,
            clock_rate: self.clock_rate,
        };

        ConvertedDifficulty::from_difficulty(this)
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

    /// Adjust the clock rate used in the calculation between 0.01 and 100.0.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        Self {
            clock_rate: Some(clock_rate.clamp(0.01, 100.0)),
            ..self
        }
    }

    /// Perform the difficulty calculation.
    ///
    /// The returned attributes depend on the map's mode.
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
    pub fn gradual_difficulty(&self, map: &Beatmap) -> GradualDifficulty {
        GradualDifficulty::new(self, map)
    }

    /// Create a gradual performance calculator for a [`Beatmap`].
    pub fn gradual_performance(&self, map: &Beatmap) -> GradualPerformance {
        GradualPerformance::new(self, map)
    }

    pub(crate) const fn get_mods(&self) -> u32 {
        self.mods
    }

    pub(crate) fn get_clock_rate(&self) -> f64 {
        self.clock_rate.unwrap_or_else(|| self.mods.clock_rate())
    }

    pub(crate) fn get_passed_objects(&self) -> usize {
        self.passed_objects.map_or(usize::MAX, |n| n as usize)
    }
}
