use rosu_map::section::general::GameMode;
use rosu_mods::GameMod;

use crate::{
    Difficulty,
    any::difficulty::ModsDependent,
    model::{beatmap::attributes::hit_windows::GameModeHitWindows, mods::GameMods},
    util::float_ext::FloatExt,
};

use super::Beatmap;

/// Summary struct for a [`Beatmap`]'s attributes.
///
/// Clock rate is *not* considered in attribute values.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatmapAttributes {
    difficulty: BeatmapDifficulty,
    mode: GameMode,
    clock_rate: f64,
    is_convert: bool,
    classic_and_not_v2: bool,
    mod_status: ModStatus,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ModStatus {
    Neither,
    Easy,
    HardRock,
}

impl ModStatus {
    fn new(mods: &GameMods) -> Self {
        if mods.hr() {
            Self::HardRock
        } else if mods.ez() {
            Self::Easy
        } else {
            Self::Neither
        }
    }
}

impl BeatmapAttributes {
    /// The approach rate.
    pub fn ar(&self) -> f32 {
        match self.difficulty.ar {
            BeatmapAttribute::Given(value) | BeatmapAttribute::Value(value) => value,
            // TODO: test
            BeatmapAttribute::Fixed(fixed) => match self.mode {
                GameMode::Osu | GameMode::Catch => hit_windows::AR.inverse_difficulty_range(
                    hit_windows::AR.difficulty_range(f64::from(fixed)) * self.clock_rate,
                ) as f32,
                GameMode::Taiko | GameMode::Mania => fixed,
            },
        }
    }

    /// The overall difficulty.
    pub fn od(&self) -> f32 {
        match self.difficulty.od {
            BeatmapAttribute::Given(value) | BeatmapAttribute::Value(value) => value,
            // TODO: test
            BeatmapAttribute::Fixed(fixed) => match self.mode {
                GameMode::Osu => hit_windows::osu::GREAT.inverse_difficulty_range(
                    hit_windows::osu::GREAT.difficulty_range(f64::from(fixed)) * self.clock_rate,
                ) as f32,
                GameMode::Taiko => hit_windows::taiko::GREAT.inverse_difficulty_range(
                    hit_windows::taiko::GREAT.difficulty_range(f64::from(fixed)) * self.clock_rate,
                ) as f32,
                GameMode::Mania => {
                    let factor = match self.mod_status {
                        ModStatus::Neither => 1.0,
                        ModStatus::Easy => 1.0 / 1.4,
                        ModStatus::HardRock => 1.4,
                    };

                    hit_windows::mania::PERFECT.inverse_difficulty_range(
                        hit_windows::mania::PERFECT.difficulty_range(f64::from(fixed)) * factor,
                    ) as f32
                }
                GameMode::Catch => fixed,
            },
        }
    }

    /// The circle size.
    pub const fn cs(&self) -> f32 {
        self.difficulty.cs.get_raw()
    }

    /// The health drain rate.
    pub const fn hp(&self) -> f32 {
        self.difficulty.hp.get_raw()
    }

    /// The clock rate.
    pub const fn clock_rate(&self) -> f64 {
        self.clock_rate
    }

