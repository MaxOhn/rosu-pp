use super::DifficultyAttributes;
use crate::{Beatmap, Mods, PpResult, StarResult};

/// Calculator for pp on osu!standard maps.
#[derive(Clone, Debug)]
pub struct OsuPP<'m> {
    map: &'m Beatmap,
    attributes: Option<DifficultyAttributes>,
    mods: u32,
    combo: Option<usize>,
    acc: Option<f32>,

    n300: Option<usize>,
    n100: Option<usize>,
    n50: Option<usize>,
    n_misses: usize,
    passed_objects: Option<usize>,
}

impl<'m> OsuPP<'m> {
    #[inline]
    pub fn new(map: &'m Beatmap) -> Self {
        Self {
            map,
            attributes: None,
            mods: 0,
            combo: None,
            acc: None,

            n300: None,
            n100: None,
            n50: None,
            n_misses: 0,
            passed_objects: None,
        }
    }

    /// [`OsuAttributeProvider`] is implemented by [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// and by [`PpResult`](crate::PpResult) meaning you can give the
    /// result of a star calculation or a pp calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    #[inline]
    pub fn attributes(mut self, attributes: impl OsuAttributeProvider) -> Self {
        if let Some(attributes) = attributes.attributes() {
            self.attributes.replace(attributes);
        }

        self
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Specify the max combo of the play.
    #[inline]
    pub fn combo(mut self, combo: usize) -> Self {
        self.combo.replace(combo);

        self
    }

    /// Specify the amount of 300s of a play.
    #[inline]
    pub fn n300(mut self, n300: usize) -> Self {
        self.n300.replace(n300);

        self
    }

    /// Specify the amount of 100s of a play.
    #[inline]
    pub fn n100(mut self, n100: usize) -> Self {
        self.n100.replace(n100);

        self
    }

    /// Specify the amount of 50s of a play.
    #[inline]
    pub fn n50(mut self, n50: usize) -> Self {
        self.n50.replace(n50);

        self
    }

    /// Specify the amount of misses of a play.
    #[inline]
    pub fn misses(mut self, n_misses: usize) -> Self {
        self.n_misses = n_misses;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects.replace(passed_objects);

        self
    }

    /// Generate the hit results with respect to the given accuracy between `0` and `100`.
    ///
    /// Be sure to set `misses` beforehand!
    /// In case of a partial play, be also sure to set `passed_objects` beforehand!
    pub fn accuracy(mut self, acc: f32) -> Self {
        let n_objects = self
            .passed_objects
            .unwrap_or_else(|| self.map.hit_objects.len());

        let acc = acc / 100.0;

        if self.n100.or(self.n50).is_some() {
            let mut n100 = self.n100.unwrap_or(0);
            let mut n50 = self.n50.unwrap_or(0);

            let placed_points = 2 * n100 + n50 + self.n_misses;
            let missing_objects = n_objects - n100 - n50 - self.n_misses;
            let missing_points =
                ((6.0 * acc * n_objects as f32).round() as usize).saturating_sub(placed_points);

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
            let target_total = (acc * n_objects as f32 * 6.0).round() as usize;
            let delta = target_total - (n_objects - self.n_misses);

            let mut n300 = delta / 5;
            let mut n100 = delta % 5;
            let mut n50 = n_objects - n300 - n100 - self.n_misses;

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
            let n_objects = self
                .passed_objects
                .unwrap_or_else(|| self.map.hit_objects.len());

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
            }

            let numerator = self.n50.unwrap() + self.n100.unwrap() * 2 + self.n300.unwrap() * 6;
            self.acc.replace(numerator as f32 / n_objects as f32 / 6.0);
        }
    }

    /// Returns an object which contains the pp and [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// containing stars and other attributes.
    #[cfg(feature = "no_leniency")]
    pub fn calculate(self) -> PpResult {
        self.calculate_with_func(super::no_leniency::stars)
    }

    /// Returns an object which contains the pp and [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// containing stars and other attributes.
    #[cfg(feature = "no_sliders_no_leniency")]
    pub fn calculate(self) -> PpResult {
        self.calculate_with_func(super::no_sliders_no_leniency::stars)
    }

    /// Returns an object which contains the pp and [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// containing stars and other attributes.
    #[cfg(feature = "all_included")]
    pub fn calculate(self) -> PpResult {
        self.calculate_with_func(super::all_included::stars)
    }

