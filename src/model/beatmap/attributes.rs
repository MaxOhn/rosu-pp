use rosu_map::section::general::GameMode;

use crate::{
    Difficulty,
    any::difficulty::ModsDependent,
    model::mods::GameMods,
    util::{float_ext::FloatExt, ruleset_ext::PeppyStarsBeatmapAttributes},
};

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

struct BeatmapAttributesExt;

impl BeatmapAttributesExt {
    fn difficulty_range(difficulty: f64, min: f64, mid: f64, max: f64) -> f64 {
        if difficulty > 5.0 {
            mid + (max - mid) * Self::difficulty_range_value(difficulty)
        } else if difficulty < 5.0 {
            mid + (mid - min) * Self::difficulty_range_value(difficulty)
        } else {
            mid
        }
    }

    fn difficulty_range_value(difficulty: f64) -> f64 {
        (difficulty - 5.0) / 5.0
    }

    fn inverse_difficulty_range(difficulty_value: f64, diff0: f64, diff5: f64, diff10: f64) -> f64 {
        if FloatExt::eq(
            f64::signum(difficulty_value - diff5),
            f64::signum(diff10 - diff5),
        ) {
            (difficulty_value - diff5) / (diff10 - diff5) * 5.0 + 5.0
        } else {
            (difficulty_value - diff5) / (diff5 - diff0) * 5.0 + 5.0
        }
    }
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

        let raw_ar = {
            let ar = self.ar.value(mods, GameMods::ar);

            if self.ar.with_mods() {
                ar
            } else {
                apply_mods_mult(mods, ar)
            }
        };

        let preempt = hit_windows::AR.difficulty_range(f64::from(raw_ar)) / ar_clock_rate;