    /// Calculate the AR and OD hit windows.
    pub fn hit_windows(&self) -> HitWindows {
        let clock_rate = self.clock_rate;

        // Same for osu! and osu!catch (?)
        let ar = || match self.difficulty.ar {
            BeatmapAttribute::Value(value) | BeatmapAttribute::Given(value) => {
                hit_windows::AR.difficulty_range(f64::from(value)) / clock_rate
            }
            BeatmapAttribute::Fixed(fixed) => hit_windows::AR.difficulty_range(f64::from(fixed)),
        };

        // Same for osu! and osu!taiko.
        // See `{OsuHitWindows,TaikoHitWindows}.SetDifficulty`
        let set_difficulty = |hit_windows: &GameModeHitWindows| match self.difficulty.od {
            BeatmapAttribute::Value(value) | BeatmapAttribute::Given(value) => {
                (f64::floor(hit_windows.difficulty_range(f64::from(value))) - 0.5) / clock_rate
            }
            BeatmapAttribute::Fixed(fixed) => {
                //     Fixed           = f^-1(f(Value) / C)
                // <=> f(Fixed)        = f(Value) / C
                // <=> f(Fixed) * C    = f(Value)
                let f_value = hit_windows.difficulty_range(f64::from(fixed)) * clock_rate;

                (f64::floor(f_value) - 0.5) / clock_rate
            }
        };

        match self.mode {
            GameMode::Osu => HitWindows {
                ar: Some(ar()),
                od_great: Some(set_difficulty(&hit_windows::osu::GREAT)),
                od_ok: Some(set_difficulty(&hit_windows::osu::OK)),
                od_meh: Some(set_difficulty(&hit_windows::osu::MEH)),
                ..Default::default()
            },
            GameMode::Taiko => HitWindows {
                od_great: Some(set_difficulty(&hit_windows::taiko::GREAT)),
                od_ok: Some(set_difficulty(&hit_windows::taiko::OK)),
                ..Default::default()
            },
            GameMode::Catch => HitWindows {
                ar: Some(ar()),
                ..Default::default()
            },
            GameMode::Mania => {
                let speed_multiplier: f64 = 1.0;
                let difficulty_multiplier: f64 = 1.0;
                let total_multiplier = speed_multiplier / difficulty_multiplier;

                // Clock rate is irrelevant, apparently
                let od = f64::from(self.difficulty.od.get_raw());

                let (perfect, great, good, ok, meh) = if self.classic_and_not_v2 {
                    if self.is_convert {
                        (
                            f64::floor(16.0 * total_multiplier) + 0.5,
                            f64::floor(
                                (if f64::round_ties_even(od) > 4.0 {
                                    34.0
                                } else {
                                    47.0
                                }) * total_multiplier,
                            ) + 0.5,
                            f64::floor(
                                (if f64::round_ties_even(od) > 4.0 {
                                    67.0
                                } else {
                                    77.0
                                }) * total_multiplier,
                            ) + 0.5,
                            f64::floor(97.0 * total_multiplier) + 0.5,
                            f64::floor(121.0 * total_multiplier) + 0.5,
                        )
                    } else {
                        let inverted_od = f64::clamp(10.0 - od, 0.0, 10.0);

                        let hit_window = |add: f64| {
                            f64::floor((add + 3.0 * inverted_od) * total_multiplier) + 0.5
                        };

                        (
                            f64::floor(16.0 * total_multiplier) + 0.5,
                            hit_window(34.0),
                            hit_window(67.0),
                            hit_window(97.0),
                            hit_window(121.0),
                        )
                    }
                } else {
                    let hit_window = |hit_windows: &GameModeHitWindows| {
                        f64::floor(hit_windows.difficulty_range(od) * total_multiplier) + 0.5
                    };

                    (
                        hit_window(&hit_windows::mania::PERFECT),
                        hit_window(&hit_windows::mania::GREAT),
                        hit_window(&hit_windows::mania::GOOD),
                        hit_window(&hit_windows::mania::OK),
                        hit_window(&hit_windows::mania::MEH),
                    )
                };

                HitWindows {
                    ar: None,
                    od_perfect: Some(perfect),
                    od_great: Some(great),
                    od_good: Some(good),
                    od_ok: Some(ok),
                    od_meh: Some(meh),
                }
            }
        }
    }

    /// Convert [`BeatmapAttributes`] into [`AdjustedBeatmapAttributes`] by
    /// applying the clock rate to the attribute values.
    pub fn apply_clock_rate(&self) -> AdjustedBeatmapAttributes {
        let clock_rate = self.clock_rate;

        let (ar, od) = match self.mode {
            GameMode::Osu => {
                let ar = self.difficulty.ar.map_or_else(f64::from, |ar| {
                    let mut preempt = hit_windows::AR.difficulty_range(f64::from(ar));
                    preempt /= clock_rate;

                    hit_windows::AR.inverse_difficulty_range(preempt)
                });

                let od = self.difficulty.od.map_or_else(f64::from, |od| {
                    let mut great_hit_window =
                        hit_windows::osu::GREAT.difficulty_range(f64::from(od));
                    great_hit_window /= clock_rate;

                    hit_windows::osu::GREAT.inverse_difficulty_range(great_hit_window)
                });

                (ar, od)
            }
            GameMode::Taiko => {
                let od = self.difficulty.od.map_or_else(f64::from, |od| {
                    let mut great_hit_window =
                        hit_windows::taiko::GREAT.difficulty_range(f64::from(od));
                    great_hit_window /= clock_rate;

                    hit_windows::taiko::GREAT.inverse_difficulty_range(great_hit_window)
                });

                (f64::from(self.difficulty.ar.get_raw()), od)
            }
            GameMode::Catch => {
                let ar = self.difficulty.ar.map_or_else(f64::from, |ar| {
                    let mut preempt = hit_windows::AR.difficulty_range(f64::from(ar));
                    preempt /= clock_rate;

                    hit_windows::AR.inverse_difficulty_range(preempt)
                });

                (ar, f64::from(self.difficulty.od.get_raw()))
            }
            GameMode::Mania => {
                let od = self.difficulty.od.map_or_else(f64::from, |od| {
                    let mut perfect_hit_window =
                        hit_windows::mania::PERFECT.difficulty_range(f64::from(od));

                    match self.mod_status {
                        ModStatus::Neither => {}
                        ModStatus::Easy => perfect_hit_window /= 1.0 / 1.4,
                        ModStatus::HardRock => perfect_hit_window /= 1.4,
                    }

                    hit_windows::mania::PERFECT.inverse_difficulty_range(perfect_hit_window)
                });

                // Ignoring CS

                (f64::from(self.difficulty.ar.get_raw()), od)
            }
        };

        AdjustedBeatmapAttributes {
            ar,
            cs: self.difficulty.cs.get_raw(),
            hp: self.difficulty.hp.get_raw(),
            od,
            clock_rate,
        }
    }
}

