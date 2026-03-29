use rosu_map::section::general::GameMode;

pub use self::{
    attribute::BeatmapAttribute, builder::BeatmapAttributesBuilder, hit_windows::HitWindows,
};

pub(crate) use self::{difficulty::BeatmapDifficulty, ext::BeatmapAttributesExt};

use crate::{GameMods, model::beatmap::attributes::hit_windows::GameModeHitWindows};

mod attribute;
mod builder;
mod difficulty;
mod ext;
mod hit_windows;

/// Summary struct for a [`Beatmap`]'s attributes.
///
/// The difference between this and [`BeatmapAttributes`] is that this struct
/// considers the clock rate in its attribute values.
///
/// [`Beatmap`]: crate::Beatmap
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
}

/// Summary struct for a [`Beatmap`]'s attributes.
///
/// Clock rate is *not* considered in attribute values.
///
/// [`Beatmap`]: crate::Beatmap
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
    /// Create a new [`BeatmapAttributesBuilder`].
    pub const fn builder() -> BeatmapAttributesBuilder {
        BeatmapAttributesBuilder::new()
    }

    /// The approach rate.
    pub fn ar(&self) -> f32 {
        match self.difficulty.ar {
            BeatmapAttribute::None => BeatmapAttribute::DEFAULT,
            BeatmapAttribute::Given(value) | BeatmapAttribute::Value(value) => value,
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
            BeatmapAttribute::None => BeatmapAttribute::DEFAULT,
            BeatmapAttribute::Given(value) | BeatmapAttribute::Value(value) => value,
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
        let ar = || {
            let value = match self.difficulty.ar {
                BeatmapAttribute::None => BeatmapAttribute::DEFAULT,
                BeatmapAttribute::Value(value) | BeatmapAttribute::Given(value) => value,
                BeatmapAttribute::Fixed(fixed) => {
                    return hit_windows::AR.difficulty_range(f64::from(fixed));
                }
            };

            hit_windows::AR.difficulty_range(f64::from(value)) / clock_rate
        };

        // See `{OsuHitWindows,TaikoHitWindows}.SetDifficulty`
        let set_difficulty = |hit_windows: &GameModeHitWindows| {
            let value = match self.difficulty.od {
                BeatmapAttribute::None => BeatmapAttribute::DEFAULT,
                BeatmapAttribute::Value(value) | BeatmapAttribute::Given(value) => value,
                BeatmapAttribute::Fixed(fixed) => {
                    //     Fixed           = f^-1(f(Value) / C)
                    // <=> f(Fixed)        = f(Value) / C
                    // <=> f(Fixed) * C    = f(Value)
                    let f_value = hit_windows.difficulty_range(f64::from(fixed)) * clock_rate;

                    return (f64::floor(f_value) - 0.5) / clock_rate;
                }
            };

            (f64::floor(hit_windows.difficulty_range(f64::from(value))) - 0.5) / clock_rate
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
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::float_cmp, reason = "we're just testing here")]

    use rosu_mods::{
        GameMod, GameMods,
        generated_mods::{DifficultyAdjustOsu, DoubleTimeCatch, DoubleTimeOsu, HiddenOsu},
    };

    use crate::Difficulty;

    use super::*;

    #[test]
    fn default_ar() {
        let gamemod = GameMod::HiddenOsu(HiddenOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributes::builder().difficulty(&diff).build();

        assert_eq!(attrs.ar(), 5.0);
    }

    #[test]
    fn ar_without_mods() {
        let gamemod = GameMod::DoubleTimeOsu(DoubleTimeOsu::default());
        let diff = Difficulty::new().mods(GameMods::from(gamemod));
        let attrs = BeatmapAttributes::builder()
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
        let attrs = BeatmapAttributes::builder()
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

        let attrs = BeatmapAttributes::builder()
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

        let attrs = BeatmapAttributes::builder()
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

        let attrs = BeatmapAttributes::builder()
            .difficulty(&diff)
            .build()
            .apply_clock_rate();

        assert_eq!(attrs.ar, 8.5);
    }

    #[test]
    fn set_od_before_applying_hr() {
        let mut hr = GameMods::new();
        hr.insert(GameMod::HardRockOsu(Default::default()));

        let attrs = BeatmapAttributes::builder()
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

        let attrs = BeatmapAttributes::builder()
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
            let fixed = BeatmapAttributes::builder()
                .mode(mode, false)
                .ar(6.0, true)
                .od(6.0, true)
                .build()
                .hit_windows();

            let given = BeatmapAttributes::builder()
                .mode(mode, false)
                .ar(6.0, false)
                .od(6.0, false)
                .build()
                .hit_windows();

            assert_eq!(fixed, given, "{mode:?}");
        }
    }

    #[test]
    fn getter_fixed_vs_given() {
        for mode in [
            GameMode::Osu,
            GameMode::Taiko,
            GameMode::Catch,
            GameMode::Mania,
        ] {
            let fixed = BeatmapAttributes::builder()
                .mode(mode, false)
                .ar(7.1, true)
                .od(7.1, true)
                .build();

            let given = BeatmapAttributes::builder()
                .mode(mode, false)
                .ar(7.1, false)
                .od(7.1, false)
                .build();

            assert_eq!(fixed.ar(), given.ar(), "{mode:?}");
            assert_eq!(fixed.od(), given.od(), "{mode:?}");
        }
    }
}
