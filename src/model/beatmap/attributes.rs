use rosu_map::section::general::GameMode;

use crate::util::{float_ext::FloatExt, mods::Mods};

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
    /// Hit window for approach rate i.e. TimePreempt in milliseconds.
    pub ar: f64,
    /// Hit window for overall difficulty i.e. time to hit a 300 ("Great") in milliseconds.
    pub od: f64,
}

/// A builder for [`BeatmapAttributes`] and [`HitWindows`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BeatmapAttributesBuilder {
    mode: GameMode,
    ar: f32,
    od: f32,
    cs: f32,
    hp: f32,
    mods: u32,
    clock_rate: Option<f64>,
    is_convert: bool,
}

impl BeatmapAttributesBuilder {
    const OSU_MIN: f64 = 80.0;
    const OSU_AVG: f64 = 50.0;
    const OSU_MAX: f64 = 20.0;

    const TAIKO_MIN: f64 = 50.0;
    const TAIKO_AVG: f64 = 35.0;
    const TAIKO_MAX: f64 = 20.0;

    /// Create a new [`BeatmapAttributesBuilder`].
    pub fn new(map: &Beatmap) -> Self {
        map.into()
    }

    const fn new_internal(map: &Beatmap, is_convert: bool) -> Self {
        Self {
            mode: map.mode,
            ar: map.ar,
            od: map.od,
            cs: map.cs,
            hp: map.hp,
            mods: 0,
            clock_rate: None,
            is_convert,
        }
    }

    /// Specify the approach rate.
    pub fn ar(&mut self, ar: f32) -> &mut Self {
        self.ar = ar;

        self
    }

    /// Specify the overall difficulty.
    pub fn od(&mut self, od: f32) -> &mut Self {
        self.od = od;

        self
    }

    /// Specify the circle size.
    pub fn cs(&mut self, cs: f32) -> &mut Self {
        self.cs = cs;

        self
    }

    /// Specify the drain rate.
    pub fn hp(&mut self, hp: f32) -> &mut Self {
        self.hp = hp;

        self
    }

    /// Specify the mods.
    pub fn mods(&mut self, mods: u32) -> &mut Self {
        self.mods = mods;

        self
    }

    /// Specify a custom clock rate.
    pub fn clock_rate(&mut self, clock_rate: f64) -> &mut Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Calculate the AR and OD hit windows.
    pub fn hit_windows(&self) -> HitWindows {
        let mods = self.mods;
        let clock_rate = self.clock_rate.unwrap_or_else(|| mods.clock_rate());

        let mod_mult = |val: f32| {
            if mods.hr() {
                (val * 1.4).min(10.0)
            } else if mods.ez() {
                val * 0.5
            } else {
                val
            }
        };

        let raw_ar = mod_mult(self.ar);
        let preempt = difficulty_range(f64::from(raw_ar), 1800.0, 1200.0, 450.0) / clock_rate;

        // OD
        let hit_window = match self.mode {
            GameMode::Osu | GameMode::Catch => {
                let raw_od = mod_mult(self.od);

                difficulty_range(
                    f64::from(raw_od),
                    Self::OSU_MIN,
                    Self::OSU_AVG,
                    Self::OSU_MAX,
                ) / clock_rate
            }
            GameMode::Taiko => {
                let raw_od = mod_mult(self.od);

                let diff_range = difficulty_range(
                    f64::from(raw_od),
                    Self::TAIKO_MIN,
                    Self::TAIKO_AVG,
                    Self::TAIKO_MAX,
                );

                diff_range / clock_rate
            }
            GameMode::Mania => {
                let mut value = if !self.is_convert {
                    34.0 + 3.0 * (10.0 - self.od).clamp(0.0, 10.0)
                } else if self.od.round_even() > 4.0 {
                    34.0
                } else {
                    47.0
                };

                if mods.hr() {
                    value /= 1.4;
                } else if mods.ez() {
                    value *= 1.4;
                }

                ((f64::from(value) * clock_rate).floor() / clock_rate).ceil()
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
        let hp = (self.hp * mods.od_ar_hp_multiplier() as f32).min(10.0);

        // CS
        let mut cs = self.cs;

        if mods.hr() {
            cs = (cs * 1.3).min(10.);
        } else if mods.ez() {
            cs *= 0.5;
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
            GameMode::Catch | GameMode::Mania => f64::from(self.od),
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
        Self::new_internal(map, false)
    }
}

impl<M> From<&Converted<'_, M>> for BeatmapAttributesBuilder {
    fn from(converted: &Converted<'_, M>) -> Self {
        Self::new_internal(converted.map.as_ref(), converted.is_convert)
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
