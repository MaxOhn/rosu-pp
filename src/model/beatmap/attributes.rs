use rosu_map::section::general::GameMode;

use crate::{Difficulty, any::difficulty::ModsDependent, model::mods::GameMods};

use super::Beatmap;

/// Summary struct for a [`Beatmap`]'s attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatmapAttributes {
    /// The approach rate.
    pub ar: f64,
    /// The overall difficulty.
    pub od: f64,
    /// The circle size.
    pub cs: f64,
    /// The health drain rate
    pub hp: f64,
    /// The clock rate with respect to mods.
    pub clock_rate: f64,
    /// The hit windows for approach rate and overall difficulty.
    pub hit_windows: HitWindows,
}

/// AR and OD hit windows
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitWindows {
    /// Hit window for approach rate i.e. `TimePreempt` in milliseconds.
    pub ar: f64,
    /// Great hit window for overall difficulty i.e. time to hit a 300 ("Great") in milliseconds.
    pub od_great: f64,
    /// Ok hit window for overall difficulty i.e. time to hit a 100 ("Ok") in milliseconds.
    ///
    /// `None` for osu!mania.
    pub od_ok: Option<f64>,
    /// Meh hit window for overall difficulty i.e. time to hit a 50 ("Meh") in milliseconds.
    ///
    /// Only `Some` for osu!standard.
    pub od_meh: Option<f64>,
}

/// A builder for [`BeatmapAttributes`] and [`HitWindows`].
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct BeatmapAttributesBuilder {
    mode: GameMode,
    is_convert: bool,
    ar: ModsDependentKind,
    od: ModsDependentKind,
    cs: ModsDependentKind,
    hp: ModsDependentKind,
    mods: GameMods,
    clock_rate: Option<f64>,
}

struct GameModeHitWindows {
    min: f64,
    avg: f64,
    max: f64,
}

const OSU_GREAT: GameModeHitWindows = GameModeHitWindows {
    min: 80.0,
    avg: 50.0,
    max: 20.0,
};

const OSU_OK: GameModeHitWindows = GameModeHitWindows {
    min: 140.0,
    avg: 100.0,
    max: 60.0,
};

const OSU_MEH: GameModeHitWindows = GameModeHitWindows {
    min: 200.0,
    avg: 150.0,
    max: 100.0,
};

const TAIKO_GREAT: GameModeHitWindows = GameModeHitWindows {
    min: 50.0,
    avg: 35.0,
    max: 20.0,
};

const TAIKO_OK: GameModeHitWindows = GameModeHitWindows {
    min: 120.0,
    avg: 80.0,
    max: 50.0,
};

const AR_WINDOWS: GameModeHitWindows = GameModeHitWindows {
    min: 1800.0,
    avg: 1200.0,
    max: 450.0,
};

impl BeatmapAttributesBuilder {
    /// Create a new [`BeatmapAttributesBuilder`].
    ///
    /// The mode will be `GameMode::Osu` and attributes are set to `5.0`.
    pub const fn new() -> Self {
        Self {
            mode: GameMode::Osu,
            is_convert: false,
            ar: ModsDependentKind::DEFAULT,
            od: ModsDependentKind::DEFAULT,
            cs: ModsDependentKind::DEFAULT,
            hp: ModsDependentKind::DEFAULT,
            mods: GameMods::DEFAULT,
            clock_rate: None,
        }
    }

    /// Use the given [`Beatmap`]'s attributes, mode, and convert status.
    pub fn map(self, map: &Beatmap) -> Self {
        Self {
            mode: map.mode,
            // Clamping necessary to match lazer on maps like /b/4243836.
            ar: ModsDependentKind::Default(ModsDependent::new(map.ar.clamp(0.0, 10.0))),
            od: ModsDependentKind::Default(ModsDependent::new(map.od.clamp(0.0, 10.0))),
            cs: ModsDependentKind::Default(ModsDependent::new(map.cs)),
            hp: ModsDependentKind::Default(ModsDependent::new(map.hp)),
            is_convert: map.is_convert,
            ..self
        }
    }

    /// Specify the approach rate.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn ar(mut self, ar: f32, with_mods: bool) -> Self {
        self.ar = ModsDependentKind::Custom(ModsDependent {
            value: ar,
            with_mods,
        });