    // Omits an unnecessary error when enabled features are invalid
    #[cfg(not(any(
        feature = "no_leniency",
        feature = "no_sliders_no_leniency",
        feature = "all_included"
    )))]
    pub(crate) fn calculate(self) -> PpResult {
        unreachable!()
    }

    /// Returns an object which contains the pp and [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// containing stars and other attributes.
    ///
    /// `stars_func` will be used to calculate the difficulty attributes if they are not yet given.
    fn calculate_with_func(
        mut self,
        stars_func: impl FnOnce(&Beatmap, u32, Option<usize>) -> StarResult,
    ) -> PpResult {
        if self.attributes.is_none() {
            let attributes = stars_func(self.map, self.mods, self.passed_objects)
                .attributes()
                .unwrap();
            self.attributes.replace(attributes);
        }

        // Make sure the hitresults and accuracy are set
        self.assert_hitresults();

        let total_hits = self.total_hits();
        let mut multiplier = 1.12;

        // NF penalty
        if self.mods.nf() {
            multiplier *= (1.0 - 0.02 * self.n_misses as f32).max(0.9);
        }

        // SO penalty
        if self.mods.so() {
            let n_spinners = self.attributes.as_ref().unwrap().n_spinners;
            multiplier *= 1.0 - (n_spinners as f32 / total_hits as f32).powf(0.85);
        }

        let aim_value = self.compute_aim_value(total_hits as f32);
        let speed_value = self.compute_speed_value(total_hits as f32);
        let acc_value = self.compute_accuracy_value(total_hits);

        let pp = (aim_value.powf(1.1) + speed_value.powf(1.1) + acc_value.powf(1.1))
            .powf(1.0 / 1.1)
            * multiplier;

        let attributes = StarResult::Osu(self.attributes.unwrap());

        PpResult { pp, attributes }
    }

    fn compute_aim_value(&self, total_hits: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();

        // TD penalty
        let raw_aim = if self.mods.td() {
            attributes.aim_strain.powf(0.8)
        } else {
            attributes.aim_strain
        };

        let mut aim_value = (5.0 * (raw_aim / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f32 * 0.5 * (total_hits / 2000.0).log10();
        aim_value *= len_bonus;

        // Penalize misses
        if self.n_misses > 0 {
            aim_value *= 0.97
                * (1.0 - (self.n_misses as f32 / total_hits).powf(0.775))
                    .powi(self.n_misses as i32);
        }

        // Combo scaling
        if let Some(combo) = self.combo.filter(|_| attributes.max_combo > 0) {
            aim_value *= ((combo as f32 / attributes.max_combo as f32).powf(0.8)).min(1.0);
        }

        // AR bonus
        let mut ar_factor = 0.0;
        if attributes.ar > 10.33 {
            ar_factor += 0.4 * (attributes.ar - 10.33);
        } else if attributes.ar < 8.0 {
            ar_factor += 0.01 * (8.0 - attributes.ar);
        }
        aim_value *= 1.0 + ar_factor.min(ar_factor * total_hits / 1000.0);

        // HD bonus
        if self.mods.hd() {
            aim_value *= 1.0 + 0.04 * (12.0 - attributes.ar);
        }

        // FL bonus
        if self.mods.fl() {
            aim_value *= 1.0
                + 0.35 * (total_hits / 200.0).min(1.0)
                + (total_hits > 200.0) as u8 as f32 * 0.3 * ((total_hits - 200.0) / 300.0).min(1.0)
                + (total_hits > 500.0) as u8 as f32 * (total_hits - 500.0) / 1200.0;
        }

        // Scale with accuracy
        aim_value *= 0.5 + self.acc.unwrap() / 2.0;
        aim_value *= 0.98 + attributes.od * attributes.od / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self, total_hits: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();

        let mut speed_value =
            (5.0 * (attributes.speed_strain / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f32 * 0.5 * (total_hits / 2000.0).log10();
        speed_value *= len_bonus;

        // Penalize misses
        if self.n_misses > 0 {
            speed_value *= 0.97
                * (1.0 - (self.n_misses as f32 / total_hits).powf(0.775))
                    .powf((self.n_misses as f32).powf(0.875));
        }

        // Combo scaling
        if let Some(combo) = self.combo.filter(|_| attributes.max_combo > 0) {
            speed_value *= ((combo as f32 / attributes.max_combo as f32).powf(0.8)).min(1.0);
        }

        // AR bonus
        if attributes.ar > 10.33 {
            let ar_factor = 0.4 * (attributes.ar - 10.33);
            speed_value *= 1.0 + ar_factor.min(ar_factor * total_hits / 1000.0);
        }

        // HD bonus
        if self.mods.hd() {
            speed_value *= 1.0 + 0.04 * (12.0 - attributes.ar);
        }

        // Scaling the speed value with accuracy and OD
        let od_factor = 0.95 + attributes.od * attributes.od / 750.0;
        let acc_factor = self
            .acc
            .unwrap()
            .powf((14.5 - attributes.od.max(8.0)) / 2.0);
        speed_value *= od_factor * acc_factor;

        // Penalize n50s
        speed_value *= 0.98_f32.powf(
            (self.n50.unwrap_or(0) as f32 >= total_hits / 500.0) as u8 as f32
                * (self.n50.unwrap_or(0) as f32 - total_hits / 500.0),
        );

        speed_value
    }

    fn compute_accuracy_value(&self, total_hits: usize) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();
        let n_circles = attributes.n_circles;

        let better_acc_percentage = (n_circles > 0) as u8 as f32
            * (((self.n300.unwrap_or(0) - (total_hits - n_circles)) * 6
                + self.n100.unwrap_or(0) * 2
                + self.n50.unwrap_or(0)) as f32
                / (n_circles * 6) as f32)
                .max(0.0);

        let mut acc_value = 1.52163_f32.powf(attributes.od) * better_acc_percentage.powi(24) * 2.83;

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
    fn total_hits(&self) -> usize {
        self.n300.unwrap_or(0) + self.n100.unwrap_or(0) + self.n50.unwrap_or(0) + self.n_misses
    }
}

pub trait OsuAttributeProvider {
    fn attributes(self) -> Option<DifficultyAttributes>;
}

impl OsuAttributeProvider for DifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<DifficultyAttributes> {
        Some(self)
    }
}

impl OsuAttributeProvider for StarResult {
    #[inline]
    fn attributes(self) -> Option<DifficultyAttributes> {
        #[allow(irrefutable_let_patterns)]
        if let Self::Osu(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }
}

impl OsuAttributeProvider for PpResult {
    #[inline]
    fn attributes(self) -> Option<DifficultyAttributes> {
        self.attributes.attributes()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Beatmap;

    #[test]
    fn osu_only_accuracy() {
        let map = Beatmap::default();

        let total_objects = 1234;
        let target_acc = 97.5;

        let calculator = OsuPP::new(&map)
            .passed_objects(total_objects)
            .accuracy(target_acc);

        let numerator = 6 * calculator.n300.unwrap_or(0)
            + 2 * calculator.n100.unwrap_or(0)
            + calculator.n50.unwrap_or(0);
        let denominator = 6 * total_objects;
        let acc = 100.0 * numerator as f32 / denominator as f32;

        assert!(
            (target_acc - acc).abs() < 1.0,
            "Expected: {} | Actual: {}",
            target_acc,
            acc
        );
    }

    #[test]
    fn osu_accuracy_and_n50() {
        let map = Beatmap::default();

        let total_objects = 1234;
        let target_acc = 97.5;
        let n50 = 30;

        let calculator = OsuPP::new(&map)
            .passed_objects(total_objects)
            .n50(n50)
            .accuracy(target_acc);

        assert!(
            (calculator.n50.unwrap() as i32 - n50 as i32).abs() <= 4,
            "Expected: {} | Actual: {}",
            n50,
            calculator.n50.unwrap()
        );

        let numerator = 6 * calculator.n300.unwrap_or(0)
            + 2 * calculator.n100.unwrap_or(0)
            + calculator.n50.unwrap_or(0);
        let denominator = 6 * total_objects;
        let acc = 100.0 * numerator as f32 / denominator as f32;

        assert!(
            (target_acc - acc).abs() < 1.0,
            "Expected: {} | Actual: {}",
            target_acc,
            acc
        );
    }

    #[test]
    fn osu_missing_objects() {
        let map = Beatmap::default();

        let total_objects = 1234;
        let n300 = 1000;
        let n100 = 200;
        let n50 = 30;

        let mut calculator = OsuPP::new(&map)
            .passed_objects(total_objects)
            .n300(n300)
            .n100(n100)
            .n50(n50);

        calculator.assert_hitresults();

        let n_objects = calculator.n300.unwrap()
            + calculator.n100.unwrap()
            + calculator.n50.unwrap()
            + calculator.n_misses;

        assert_eq!(
            total_objects, n_objects,
            "Expected: {} | Actual: {}",
            total_objects, n_objects
        );
    }
}
