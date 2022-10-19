use crate::{Beatmap, GameMode, Mods};

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
    pub hit_windows: BeatmapHitWindows,
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// AR and OD hit windows
pub struct BeatmapHitWindows {
    /// Hit window for approach rate i.e. TimePreempt in milliseconds.
    pub ar: f64,
    /// Hit window for overall difficulty i.e. time to hit a 300 ("Great") in milliseconds.
    pub od: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Specify values for this builder to get [`BeatmapAttributes`] or [`BeatmapHitWindows`] based on
/// mods & co.
pub struct BeatmapAttributesBuilder {
    mode: GameMode,
    ar: f32,
    od: f32,
    cs: f32,
    hp: f32,
    mods: Option<u32>,
    clock_rate: Option<f64>,
    converted: bool,
}

impl BeatmapAttributesBuilder {
    const OSU_MIN: f64 = 80.0;
    const OSU_AVG: f64 = 50.0;
    const OSU_MAX: f64 = 20.0;

    const TAIKO_MIN: f64 = 50.0;
    const TAIKO_AVG: f64 = 35.0;
    const TAIKO_MAX: f64 = 20.0;

    #[inline]
    /// Create a new [`BeatmapAttributesBuilder`].
    pub fn new(map: &Beatmap) -> Self {
        Self::from(map)
    }

    #[inline]
    /// Specify the mode.
    pub fn mode(&mut self, mode: GameMode) -> &mut Self {
        self.mode = mode;

        self
    }

    #[inline]
    /// Specify the approach rate.
    pub fn ar(&mut self, ar: f32) -> &mut Self {
        self.ar = ar;

        self
    }

    #[inline]
    /// Specify the overall difficulty.
    pub fn od(&mut self, od: f32) -> &mut Self {
        self.od = od;

        self
    }

    #[inline]
    /// Specify the circle size.
    pub fn cs(&mut self, cs: f32) -> &mut Self {
        self.cs = cs;

        self
    }

    #[inline]
    /// Specify the drain rate.
    pub fn hp(&mut self, hp: f32) -> &mut Self {
        self.hp = hp;

        self
    }

    #[inline]
    /// Specify the mods.
    pub fn mods(&mut self, mods: u32) -> &mut Self {
        self.mods = Some(mods);

        self
    }

    #[inline]
    /// Specify a custom clock rate.
    pub fn clock_rate(&mut self, clock_rate: f64) -> &mut Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    #[inline]
    /// Specify whether it's a converted map.
    /// Only relevant for mania.
    pub fn converted(&mut self, converted: bool) -> &mut Self {
        self.converted = converted;

        self
    }

    #[inline]
    /// Calculate the AR and OD hit windows.
    pub fn hit_windows(&self) -> BeatmapHitWindows {
        let mods = self.mods.unwrap_or(0);
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
        let preempt = difficulty_range(raw_ar as f64, 1800.0, 1200.0, 450.0) / clock_rate;

        // OD
        let hit_window = match self.mode {
            GameMode::Osu | GameMode::Catch => {
                let raw_od = mod_mult(self.od);

                difficulty_range(raw_od as f64, Self::OSU_MIN, Self::OSU_AVG, Self::OSU_MAX)
                    / clock_rate
            }
            GameMode::Taiko => {
                let raw_od = mod_mult(self.od);

                let diff_range = difficulty_range(
                    raw_od as f64,
                    Self::TAIKO_MIN,
                    Self::TAIKO_AVG,
                    Self::TAIKO_MAX,
                );

                diff_range / clock_rate
            }
            GameMode::Mania => {
                let mut value = if !self.converted {
                    34.0 + 3.0 * (10.0 - self.od).clamp(0.0, 10.0)
                } else if self.od > 4.0 {
                    34.0
                } else {
                    47.0
                };

                if mods.hr() {
                    value /= 1.4;
                } else if mods.ez() {
                    value *= 1.4;
                }

                ((value as f64 * clock_rate).floor() / clock_rate).ceil()
            }
        };

        BeatmapHitWindows {
            ar: preempt,
            od: hit_window,
        }
    }

    /// Calculate the [`BeatmapAttributes`].
    pub fn build(&self) -> BeatmapAttributes {
        let mods = self.mods.unwrap_or(0);
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
        let BeatmapHitWindows { ar, od } = hit_windows;

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
            GameMode::Catch | GameMode::Mania => self.od as f64,
        };

        BeatmapAttributes {
            ar,
            od,
            cs: cs as f64,
            hp: hp as f64,
            clock_rate,
            hit_windows,
        }
    }
}

impl From<&Beatmap> for BeatmapAttributesBuilder {
    #[inline]
    fn from(map: &Beatmap) -> Self {
        Self {
            mode: map.mode,
            ar: map.ar,
            od: map.od,
            cs: map.cs,
            hp: map.hp,
            mods: None,
            clock_rate: None,
            converted: false,
        }
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