        self
    }

    /// Specify the overall difficulty.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn od(mut self, od: f32, with_mods: bool) -> Self {
        self.od = ModsDependentKind::Custom(ModsDependent {
            value: od,
            with_mods,
        });

        self
    }

    /// Specify the circle size.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn cs(mut self, cs: f32, with_mods: bool) -> Self {
        self.cs = ModsDependentKind::Custom(ModsDependent {
            value: cs,
            with_mods,
        });

        self
    }

    /// Specify the drain rate.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn hp(mut self, hp: f32, with_mods: bool) -> Self {
        self.hp = ModsDependentKind::Custom(ModsDependent {
            value: hp,
            with_mods,
        });

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
        Self {
            mode: self.mode,
            is_convert: self.is_convert,
            ar: difficulty
                .get_ar()
                .map_or(self.ar, ModsDependentKind::Custom),
            od: difficulty
                .get_od()
                .map_or(self.od, ModsDependentKind::Custom),
            cs: difficulty
                .get_cs()
                .map_or(self.cs, ModsDependentKind::Custom),
            hp: difficulty
                .get_hp()
                .map_or(self.hp, ModsDependentKind::Custom),
            mods: difficulty.get_mods().clone(),
            clock_rate: Some(difficulty.get_clock_rate()),
        }
    }

    /// Calculate the AR and OD hit windows.
    pub fn hit_windows(&self) -> HitWindows {
        let mods = &self.mods;
        let clock_rate = self.clock_rate.unwrap_or_else(|| mods.clock_rate());

        let ar_clock_rate = if self.ar.with_mods() { 1.0 } else { clock_rate };
        let od_clock_rate = if self.od.with_mods() { 1.0 } else { clock_rate };

        let mod_mult = |val: f32| {
            if mods.hr() {
                (val * 1.4).min(10.0)
            } else if mods.ez() {
                val * 0.5
            } else {
                val
            }
        };

        let raw_ar = if self.ar.with_mods() {
            self.ar.value(mods, GameMods::ar)
        } else {
            mod_mult(self.ar.value(mods, GameMods::ar))
        };

        let preempt = difficulty_range(f64::from(raw_ar), AR_WINDOWS) / ar_clock_rate;

        // OD
        let (great, ok, meh) = match self.mode {
            GameMode::Osu | GameMode::Catch => {
                let raw_od = if self.od.with_mods() {
                    self.od.value(mods, GameMods::od)
                } else {
                    mod_mult(self.od.value(mods, GameMods::od))
                };

                let great = difficulty_range(f64::from(raw_od), OSU_GREAT) / od_clock_rate;
                let ok = difficulty_range(f64::from(raw_od), OSU_OK) / od_clock_rate;
                let meh = difficulty_range(f64::from(raw_od), OSU_MEH) / od_clock_rate;

                (great, Some(ok), Some(meh))
            }
            GameMode::Taiko => {
                let raw_od = if self.od.with_mods() {
                    self.od.value(mods, GameMods::od)
                } else {
                    mod_mult(self.od.value(mods, GameMods::od))
                };

                let great = difficulty_range(f64::from(raw_od), TAIKO_GREAT) / od_clock_rate;
                let ok = difficulty_range(f64::from(raw_od), TAIKO_OK) / od_clock_rate;

                (great, Some(ok), None)
            }
            GameMode::Mania => {
                let mut value = if !self.is_convert {
                    34.0 + 3.0 * (10.0 - self.od.value(mods, GameMods::od)).clamp(0.0, 10.0)
                } else if self.od.value(mods, GameMods::od).round_ties_even() > 4.0 {
                    34.0
                } else {
                    47.0
                };

                if !self.od.with_mods() {
                    if mods.hr() {
                        value /= 1.4;
                    } else if mods.ez() {
                        value *= 1.4;
                    }
                }

                let great = ((f64::from(value) * od_clock_rate).floor() / od_clock_rate).ceil();

                (great, None, None)
            }
        };

        HitWindows {
            ar: preempt,
            od_great: great,
            od_ok: ok,
            od_meh: meh,
        }
    }

    /// Calculate the [`BeatmapAttributes`].
    pub fn build(&self) -> BeatmapAttributes {
        let mods = &self.mods;
        let clock_rate = self.clock_rate.unwrap_or_else(|| mods.clock_rate());

        // HP
        let mut hp = self.hp.value(mods, GameMods::hp);

        if !self.hp.with_mods() {
            hp *= mods.od_ar_hp_multiplier() as f32;
        }

        hp = hp.min(10.0);

        // CS
        let mut cs = self.cs.value(mods, GameMods::cs);

        if !self.cs.with_mods() {
            if mods.hr() {
                cs = (cs * 1.3).min(10.0);
            } else if mods.ez() {
                cs *= 0.5;
            }
        }

        let hit_windows = self.hit_windows();
        let HitWindows {
            ar,
            od_great,
            od_ok: _,
            od_meh: _,
        } = hit_windows;

        // AR
        let ar = if ar > 1200.0 {
            (1800.0 - ar) / 120.0
        } else {
            (1200.0 - ar) / 150.0 + 5.0
        };

        // OD
        let od = match self.mode {
            GameMode::Osu => Self::osu_great_hit_window_to_od(od_great),
            GameMode::Taiko => {
                (TAIKO_GREAT.min - od_great) / (TAIKO_GREAT.min - TAIKO_GREAT.avg) * 5.0
            }
            GameMode::Catch | GameMode::Mania => f64::from(self.od.value(mods, GameMods::od)),
        };

        BeatmapAttributes {
            ar,
            od,
            cs: f64::from(cs),
            hp: f64::from(hp),
            clock_rate,
            hit_windows,
        }
    }

    pub(crate) const fn osu_great_hit_window_to_od(hit_window: f64) -> f64 {
        (OSU_GREAT.min - hit_window) / 6.0
    }
}

