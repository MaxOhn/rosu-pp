use std::{borrow::Cow, io, path::Path, str::FromStr};

use rosu_map::{
    LATEST_FORMAT_VERSION,
    section::{general::GameMode, hit_objects::hit_samples::HitSoundType},
};

pub use rosu_map::section::events::BreakPeriod;

use crate::{
    Difficulty, GameMods, GradualDifficulty, GradualPerformance, Performance, catch::Catch,
    mania::Mania, taiko::Taiko,
};

pub use self::{
    attributes::{BeatmapAttributes, BeatmapAttributesBuilder, HitWindows},
    decode::{BeatmapState, ParseBeatmapError},
    suspicious::TooSuspicious,
};

use super::{
    control_point::{
        DifficultyPoint, EffectPoint, TimingPoint, difficulty_point_at, effect_point_at,
        timing_point_at,
    },
    hit_object::HitObject,
    mode::ConvertError,
};

mod attributes;
mod bpm;
mod decode;
mod suspicious;

/// All beatmap data that is relevant for difficulty and performance
/// calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct Beatmap {
    pub version: i32,
    pub is_convert: bool,

    // General
    pub stack_leniency: f32,
    pub mode: GameMode,

    // Difficulty
    pub ar: f32,
    pub cs: f32,
    pub hp: f32,
    pub od: f32,
    pub slider_multiplier: f64,
    pub slider_tick_rate: f64,

    // Events
    pub breaks: Vec<BreakPeriod>,

    // TimingPoints
    pub timing_points: Vec<TimingPoint>,
    pub difficulty_points: Vec<DifficultyPoint>,
    pub effect_points: Vec<EffectPoint>,

    // HitObjects
    pub hit_objects: Vec<HitObject>,
    pub hit_sounds: Vec<HitSoundType>,
}

impl Beatmap {
    /// Parse a [`Beatmap`] by providing a path to a `.osu` file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        rosu_map::from_path(path)
    }

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// slice of bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, io::Error> {
        rosu_map::from_bytes(bytes)
    }

    /// Returns a [`BeatmapAttributesBuilder`] to calculate modified beatmap
    /// attributes.
    pub fn attributes(&self) -> BeatmapAttributesBuilder {
        BeatmapAttributesBuilder::new().map(self)
    }

    /// The beats per minute of the map.
    pub fn bpm(&self) -> f64 {
        bpm::bpm(self.hit_objects.last(), &self.timing_points)
    }

    /// Create a performance calculator for this [`Beatmap`].
    pub fn performance(&self) -> Performance<'_> {
        Performance::new(self)
    }

    /// Create a gradual difficulty calculator for this [`Beatmap`].
    pub fn gradual_difficulty(&self, difficulty: Difficulty) -> GradualDifficulty {
        GradualDifficulty::new(difficulty, self)
    }

    /// Create a gradual performance calculator for this [`Beatmap`].
    pub fn gradual_performance(&self, difficulty: Difficulty) -> GradualPerformance {
        GradualPerformance::new(difficulty, self)
    }

    /// Finds the [`TimingPoint`] that is active at the given time.
    pub(crate) fn timing_point_at(&self, time: f64) -> Option<&TimingPoint> {
        timing_point_at(&self.timing_points, time)
    }

    /// Finds the [`DifficultyPoint`] that is active at the given time.
    pub(crate) fn difficulty_point_at(&self, time: f64) -> Option<&DifficultyPoint> {
        difficulty_point_at(&self.difficulty_points, time)
    }

    /// Finds the [`EffectPoint`] that is active at the given time.
    pub(crate) fn effect_point_at(&self, time: f64) -> Option<&EffectPoint> {
        effect_point_at(&self.effect_points, time)
    }

    /// Sum up the duration of all breaks (in milliseconds).
    pub fn total_break_time(&self) -> f64 {
        self.breaks.iter().map(BreakPeriod::duration).sum()
    }

    /// Attempt to convert a [`Beatmap`] to the specified mode.
    pub fn convert(mut self, mode: GameMode, mods: &GameMods) -> Result<Self, ConvertError> {
        self.convert_mut(mode, mods)?;

        Ok(self)
    }

    /// Attempt to convert a [`&Beatmap`] to the specified mode.
    ///
    /// [`&Beatmap`]: Beatmap
    pub fn convert_ref(
        &self,
        mode: GameMode,
        mods: &GameMods,
    ) -> Result<Cow<'_, Self>, ConvertError> {
        if self.mode == mode {
            return Ok(Cow::Borrowed(self));
        } else if self.is_convert {
            return Err(ConvertError::AlreadyConverted);
        } else if self.mode != GameMode::Osu {
            return Err(ConvertError::Convert {
                from: self.mode,
                to: mode,
            });
        }

        let mut map = self.to_owned();

        match mode {
            GameMode::Taiko => Taiko::convert(&mut map),
            GameMode::Catch => Catch::convert(&mut map),
            GameMode::Mania => Mania::convert(&mut map, mods),
            GameMode::Osu => unreachable!(),
        }

        Ok(Cow::Owned(map))
    }

    /// Attempt to convert a [`&mut Beatmap`] to the specified mode.
    ///
    /// [`&mut Beatmap`]: Beatmap
    pub fn convert_mut(&mut self, mode: GameMode, mods: &GameMods) -> Result<(), ConvertError> {
        if self.mode == mode {
            return Ok(());
        } else if self.is_convert {
            return Err(ConvertError::AlreadyConverted);
        } else if self.mode != GameMode::Osu {
            return Err(ConvertError::Convert {
                from: self.mode,
                to: mode,
            });
        }

        match mode {
            GameMode::Taiko => Taiko::convert(self),
            GameMode::Catch => Catch::convert(self),
            GameMode::Mania => Mania::convert(self, mods),
            GameMode::Osu => unreachable!(),
        }

        Ok(())
    }

    /// Check whether hitobjects appear too suspicious for further calculation.
    ///
    /// Sometimes a [`Beatmap`] isn't created for gameplay but rather to test
    /// the limits of osu! itself. Difficulty- and/or performance calculation
    /// should likely be avoided on these maps due to potential performance
    /// issues.
    pub fn check_suspicion(&self) -> Result<(), TooSuspicious> {
        match TooSuspicious::new(self) {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }
}

impl FromStr for Beatmap {
    type Err = io::Error;

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        rosu_map::from_str(s)
    }
}

const DEFAULT_SLIDER_LENIENCY: f32 = 0.7;

impl Default for Beatmap {
    fn default() -> Self {
        Self {
            version: LATEST_FORMAT_VERSION,
            is_convert: false,
            stack_leniency: DEFAULT_SLIDER_LENIENCY,
            mode: GameMode::default(),
            ar: 5.0,
            cs: 5.0,
            hp: 5.0,
            od: 5.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,
            breaks: Vec::default(),
            timing_points: Vec::default(),
            difficulty_points: Vec::default(),
            effect_points: Vec::default(),
            hit_objects: Vec::default(),
            hit_sounds: Vec::default(),
        }
    }
}