/// Summary struct for a [`Beatmap`]'s attributes.
///
/// The difference between this and [`BeatmapAttributes`] is that this struct
/// considers the clock rate in its attribute values.
#[derive(Clone, Debug, PartialEq)]
pub struct AdjustedBeatmapAttributes {
    /// The approach rate.
    pub ar: f64,
    /// The circle size.
    pub cs: f32,
    /// The health drain rate.
    pub hp: f32,
    /// The overall difficulty.
    pub od: f64,
    // The clock rate.
    pub clock_rate: f64,
}

pub(crate) struct BeatmapAttributesExt;

impl BeatmapAttributesExt {
    pub(crate) fn difficulty_range(difficulty: f64, min: f64, mid: f64, max: f64) -> f64 {
        if difficulty > 5.0 {
            mid + (max - mid) * Self::difficulty_range_value(difficulty)
        } else if difficulty < 5.0 {
            mid + (mid - min) * Self::difficulty_range_value(difficulty)
        } else {
            mid
        }
    }

    pub(crate) fn difficulty_range_value(difficulty: f64) -> f64 {
        (difficulty - 5.0) / 5.0
    }

    pub(crate) fn inverse_difficulty_range(
        difficulty_value: f64,
        diff0: f64,
        diff5: f64,
        diff10: f64,
    ) -> f64 {
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
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct HitWindows {
    /// Hit window for approach rate i.e. `TimePreempt` in milliseconds.
    ///
    /// Only available for osu!standard and osu!catch.
    pub ar: Option<f64>,
    /// Perfect hit window for overall difficulty i.e. time to hit a "Perfect"
    /// in milliseconds.
    ///
    /// Only available for osu!mania.
    pub od_perfect: Option<f64>,
    /// Great hit window for overall difficulty i.e. time to hit a 300 ("Great")
    /// in milliseconds.
    ///
    /// Only available for osu!standard, osu!taiko, and osu!mania.
    pub od_great: Option<f64>,
    /// Good hit window for overall difficulty i.e. time to hit a "Good" in
    /// milliseconds.
    ///
    /// Only available for osu!mania.
    pub od_good: Option<f64>,
    /// Ok hit window for overall difficulty i.e. time to hit a 100 ("Ok") in
    /// milliseconds.
    ///
    /// Only available for osu!standard, osu!taiko, and osu!mania.
    pub od_ok: Option<f64>,
    /// Meh hit window for overall difficulty i.e. time to hit a 50 ("Meh") in
    /// milliseconds.
    ///
    /// Only available for osu!standard and osu!mania.
    pub od_meh: Option<f64>,
}

/// A builder for [`BeatmapAttributes`] and [`HitWindows`].
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
        Self {
            mode: self.mode,
            is_convert: self.is_convert,
            difficulty: BeatmapDifficulty {
                ar: difficulty
                    .get_ar()
                    .map_or(self.difficulty.ar, BeatmapAttribute::new),
                od: difficulty
                    .get_od()
                    .map_or(self.difficulty.od, BeatmapAttribute::new),
                cs: difficulty
                    .get_cs()
                    .map_or(self.difficulty.cs, BeatmapAttribute::new),
                hp: difficulty
                    .get_hp()
                    .map_or(self.difficulty.hp, BeatmapAttribute::new),
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

    pub(crate) const fn osu_great_hit_window_to_od(hit_window: f64) -> f64 {
        (79.5 - hit_window) / 6.0
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

#[derive(Copy, Clone, Debug, PartialEq)]
enum BeatmapAttribute {
    /// Variable value that may be overriden and adjusted based on mods and
    /// clock rate.
    Value(f32),
    /// Given by the user and may not be overriden by custom mod values.
    ///
    /// Mods and clock rate may *adjust* the value, though.
    ///
    /// # Example
    /// Mods include `DifficultyAdjust` which sets AR to 9.5 but the user
    /// specified AR to be 9.7. In this case, the user's value is should take
    /// precedence.
    Given(f32),
    /// Represents a final value that should not be adjusted based on mods or
    /// clock rate.
    Fixed(f32),
}

impl BeatmapAttribute {
    const DEFAULT: Self = Self::Value(5.0);

    const fn new(value: ModsDependent) -> Self {
        if value.with_mods {
            Self::Fixed(value.value)
        } else {
            Self::Given(value.value)
        }
    }

    /// Mutates the `Value` and `Given` variants.
    fn try_mutate(&mut self, f: impl Fn(&mut f32)) {
        if let Self::Value(value) | Self::Given(value) = self {
            f(value);
        }
    }

    /// Sets the `Value` variant only.
    const fn try_set(&mut self, value: f32) {
        if let Self::Value(old) = self {
            *old = value;
        }
    }

    /// Applies `f` onto the `Value` and `Given` variants and `default` onto the
    /// `Fixed` variant.
    fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(f32) -> U,
        F: FnOnce(f32) -> U,
    {
        match self {
            Self::Value(value) | Self::Given(value) => f(value),
            Self::Fixed(fixed) => default(fixed),
        }
    }

    const fn get_raw(self) -> f32 {
        match self {
            Self::Value(value) | Self::Given(value) | Self::Fixed(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct BeatmapDifficulty {
    ar: BeatmapAttribute,
    cs: BeatmapAttribute,
    hp: BeatmapAttribute,
    od: BeatmapAttribute,
}

impl BeatmapDifficulty {
    const DEFAULT: Self = Self {
        ar: BeatmapAttribute::DEFAULT,
        cs: BeatmapAttribute::DEFAULT,
        hp: BeatmapAttribute::DEFAULT,
        od: BeatmapAttribute::DEFAULT,
    };

    fn apply_mods(&mut self, mods: &GameMods, mode: GameMode) {
        // First we *set* values
        if let GameMods::Lazer(mods) = mods {
            macro_rules! set_if_some {
                ( $attr:ident = $opt:expr ) => {
                    if let Some(value) = $opt {
                        self.$attr.try_set(value as f32);
                    }
                };
            }

            for m in mods.iter() {
                let (hp, od) = match m {
                    GameMod::DifficultyAdjustCatch(da) => {
                        set_if_some!(ar = da.approach_rate);
                        set_if_some!(cs = da.circle_size);

                        (da.drain_rate, da.overall_difficulty)
                    }
                    GameMod::DifficultyAdjustMania(da) => (da.drain_rate, da.overall_difficulty),
                    GameMod::DifficultyAdjustOsu(da) => {
                        set_if_some!(ar = da.approach_rate);
                        set_if_some!(cs = da.circle_size);

                        (da.drain_rate, da.overall_difficulty)
                    }
                    GameMod::DifficultyAdjustTaiko(da) => {
                        // Ignoring slider multiplier

                        (da.drain_rate, da.overall_difficulty)
                    }
                    _ => continue,
                };

                set_if_some!(hp = hp);
                set_if_some!(od = od);
            }
        }

        // Then we *adjust* values
        if mods.ez() {
            const ADJUST_RATIO: f32 = 0.5;

            self.ar.try_mutate(|ar| *ar *= ADJUST_RATIO);
            self.cs.try_mutate(|cs| *cs *= ADJUST_RATIO);
            self.hp.try_mutate(|hp| *hp *= ADJUST_RATIO);

            match mode {
                GameMode::Osu => self.od.try_mutate(|od| *od *= ADJUST_RATIO),
                // Ignoring slider multiplier
                GameMode::Taiko => self.od.try_mutate(|od| *od *= ADJUST_RATIO),
                GameMode::Catch => self.od.try_mutate(|od| *od *= ADJUST_RATIO),
                GameMode::Mania => {}
            }
        } else if mods.hr() {
            const ADJUST_RATIO: f32 = 1.4;

            self.hp
                .try_mutate(|hp| *hp = f32::min(*hp * ADJUST_RATIO, 10.0));

            match mode {
                GameMode::Osu => {
                    self.od
                        .try_mutate(|od| *od = f32::min(*od * ADJUST_RATIO, 10.0));
                    // * CS uses a custom 1.3 ratio.
                    self.cs.try_mutate(|cs| *cs = f32::min(*cs * 1.3, 10.0));
                    self.ar
                        .try_mutate(|ar| *ar = f32::min(*ar * ADJUST_RATIO, 10.0));
                }
                // Ignoring slider multiplier
                GameMode::Taiko => self
                    .od
                    .try_mutate(|od| *od = f32::min(*od * ADJUST_RATIO, 10.0)),
                GameMode::Catch => {
                    self.od
                        .try_mutate(|od| *od = f32::min(*od * ADJUST_RATIO, 10.0));
                    // * CS uses a custom 1.3 ratio.
                    self.cs.try_mutate(|cs| *cs = f32::min(*cs * 1.3, 10.0));
                    self.ar
                        .try_mutate(|ar| *ar = f32::min(*ar * ADJUST_RATIO, 10.0));
                }
                GameMode::Mania => {}
            }
        }
    }
}

impl Default for BeatmapAttributesBuilder {
    fn default() -> Self {
        Self::new()
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

        pub const GREAT: GameModeHitWindows = GameModeHitWindows {
            min: 64.0,
            mid: 49.0,
            max: 34.0,
        };

        pub const GOOD: GameModeHitWindows = GameModeHitWindows {
            min: 97.0,
            mid: 82.0,
            max: 67.0,
        };

        pub const OK: GameModeHitWindows = GameModeHitWindows {
            min: 127.0,
            mid: 112.0,
            max: 97.0,
        };

        pub const MEH: GameModeHitWindows = GameModeHitWindows {
            min: 151.0,
            mid: 136.0,
            max: 121.0,
        };
    }

    // Same in both Osu and Catch
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

        assert_eq!(attrs.ar(), 5.0);
    }

    #[test]
    fn ar_without_mods() {
        let gamemod = GameMod::DoubleTimeOsu(DoubleTimeOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, false)
            .difficulty(&diff)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.ar, 10.0);
    }

    #[test]
    fn ar_with_mods() {
        let gamemod = GameMod::DoubleTimeOsu(DoubleTimeOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, true)
            .difficulty(&diff)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.ar, 8.5);
    }

    #[test]
    fn mods_ar() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(DoubleTimeCatch::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(7.0),
            ..DifficultyAdjustOsu::default()
        }));
        let diff = Difficulty::new().mods(mods);

        let attrs = BeatmapAttributesBuilder::new()
            .difficulty(&diff)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.ar, 9.0);
    }

    #[test]
    fn ar_mods_ar_without_mods() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(DoubleTimeCatch::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(9.0),
            ..DifficultyAdjustOsu::default()
        }));

