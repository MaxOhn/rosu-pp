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

#[rustfmt::skip]
macro_rules! set_attr {
    ( $short:ident = $long:literal ) => {
        #[doc = concat!(
            "Specify the ", $long,
            ". \n\n`fixed` determines if the given value should be used before \
            or after accounting for mods, e.g. on `true` the value will be used \
            as-is and on `false` it will be modified based on the mods."
        )]
        pub const fn $short(&mut self, $short: f32, fixed: bool) -> &mut Self {
            self.difficulty.$short = if fixed {
                BeatmapAttribute::Fixed($short)
            } else {
                BeatmapAttribute::Given($short)
            };

            self
        }
    };
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
    pub const fn map(&mut self, map: &Beatmap) -> &mut Self {
        self.mode = map.mode;
        self.is_convert = map.is_convert;

        self.difficulty = BeatmapDifficulty {
            // Clamping necessary to match lazer on maps like /b/4243836.
            ar: BeatmapAttribute::Value(map.ar.clamp(0.0, 10.0)),
            od: BeatmapAttribute::Value(map.od.clamp(0.0, 10.0)),
            cs: BeatmapAttribute::Value(map.cs),
            hp: BeatmapAttribute::Value(map.hp),
        };

        self
    }

    set_attr!(ar = "approach rate");

    set_attr!(od = "overall difficulty");

    set_attr!(cs = "circle size");

    set_attr!(hp = "drain rate");

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
    pub fn mods(&mut self, mods: impl Into<GameMods>) -> &mut Self {
        self.mods = mods.into();

        self
    }

    /// Specify a custom clock rate.
    pub const fn clock_rate(&mut self, clock_rate: f64) -> &mut Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Specify a [`GameMode`] and whether it's a converted map.
    pub const fn mode(&mut self, mode: GameMode, is_convert: bool) -> &mut Self {
        self.mode = mode;
        self.is_convert = is_convert;

        self
    }

    /// Specify all settings through [`Difficulty`].
    pub fn difficulty(&mut self, difficulty: &Difficulty) -> &mut Self {
        let map_diff = difficulty.get_map_difficulty();

        self.difficulty = BeatmapDifficulty {
            ar: self.difficulty.ar.overwrite(map_diff.ar),
            cs: self.difficulty.cs.overwrite(map_diff.cs),
            hp: self.difficulty.hp.overwrite(map_diff.hp),
            od: self.difficulty.od.overwrite(map_diff.od),
        };
        self.mods = difficulty.get_mods().clone();
        self.clock_rate = Some(difficulty.get_clock_rate());

        self
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
        let mut this = Self::new();
        this.map(map);

        this
    }
}

impl From<&Difficulty> for BeatmapAttributesBuilder {
    fn from(difficulty: &Difficulty) -> Self {
        let mut this = Self::new();
        this.difficulty(difficulty);

        this
    }
}
