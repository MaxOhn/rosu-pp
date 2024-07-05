use rosu_map::section::general::GameMode;

use crate::{any::difficulty::ModsDependent, model::mods::GameMods, Difficulty};

use super::{converted::Converted, Beatmap};

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
    /// Hit window for overall difficulty i.e. time to hit a 300 ("Great") in milliseconds.
    pub od: f64,
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

impl BeatmapAttributesBuilder {
    const OSU_MIN: f64 = 80.0;
    const OSU_AVG: f64 = 50.0;
    const OSU_MAX: f64 = 20.0;

    const TAIKO_MIN: f64 = 50.0;
    const TAIKO_AVG: f64 = 35.0;
    const TAIKO_MAX: f64 = 20.0;

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
            ar: ModsDependentKind::Default(ModsDependent::new(map.ar)),
            od: ModsDependentKind::Default(ModsDependent::new(map.od)),
            cs: ModsDependentKind::Default(ModsDependent::new(map.cs)),
            hp: ModsDependentKind::Default(ModsDependent::new(map.hp)),
            mods: GameMods::DEFAULT,
            clock_rate: None,
            is_convert: map.is_convert,
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

    /// Specify the mods.
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

        let clock_rate = self
            .clock_rate
            .unwrap_or_else(|| f64::from(mods.clock_rate()));

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

        let preempt = difficulty_range(f64::from(raw_ar), 1800.0, 1200.0, 450.0) / ar_clock_rate;

        // OD
        let hit_window = match self.mode {
            GameMode::Osu | GameMode::Catch => {
                let raw_od = if self.od.with_mods() {
                    self.od.value(mods, GameMods::od)
                } else {
                    mod_mult(self.od.value(mods, GameMods::od))
                };

                difficulty_range(
                    f64::from(raw_od),
                    Self::OSU_MIN,
                    Self::OSU_AVG,
                    Self::OSU_MAX,
                ) / od_clock_rate
            }
            GameMode::Taiko => {
                let raw_od = if self.od.with_mods() {
                    self.od.value(mods, GameMods::od)
                } else {
                    mod_mult(self.od.value(mods, GameMods::od))
                };

                let diff_range = difficulty_range(
                    f64::from(raw_od),
                    Self::TAIKO_MIN,
                    Self::TAIKO_AVG,
                    Self::TAIKO_MAX,
                );

                diff_range / od_clock_rate
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

                ((f64::from(value) * od_clock_rate).floor() / od_clock_rate).ceil()
            }
        };

        HitWindows {
            ar: preempt,
            od: hit_window,
        }
    }

    /// Calculate the [`BeatmapAttributes`].
    pub fn build(&self) -> BeatmapAttributes {
        let mods = &self.mods;
        let clock_rate = self
            .clock_rate
            .unwrap_or_else(|| f64::from(mods.clock_rate()));

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
        let HitWindows { ar, od } = hit_windows;

        // AR
        let ar = if ar > 1200.0 {
            (1800.0 - ar) / 120.0
        } else {
            (1200.0 - ar) / 150.0 + 5.0
        };

        // OD
        let od = match self.mode {
            GameMode::Osu => (Self::OSU_MIN - od) / 6.0,
            GameMode::Taiko => (Self::TAIKO_MIN - od) / (Self::TAIKO_MIN - Self::TAIKO_AVG) * 5.0,
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
}

impl From<&Beatmap> for BeatmapAttributesBuilder {
    fn from(map: &Beatmap) -> Self {
        Self::new().map(map)
    }
}

impl<M> From<&Converted<'_, M>> for BeatmapAttributesBuilder {
    fn from(converted: &Converted<'_, M>) -> Self {
        Self::new().map(converted)
    }
}

fn difficulty_range(difficulty: f64, min: f64, mid: f64, max: f64) -> f64 {
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

    fn value(&self, mods: &GameMods, mods_fn: impl Fn(&GameMods) -> Option<f32>) -> f32 {
        match self {
            ModsDependentKind::Default(inner) => mods_fn(mods).unwrap_or(inner.value),
            ModsDependentKind::Custom(inner) => inner.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use rosu_mods::{generated_mods::DifficultyAdjustOsu, GameMod, GameMods};

    use super::*;

    #[test]
    fn default_ar() {
        let gamemod = GameMod::HiddenOsu(Default::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 5.0);
    }

    #[test]
    fn custom_ar_without_mods() {
        let gamemod = GameMod::DoubleTimeOsu(Default::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, false)
            .difficulty(&diff)
            .build();

        assert_eq!(attrs.ar, 10.0);
    }

    #[test]
    fn custom_ar_with_mods() {
        let gamemod = GameMod::DoubleTimeOsu(Default::default());
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
        mods.insert(GameMod::DoubleTimeCatch(Default::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(7.0),
            ..Default::default()
        }));
        let diff = Difficulty::new().mods(mods);
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 9.0);
    }

    #[test]
    fn custom_ar_custom_mods_ar_without_mods() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(Default::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(9.0),
            ..Default::default()
        }));

        let diff = Difficulty::new().mods(mods).ar(8.5, false);
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 10.0);
    }

    #[test]
    fn custom_ar_custom_mods_ar_with_mods() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(Default::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(9.0),
            ..Default::default()
        }));

        let diff = Difficulty::new().mods(mods).ar(8.5, true);
        let attrs = BeatmapAttributesBuilder::new().difficulty(&diff).build();

        assert_eq!(attrs.ar, 8.5);
    }
}