        // OD
        let (great, ok, meh) = match self.mode {
            GameMode::Osu | GameMode::Catch => {
                let raw_od = {
                    let od = self.od.value(mods, GameMods::od);

                    if self.od.with_mods() {
                        od
                    } else {
                        apply_mods_mult(mods, od)
                    }
                };

                let raw_od = f64::from(raw_od);

                let great = (hit_windows::osu::GREAT.difficulty_range(raw_od).floor() - 0.5)
                    / od_clock_rate;
                let ok =
                    (hit_windows::osu::OK.difficulty_range(raw_od).floor() - 0.5) / od_clock_rate;
                let meh =
                    (hit_windows::osu::MEH.difficulty_range(raw_od).floor() - 0.5) / od_clock_rate;

                (great, Some(ok), Some(meh))
            }
            GameMode::Taiko => {
                let raw_od = {
                    let od = self.od.value(mods, GameMods::od);

                    if self.od.with_mods() {
                        od
                    } else {
                        apply_mods_mult(mods, od)
                    }
                };

                let raw_od = f64::from(raw_od);

                let great = (hit_windows::taiko::GREAT.difficulty_range(raw_od).floor() - 0.5)
                    / od_clock_rate;
                let ok =
                    (hit_windows::taiko::OK.difficulty_range(raw_od).floor() - 0.5) / od_clock_rate;

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

    /// Calculate the [`PeppyStarsBeatmapAttributes`].
    ///
    /// Importantly, the OD value will not consider adjusted clock rates.
    ///
    /// # Panics
    ///
    /// Panics if the mode is not [`GameMode::Osu`].
    pub(crate) fn build_peppy_stars(self) -> PeppyStarsBeatmapAttributes {
        debug_assert_eq!(self.mode, GameMode::Osu);

        let mut raw_od = self.od.value(&self.mods, GameMods::od);

        if !self.od.with_mods() {
            raw_od = apply_mods_mult(&self.mods, raw_od);
        }

        PeppyStarsBeatmapAttributes {
            cs: self.build_cs(),
            hp: self.build_hp(),
            od: f64::from(raw_od),
        }
    }

    /// Calculate the [`BeatmapAttributes`].
    pub fn build(&self) -> BeatmapAttributes {
        let mods = &self.mods;
        let clock_rate = self.clock_rate.unwrap_or_else(|| mods.clock_rate());

        let hit_windows = self.hit_windows();

        let od = match self.mode {
            GameMode::Osu => Self::osu_great_hit_window_to_od(hit_windows.od_great),
            GameMode::Taiko => {
                (hit_windows::taiko::GREAT.min - hit_windows.od_great)
                    / (hit_windows::taiko::GREAT.min - hit_windows::taiko::GREAT.mid)
                    * 5.0
            }
            GameMode::Catch => 5.0, // osu!catch has no OD anyway
            GameMode::Mania => {
                let raw_od = self.od.value(mods, GameMods::od);
                let mut perfect_hit_window =
                    hit_windows::mania::PERFECT.difficulty_range(f64::from(raw_od));

                if mods.hr() {
                    perfect_hit_window /= 1.4;
                } else if mods.ez() {
                    perfect_hit_window /= 1.0 / 1.4;
                }

                hit_windows::mania::PERFECT.inverse_difficulty_range(perfect_hit_window)
            }
        };

        BeatmapAttributes {
            ar: hit_windows::AR.inverse_difficulty_range(hit_windows.ar),
            od,
            cs: self.build_cs(),
            hp: self.build_hp(),
            clock_rate,
            hit_windows,
        }
    }

    pub(crate) const fn osu_great_hit_window_to_od(hit_window: f64) -> f64 {
        (79.5 - hit_window) / 6.0
    }

    fn build_cs(&self) -> f64 {
        let mut cs = self.cs.value(&self.mods, GameMods::cs);

        if !self.cs.with_mods() {
            if self.mods.hr() {
                cs = (cs * 1.3).min(10.0);
            } else if self.mods.ez() {
                cs *= 0.5;
            }
        }

        f64::from(cs)
    }

    fn build_hp(&self) -> f64 {
        let mut hp = self.hp.value(&self.mods, GameMods::hp);

        if !self.hp.with_mods() {
            hp *= self.mods.od_ar_hp_multiplier() as f32;
        }

        f64::from(hp.min(10.0))
    }
}

impl From<&Beatmap> for BeatmapAttributesBuilder {
    fn from(map: &Beatmap) -> Self {
        Self::new().map(map)
    }
}

fn apply_mods_mult(mods: &GameMods, od: f32) -> f32 {
    if mods.hr() {
        (od * 1.4).min(10.0)
    } else if mods.ez() {
        od * 0.5
    } else {
        od
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

mod hit_windows {
    use super::BeatmapAttributesExt;

    pub(super) struct GameModeHitWindows {
        pub min: f64,
        pub mid: f64,
        pub max: f64,
    }

    impl GameModeHitWindows {
        pub fn difficulty_range(&self, difficulty: f64) -> f64 {
            let Self { min, mid, max } = *self;

            BeatmapAttributesExt::difficulty_range(difficulty, min, mid, max)
        }

        pub fn inverse_difficulty_range(&self, difficulty_value: f64) -> f64 {
            let Self { min, mid, max } = *self;

            BeatmapAttributesExt::inverse_difficulty_range(difficulty_value, min, mid, max)
        }
    }

    pub mod osu {
        use super::GameModeHitWindows;

        pub const GREAT: GameModeHitWindows = GameModeHitWindows {
            min: 80.0,
            mid: 50.0,
            max: 20.0,
        };

        pub const OK: GameModeHitWindows = GameModeHitWindows {
            min: 140.0,
            mid: 100.0,
            max: 60.0,
        };

        pub const MEH: GameModeHitWindows = GameModeHitWindows {
            min: 200.0,
            mid: 150.0,
            max: 100.0,
        };
    }

    pub mod taiko {
        use super::GameModeHitWindows;

        pub const GREAT: GameModeHitWindows = GameModeHitWindows {
            min: 50.0,
            mid: 35.0,
            max: 20.0,
        };

        pub const OK: GameModeHitWindows = GameModeHitWindows {
            min: 120.0,
            mid: 80.0,
            max: 50.0,
        };
    }

    pub mod mania {
        use super::GameModeHitWindows;

        pub const PERFECT: GameModeHitWindows = GameModeHitWindows {
            min: 22.4,
            mid: 19.4,
            max: 13.9,
        };
    }

    pub const AR: GameModeHitWindows = GameModeHitWindows {
        min: 1800.0,
        mid: 1200.0,
        max: 450.0,
    };
}

#[cfg(test)]
mod tests {
    #![expect(clippy::float_cmp, reason = "we're just testing here")]

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
