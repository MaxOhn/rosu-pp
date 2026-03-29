use rosu_map::section::general::GameMode;

use crate::{
    Beatmap, Difficulty, GameMods,
    model::beatmap::{
        BeatmapAttributes,
        attributes::{ModStatus, attribute::BeatmapAttribute, difficulty::BeatmapDifficulty},
    },
};

/// A builder for [`BeatmapAttributes`].
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct BeatmapAttributesBuilder {
    mode: GameMode,
    is_convert: bool,
    difficulty: BeatmapDifficulty,
    mods: GameMods,
    clock_rate: Option<f64>,
}

impl BeatmapAttributesBuilder {
    /// Create a new [`BeatmapAttributesBuilder`].
    ///
    /// The mode will be `GameMode::Osu` and attributes are set to `5.0`.
    pub const fn new() -> Self {
        Self {
            mode: GameMode::Osu,
            is_convert: false,
            difficulty: BeatmapDifficulty::DEFAULT,
            mods: GameMods::DEFAULT,
            clock_rate: None,
        }
    }

    /// Use the given [`Beatmap`]'s attributes, mode, and convert status.
    pub fn map(self, map: &Beatmap) -> Self {
        Self {
            mode: map.mode,
            difficulty: BeatmapDifficulty {
                // Clamping necessary to match lazer on maps like /b/4243836.
                ar: BeatmapAttribute::Value(map.ar.clamp(0.0, 10.0)),
                od: BeatmapAttribute::Value(map.od.clamp(0.0, 10.0)),
                cs: BeatmapAttribute::Value(map.cs),
                hp: BeatmapAttribute::Value(map.hp),
            },
            is_convert: map.is_convert,
            ..self
        }
    }

    /// Specify the approach rate.
    ///
    /// `fixed` determines if the given value should be used before or after
    /// accounting for mods, e.g. on `true` the value will be used as-is and on
    /// `false` it will be modified based on the mods.
    pub const fn ar(mut self, ar: f32, fixed: bool) -> Self {
        self.difficulty.ar = if fixed {
            BeatmapAttribute::Fixed(ar)
        } else {
            BeatmapAttribute::Given(ar)
        };

        self
    }

    /// Specify the overall difficulty.
    ///
    /// `fixed` determines if the given value should be used before or after
    /// accounting for mods, e.g. on `true` the value will be used as is and on
    /// `false` it will be modified based on the mods.
    pub const fn od(mut self, od: f32, fixed: bool) -> Self {
        self.difficulty.od = if fixed {
            BeatmapAttribute::Fixed(od)
        } else {
            BeatmapAttribute::Given(od)
        };

        self
    }

    /// Specify the circle size.
    ///
    /// `fixed` determines if the given value should be used before or after
    /// accounting for mods, e.g. on `true` the value will be used as is and on
    /// `false` it will be modified based on the mods.
    pub const fn cs(mut self, cs: f32, fixed: bool) -> Self {
        self.difficulty.cs = if fixed {
            BeatmapAttribute::Fixed(cs)
        } else {
            BeatmapAttribute::Given(cs)
        };

        self
    }

    /// Specify the drain rate.
    ///
    /// `fixed` determines if the given value should be used before or after
    /// accounting for mods, e.g. on `true` the value will be used as is and on
    /// `false` it will be modified based on the mods.
    pub const fn hp(mut self, hp: f32, fixed: bool) -> Self {
        self.difficulty.hp = if fixed {
            BeatmapAttribute::Fixed(hp)
        } else {
            BeatmapAttribute::Given(hp)
        };

        self
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
    pub fn mods(mut self, mods: impl Into<GameMods>) -> Self {
        self.mods = mods.into();

        self
    }

    /// Specify a custom clock rate.
    pub const fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Specify a [`GameMode`] and whether it's a converted map.
    pub const fn mode(mut self, mode: GameMode, is_convert: bool) -> Self {
        self.mode = mode;
        self.is_convert = is_convert;

        self
    }

    /// Specify all settings through [`Difficulty`].
    pub fn difficulty(self, difficulty: &Difficulty) -> Self {
        let map_diff = difficulty.get_map_difficulty();

        Self {
            mode: self.mode,
            is_convert: self.is_convert,
            difficulty: BeatmapDifficulty {
                ar: self.difficulty.ar.overwrite(map_diff.ar),
                cs: self.difficulty.cs.overwrite(map_diff.cs),
                hp: self.difficulty.hp.overwrite(map_diff.hp),
                od: self.difficulty.od.overwrite(map_diff.od),
            },
            mods: difficulty.get_mods().clone(),
            clock_rate: Some(difficulty.get_clock_rate()),
        }
    }

    /// Calculate the [`BeatmapAttributes`].
    pub fn build(&self) -> BeatmapAttributes {
        let mods = &self.mods;

        let mut difficulty = self.difficulty.clone();
        difficulty.apply_mods(mods, self.mode);

        BeatmapAttributes {
            difficulty,
            clock_rate: self.clock_rate.unwrap_or_else(|| mods.clock_rate()),
            mod_status: ModStatus::new(mods),
            mode: self.mode,
            is_convert: self.is_convert,
            classic_and_not_v2: mods.cl() && !mods.sv2(),
        }
    }
}

impl Default for BeatmapAttributesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Beatmap> for BeatmapAttributesBuilder {
    fn from(map: &Beatmap) -> Self {
        Self::new().map(map)
    }
}

impl From<&Difficulty> for BeatmapAttributesBuilder {
    fn from(difficulty: &Difficulty) -> Self {
        Self::new().difficulty(difficulty)
    }
}