        let diff = Difficulty::new().mods(mods).ar(8.5, false);

        let attrs = BeatmapAttributesBuilder::new()
            .difficulty(&diff)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.ar, 10.0);
    }

    #[test]
    fn ar_mods_ar_with_mods() {
        let mut mods = GameMods::new();
        mods.insert(GameMod::DoubleTimeCatch(DoubleTimeCatch::default()));
        mods.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            approach_rate: Some(9.0),
            ..DifficultyAdjustOsu::default()
        }));

        let diff = Difficulty::new().mods(mods).ar(8.5, true);

        let attrs = BeatmapAttributesBuilder::new()
            .difficulty(&diff)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.ar, 8.5);
    }

    #[test]
    fn set_od_before_applying_hr() {
        let mut hr = GameMods::new();
        hr.insert(GameMod::HardRockOsu(Default::default()));

        let attrs = BeatmapAttributesBuilder::new()
            .ar(5.0, false)
            .mods(hr)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.od, 7.0);

        let mut hrda = GameMods::new();
        hrda.insert(GameMod::HardRockOsu(Default::default()));
        hrda.insert(GameMod::DifficultyAdjustOsu(DifficultyAdjustOsu {
            overall_difficulty: Some(7.0),
            ..Default::default()
        }));

        let attrs = BeatmapAttributesBuilder::new()
            .ar(5.0, false)
            .mods(hrda)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.od, 9.800000190734863);
    }

    #[test]
    fn same_hit_windows_fixed_vs_given() {
        for mode in [
            GameMode::Osu,
            GameMode::Taiko,
            GameMode::Catch,
            GameMode::Mania,
        ] {
            let fixed = BeatmapAttributesBuilder::new()
                .mode(mode, false)
                .ar(6.0, true)
                .od(6.0, true)
                .build()
                .hit_windows();

            let given = BeatmapAttributesBuilder::new()
                .mode(mode, false)
                .ar(6.0, false)
                .od(6.0, false)
                .build()
                .hit_windows();

            assert_eq!(fixed, given, "{mode:?}");
        }
    }
}