impl From<&Beatmap> for BeatmapAttributesBuilder {
    fn from(map: &Beatmap) -> Self {
        Self::new().map(map)
    }
}

// False positive? Value looks consumed to me...
#[allow(clippy::needless_pass_by_value)]
fn difficulty_range(difficulty: f64, windows: GameModeHitWindows) -> f64 {
    let GameModeHitWindows { min, avg: mid, max } = windows;

    if difficulty > 5.0 {
        mid + (max - mid) * (difficulty - 5.0) / 5.0
    } else if difficulty < 5.0 {
        mid - (mid - min) * (5.0 - difficulty) / 5.0
    } else {
        mid
    }
}

impl Default for BeatmapAttributesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ModsDependentKind {
    Default(ModsDependent),
    Custom(ModsDependent),
}

impl ModsDependentKind {
    const DEFAULT: Self = Self::Default(ModsDependent::new(5.0));

    const fn with_mods(&self) -> bool {
        match self {
            ModsDependentKind::Default(inner) | ModsDependentKind::Custom(inner) => inner.with_mods,
        }
    }

    fn value(&self, mods: &GameMods, mods_fn: impl Fn(&GameMods) -> Option<f64>) -> f32 {
        match self {
            ModsDependentKind::Default(inner) => mods_fn(mods).map_or(inner.value, |n| n as f32),
            ModsDependentKind::Custom(inner) => inner.value,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use rosu_mods::{
        GameMod, GameMods,
        generated_mods::{DifficultyAdjustOsu, DoubleTimeCatch, DoubleTimeOsu, HiddenOsu},
    };

    use super::*;

    #[test]
    fn default_ar() {
        let gamemod = GameMod::HiddenOsu(HiddenOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 5.0);
    }

    #[test]
    fn custom_ar_without_mods() {
        let gamemod = GameMod::DoubleTimeOsu(DoubleTimeOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, false)
            .difficulty(&diff)
            .build();

        assert_eq!(attrs.ar, 10.0);
    }

    #[test]
    fn custom_ar_with_mods() {
        let gamemod = GameMod::DoubleTimeOsu(DoubleTimeOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, true)
            .difficulty(&diff)
            .build();

        assert_eq!(attrs.ar, 8.5);
    }

    #[test]
    fn custom_mods_ar() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(DoubleTimeCatch::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(7.0),
            ..DifficultyAdjustOsu::default()
        }));
        let diff = Difficulty::new().mods(mods);
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 9.0);
    }

    #[test]
    fn custom_ar_custom_mods_ar_without_mods() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(DoubleTimeCatch::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(9.0),
            ..DifficultyAdjustOsu::default()
        }));

        let diff = Difficulty::new().mods(mods).ar(8.5, false);
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 10.0);
    }

    #[test]
    fn custom_ar_custom_mods_ar_with_mods() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(DoubleTimeCatch::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(9.0),
            ..DifficultyAdjustOsu::default()
        }));

        let diff = Difficulty::new().mods(mods).ar(8.5, true);
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 8.5);
    }
}
