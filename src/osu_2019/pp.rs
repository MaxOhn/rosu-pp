use super::stars::{stars, OsuDifficultyAttributes, OsuPerformanceAttributes};
use crate::{Beatmap, GameMods};

/// Calculator for pp on osu!standard maps.
///
/// # Example
///
/// ```
/// # use rosu_pp::{OsuPP, Beatmap};
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
/// let attrs = OsuPP::new(&map)
///     .mods(8 + 64) // HDDT
///     .combo(1234)
///     .misses(1)
///     .accuracy(98.5) // should be set last
///     .calculate();
///
/// println!("PP: {} | Stars: {}", attrs.pp(), attrs.stars());
///
/// let next_result = OsuPP::new(&map)
///     .attributes(attrs) // reusing previous results for performance
///     .mods(8 + 64)      // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[derive(Clone, Debug)]
pub struct OsuPP<'m> {
    map: Option<&'m Beatmap>,
    attributes: Option<OsuDifficultyAttributes>,
    mods: GameMods,
    combo: Option<u32>,
    acc: Option<f32>,

    n300: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    n_misses: u32,
}

impl<'m> OsuPP<'m> {
    /// Creates a new calculator for the given map.
    #[inline]
    pub fn from_map(map: &'m Beatmap) -> Self {
        Self {
            map: Some(map),
            attributes: None,
            mods: GameMods::default(),
            combo: None,
            acc: None,
            n300: None,
            n100: None,
            n50: None,
            n_misses: 0,
        }
    }

