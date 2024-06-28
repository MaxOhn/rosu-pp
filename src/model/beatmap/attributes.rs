use rosu_map::section::general::GameMode;

use crate::{any::difficulty::ModsDependent, util::mods::Mods, Difficulty};

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
    ar: ModsDependent,
    od: ModsDependent,
    cs: ModsDependent,
    hp: ModsDependent,
    mods: u32,
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
            ar: ModsDependent::new(5.0),
            od: ModsDependent::new(5.0),
            cs: ModsDependent::new(5.0),
            hp: ModsDependent::new(5.0),
            mods: 0,
            clock_rate: None,
        }
    }

    /// Use the given [`Beatmap`]'s attributes, mode, and convert status.
    pub const fn map(self, map: &Beatmap) -> Self {
        Self {
            mode: map.mode,
            ar: ModsDependent::new(map.ar),
            od: ModsDependent::new(map.od),
            cs: ModsDependent::new(map.cs),
            hp: ModsDependent::new(map.hp),
            mods: 0,
            clock_rate: None,
            is_convert: map.is_convert,
        }
    }

    /// Specify the approach rate.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn ar(self, ar: f32, with_mods: bool) -> Self {
        Self {
            ar: ModsDependent {
                value: ar,
                with_mods,
            },
            ..self
        }
    }

    /// Specify the overall difficulty.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn od(self, od: f32, with_mods: bool) -> Self {
        Self {
            od: ModsDependent {
                value: od,
                with_mods,
            },
            ..self
        }
    }

    /// Specify the circle size.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn cs(self, cs: f32, with_mods: bool) -> Self {
        Self {
            cs: ModsDependent {
                value: cs,
                with_mods,
            },
            ..self
        }
    }

    /// Specify the drain rate.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    pub const fn hp(self, hp: f32, with_mods: bool) -> Self {
        Self {
            hp: ModsDependent {
                value: hp,
                with_mods,
            },
            ..self
        }
    }

    /// Specify the mods.
    pub const fn mods(self, mods: u32) -> Self {
        Self { mods, ..self }
    }

    /// Specify a custom clock rate.
    pub const fn clock_rate(self, clock_rate: f64) -> Self {
        Self {
            clock_rate: Some(clock_rate),
            ..self
        }
    }

    /// Specify a [`GameMode`] and whether it's a converted map.
    pub const fn mode(self, mode: GameMode, is_convert: bool) -> Self {
        Self {
            mode,
            is_convert,
            ..self
        }
    }

    /// Specify all settings through [`Difficulty`].
    pub fn difficulty(self, difficulty: &Difficulty) -> Self {
        Self {
            mode: self.mode,
            is_convert: self.is_convert,
            ar: difficulty.get_ar().unwrap_or(self.ar),
            od: difficulty.get_od().unwrap_or(self.od),
            cs: difficulty.get_cs().unwrap_or(self.cs),
            hp: difficulty.get_hp().unwrap_or(self.hp),
            mods: difficulty.get_mods(),
            clock_rate: Some(difficulty.get_clock_rate()),
        }
    }

    /// Calculate the AR and OD hit windows.
    pub fn hit_windows(&self) -> HitWindows {
        let mods = self.mods;

        let clock_rate = self.clock_rate.unwrap_or(mods.clock_rate());
        let ar_clock_rate = if self.ar.with_mods { 1.0 } else { clock_rate };
        let od_clock_rate = if self.od.with_mods { 1.0 } else { clock_rate };

        let mod_mult = |val: f32| {
            if mods.hr() {
                (val * 1.4).min(10.0)
            } else if mods.ez() {
                val * 0.5
            } else {
                val
            }
        };

        let raw_ar = if self.ar.with_mods {
            self.ar.value
        } else {
            mod_mult(self.ar.value)
        };

        let preempt = difficulty_range(f64::from(raw_ar), 1800.0, 1200.0, 450.0) / ar_clock_rate;

        // OD
        let hit_window = match self.mode {
            GameMode::Osu | GameMode::Catch => {
                let raw_od = if self.od.with_mods {
                    self.od.value
                } else {
                    mod_mult(self.od.value)
                };

                difficulty_range(
                    f64::from(raw_od),
                    Self::OSU_MIN,
                    Self::OSU_AVG,
                    Self::OSU_MAX,
                ) / od_clock_rate
            }
            GameMode::Taiko => {
                let raw_od = if self.od.with_mods {
                    self.od.value
                } else {
                    mod_mult(self.od.value)
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
                    34.0 + 3.0 * (10.0 - self.od.value).clamp(0.0, 10.0)
                } else if self.od.value.round_ties_even() > 4.0 {
                    34.0
                } else {
                    47.0
                };

                if !self.od.with_mods {
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
        let mods = self.mods;
        let clock_rate = self.clock_rate.unwrap_or_else(|| mods.clock_rate());

        // HP
        let mut hp = self.hp.value;

        if !self.hp.with_mods {
            hp *= mods.od_ar_hp_multiplier() as f32;
        }

        hp = hp.min(10.0);

        // CS
        let mut cs = self.cs.value;

        if !self.cs.with_mods {
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
            GameMode::Catch | GameMode::Mania => f64::from(self.od.value),
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

#[cfg(test)]
mod tests {
    use crate::util::float_ext::FloatExt;

    use super::*;

    #[test]
    fn consider_mods() {
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, false)
            .mods(64)
            .build();

        let expected = 10.0;

        assert!(attrs.ar.eq(expected), "{} != {expected}", attrs.ar);
    }

    #[test]
    fn skip_mods() {
        let attrs = BeatmapAttributesBuilder::new()
            .ar(8.5, true)
            .mods(64)
            .build();

        let expected = 8.5;

        assert!(attrs.ar.eq(expected), "{} != {expected}", attrs.ar);
    }
}