    /// Creates a new calculator for the given attributes.
    #[inline]
    pub fn from_attributes(attributes: OsuDifficultyAttributes) -> Self {
        Self {
            map: None,
            attributes: Some(attributes),
            mods: GameMods::default(),
            combo: None,
            acc: None,
            n300: None,
            n100: None,
            n50: None,
            n_misses: 0,
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(mut self, mods: impl Into<GameMods>) -> Self {
        self.mods = mods.into();

        self
    }

    /// Specify the max combo of the play.
    #[inline]
    pub fn combo(mut self, combo: u32) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify the amount of 300s of a play.
    #[inline]
    pub fn n300(mut self, n300: u32) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 100s of a play.
    #[inline]
    pub fn n100(mut self, n100: u32) -> Self {
        self.n100 = Some(n100);

        self
    }

    /// Specify the amount of 50s of a play.
    #[inline]
    pub fn n50(mut self, n50: u32) -> Self {
        self.n50 = Some(n50);

        self
    }

    /// Specify the amount of misses of a play.
    #[inline]
    pub fn misses(mut self, n_misses: u32) -> Self {
        self.n_misses = n_misses;

        self
    }

    /// Generate the hit results with respect to the given accuracy between `0` and `100`.
    ///
    /// Be sure to set `misses` beforehand!
    pub fn accuracy(mut self, acc: f32) -> Self {
        let n_objects = self.n_objects();

        let acc = acc / 100.0;

        if self.n100.or(self.n50).is_some() {
            let mut n100 = self.n100.unwrap_or(0);
            let mut n50 = self.n50.unwrap_or(0);

            let placed_points = 2 * n100 + n50 + self.n_misses;
            let missing_objects = n_objects - n100 - n50 - self.n_misses;
            let missing_points =
                ((6.0 * acc * n_objects as f32).round() as u32).saturating_sub(placed_points);

            let mut n300 = missing_objects.min(missing_points / 6);
            n50 += missing_objects - n300;

            if let Some(orig_n50) = self.n50.filter(|_| self.n100.is_none()) {
                // Only n50s were changed, try to load some off again onto n100s
                let difference = n50 - orig_n50;
                let n = n300.min(difference / 4);

                n300 -= n;
                n100 += 5 * n;
                n50 -= 4 * n;
            }

            self.n300.replace(n300);
            self.n100.replace(n100);
            self.n50.replace(n50);
        } else {
            let misses = self.n_misses.min(n_objects);
            let target_total = (acc * n_objects as f32 * 6.0).round() as u32;
            let delta = target_total - (n_objects - misses);

            let mut n300 = delta / 5;
            let mut n100 = delta % 5;
            let mut n50 = n_objects - n300 - n100 - misses;

            // Sacrifice n300s to transform n50s into n100s
            let n = n300.min(n50 / 4);
            n300 -= n;
            n100 += 5 * n;
            n50 -= 4 * n;

            self.n300.replace(n300);
            self.n100.replace(n100);
            self.n50.replace(n50);
        }

        let acc = (6 * self.n300.unwrap() + 2 * self.n100.unwrap() + self.n50.unwrap()) as f32
            / (6 * n_objects) as f32;

        self.acc.replace(acc);

        self
    }

    fn assert_hitresults(&mut self) {
        if self.acc.is_none() {
            let n_objects = self.n_objects();

            let remaining = n_objects
                .saturating_sub(self.n300.unwrap_or(0))
                .saturating_sub(self.n100.unwrap_or(0))
                .saturating_sub(self.n50.unwrap_or(0))
                .saturating_sub(self.n_misses);

            if remaining > 0 {
                if self.n300.is_none() {
                    self.n300.replace(remaining);
                    self.n100.get_or_insert(0);
                    self.n50.get_or_insert(0);
                } else if self.n100.is_none() {
                    self.n100.replace(remaining);
                    self.n50.get_or_insert(0);
                } else if self.n50.is_none() {
                    self.n50.replace(remaining);
                } else {
                    *self.n300.as_mut().unwrap() += remaining;
                }
            } else {
                self.n300.get_or_insert(0);
                self.n100.get_or_insert(0);
                self.n50.get_or_insert(0);
            }

            let numerator = self.n50.unwrap() + self.n100.unwrap() * 2 + self.n300.unwrap() * 6;
            self.acc.replace(numerator as f32 / n_objects as f32 / 6.0);
        }
    }

    /// Returns an object which contains the pp and [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// containing stars and other attributes.
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        if self.attributes.is_none() {
            let attributes = stars(self.map.unwrap(), self.mods.clone());
            self.attributes.replace(attributes);
        }

        // Make sure the hitresults and accuracy are set
        self.assert_hitresults();

        let total_hits = self.total_hits() as f32;
        let mut multiplier = 1.09;

        let effective_miss_count = self.calculate_effective_miss_count();

        // SO penalty
        if self.mods.so() {
            multiplier *=
                1.0 - (self.attributes.as_ref().unwrap().n_spinners as f32 / total_hits).powf(0.85);
        }

        let mut aim_value = self.compute_aim_value(total_hits, effective_miss_count);
        let speed_value = self.compute_speed_value(total_hits, effective_miss_count);
        let acc_value = self.compute_accuracy_value(total_hits);

        let mut acc_depression = 1.0;

        let difficulty = self.attributes.as_ref().unwrap();
        let streams_nerf =
            ((difficulty.aim_strain / difficulty.speed_strain) * 100.0).round() / 100.0;

        if streams_nerf < 1.09 {
            let acc_factor = (1.0 - self.acc.unwrap()).abs();
            acc_depression = (0.86 - acc_factor).max(0.5);

            if acc_depression > 0.0 {
                aim_value *= acc_depression;
            }
        }

        let nodt_bonus = match !(self.mods.dt() || self.mods.nc() || self.mods.ht()) {
            true => 1.02,
            false => 1.0,
        };

        let mut pp = (aim_value.powf(1.185 * nodt_bonus)
            + speed_value.powf(0.83 * acc_depression)
            + acc_value.powf(1.14 * nodt_bonus))
        .powf(1.0 / 1.1)
            * multiplier;

        if self.mods.dt() && self.mods.hr() {
            pp *= 1.025;
        }

        if difficulty.beatmap_creator == "gwb" || difficulty.beatmap_creator == "Plasma" {
            pp *= 0.9;
        }

        pp *= match difficulty.beatmap_id {
            // Louder than steel [ok this is epic]
            1808605 => 0.85,

            // over the top [Above the stars]
            1821147 => 0.70,

            // Just press F [Parkour's ok this is epic]
            1844776 => 0.64,

            // Hardware Store [skyapple mode]
            1777768 => 0.90,

            // Akatsuki compilation [ok this is akatsuki]
            1962833 => {
                pp *= 0.885;

                if self.mods.dt() {
                    0.83
                } else {
                    1.0
                }
            }

            // Songs Compilation [Marathon]
            2403677 => 0.85,

            // Songs Compilation [Remembrance]
            2174272 => 0.85,

            // Apocalypse 1992 [Universal Annihilation]
            2382377 => 0.85,

            _ => 1.0,
        };

        OsuPerformanceAttributes {
            difficulty: self.attributes.unwrap(),
            pp_aim: aim_value as f64,
            pp_speed: speed_value as f64,
            pp_acc: acc_value as f64,
            pp: pp as f64,
            effective_miss_count: effective_miss_count as f64,
        }
    }

    fn compute_aim_value(&self, total_hits: f32, effective_miss_count: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();

        // TD penalty
        let raw_aim = if self.mods.td() {
            attributes.aim_strain.powf(0.8) as f32
        } else {
            attributes.aim_strain as f32
        };

        let mut aim_value = (5.0 * (raw_aim / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.88
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f32 * 0.5 * (total_hits / 2000.0).log10();
        aim_value *= len_bonus;

        // Penalize misses
        if effective_miss_count > 0.0 {
            let miss_penalty = self.calculate_miss_penalty(effective_miss_count);
            aim_value *= miss_penalty;
        }

        // AR bonus
        let mut ar_factor = if attributes.ar > 10.33 {
            0.3 * (attributes.ar - 10.33)
        } else {
            0.0
        };

        if attributes.ar < 8.0 {
            ar_factor = 0.025 * (8.0 - attributes.ar);
        }

        aim_value *= 1.0 + ar_factor as f32 * len_bonus;

        // HD bonus
        if self.mods.hd() {
            aim_value *= 1.0 + 0.05 * (11.0 - attributes.ar) as f32;
        }

        // FL bonus
        if self.mods.fl() {
            aim_value *= 1.0
                + 0.3 * (total_hits / 200.0).min(1.0)
                + (total_hits > 200.0) as u8 as f32
                    * 0.25
                    * ((total_hits - 200.0) / 300.0).min(1.0)
                + (total_hits > 500.0) as u8 as f32 * (total_hits - 500.0) / 1600.0;
        }

        // EZ bonus
        if self.mods.ez() {
            let mut base_buff = 1.08_f32;

            if attributes.ar <= 8.0 {
                base_buff += (7.0 - attributes.ar as f32) / 100.0;
            }

            aim_value *= base_buff;
        }

        // Scale with accuracy
        aim_value *= 0.3 + self.acc.unwrap() / 2.0;
        aim_value *= 0.98 + attributes.od as f32 * attributes.od as f32 / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self, total_hits: f32, effective_miss_count: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();

        let mut speed_value =
            (5.0 * (attributes.speed_strain as f32 / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.88
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f32 * 0.5 * (total_hits / 2000.0).log10();
        speed_value *= len_bonus;

        // Penalize misses
        if effective_miss_count > 0.0 {
            let miss_penalty = self.calculate_miss_penalty(effective_miss_count);
            speed_value *= miss_penalty;
        }

        // AR bonus
        if attributes.ar > 10.33 {
            let mut ar_factor = if attributes.ar > 10.33 {
                0.3 * (attributes.ar - 10.33)
            } else {
                0.0
            };

            if attributes.ar < 8.0 {
                ar_factor = 0.025 * (8.0 - attributes.ar);
            }

            speed_value *= 1.0 + ar_factor as f32 * len_bonus;
        }

        // HD bonus
        if self.mods.hd() {
            speed_value *= 1.0 + 0.05 * (11.0 - attributes.ar) as f32;
        }

        // Scaling the speed value with accuracy and OD
        speed_value *= (0.93 + attributes.od as f32 * attributes.od as f32 / 750.0)
            * self
                .acc
                .unwrap()
                .powf((14.5 - attributes.od.max(8.0) as f32) / 2.0);

        speed_value *= 0.98_f32.powf(match (self.n50.unwrap() as f32) < total_hits / 500.0 {
            true => 0.0,
            false => self.n50.unwrap() as f32 - total_hits / 500.0,
        });

        speed_value
    }

    fn compute_accuracy_value(&self, total_hits: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();
        let n_circles = attributes.n_circles as f32;
        let n300 = self.n300.unwrap_or(0) as f32;
        let n100 = self.n100.unwrap_or(0) as f32;
        let n50 = self.n50.unwrap_or(0) as f32;

        let better_acc_percentage = (n_circles > 0.0) as u8 as f32
            * (((n300 - (total_hits - n_circles)) * 6.0 + n100 * 2.0 + n50) / (n_circles * 6.0))
                .max(0.0);

        let mut acc_value =
            1.52163_f32.powf(attributes.od as f32) * better_acc_percentage.powi(24) * 2.83;

        // Bonus for many hitcircles
        acc_value *= ((n_circles as f32 / 1000.0).powf(0.3)).min(1.15);

        // HD bonus
        if self.mods.hd() {
            acc_value *= 1.08;
        }

        // FL bonus
        if self.mods.fl() {
            acc_value *= 1.02;
        }

        acc_value
    }

    #[inline]
    fn total_hits(&self) -> u32 {
        let n_objects = self.n_objects();

        (self.n300.unwrap_or(0) + self.n100.unwrap_or(0) + self.n50.unwrap_or(0) + self.n_misses)
            .min(n_objects)
    }

    #[inline]
    fn calculate_miss_penalty(&self, effective_miss_count: f32) -> f32 {
        let total_hits = self.total_hits() as f32;

        0.97 * (1.0 - (effective_miss_count / total_hits).powf(0.5))
            .powf(1.0 + (effective_miss_count / 1.5))
    }

    #[inline]
    fn calculate_effective_miss_count(&self) -> f32 {
        let mut combo_based_miss_count = 0.0;

        let attributes = self.attributes.as_ref().unwrap();
        let combo = self.combo.unwrap_or(attributes.max_combo as u32) as f32;
        let n100 = self.n100.unwrap_or(0) as f32;
        let n50 = self.n50.unwrap_or(0) as f32;

        if attributes.n_sliders > 0 {
            let fc_threshold = attributes.max_combo as f32 - (0.1 * attributes.n_sliders as f32);
            if combo < fc_threshold {
                combo_based_miss_count = fc_threshold / combo.max(1.0);
            }
        }

        combo_based_miss_count = combo_based_miss_count.min(n100 + n50 + self.n_misses as f32);
        combo_based_miss_count.max(self.n_misses as f32)
    }

    #[inline]
    fn n_objects(&self) -> u32 {
        match self.attributes.as_ref() {
            Some(attributes) => {
                (attributes.n_circles + attributes.n_sliders + attributes.n_spinners) as u32
            }
            None => self.map.unwrap().hit_objects.len() as u32,
        }
    }
}

/// Provides attributes for an osu! beatmap.
pub trait OsuAttributeProvider {
    /// Returns the attributes of the map.
    fn attributes(self) -> Option<OsuDifficultyAttributes>;
}

impl OsuAttributeProvider for OsuDifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<OsuDifficultyAttributes> {
        Some(self)
    }
}

impl OsuAttributeProvider for OsuPerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<OsuDifficultyAttributes> {
        Some(self.difficulty)
    }
}
