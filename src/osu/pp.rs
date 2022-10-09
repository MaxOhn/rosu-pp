use super::{
    OsuDifficultyAttributes, OsuPerformanceAttributes, OsuScoreState, PERFORMANCE_BASE_MULTIPLIER,
};
use crate::{
    AnyPP, Beatmap, DifficultyAttributes, GameMode, Mods, OsuStars, PerformanceAttributes,
};

/// Performance calculator on osu!standard maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{OsuPP, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let pp_result = OsuPP::new(&map)
///     .mods(8 + 64) // HDDT
///     .combo(1234)
///     .misses(1)
///     .accuracy(98.5) // should be set last
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = OsuPP::new(&map)
///     .attributes(pp_result)  // reusing previous results for performance
///     .mods(8 + 64)           // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct OsuPP<'map> {
    pub(crate) map: &'map Beatmap,
    pub(crate) attributes: Option<OsuDifficultyAttributes>,
    pub(crate) mods: u32,
    pub(crate) acc: Option<f64>,
    pub(crate) combo: Option<usize>,

    pub(crate) n300: Option<usize>,
    pub(crate) n100: Option<usize>,
    pub(crate) n50: Option<usize>,
    pub(crate) n_misses: usize,
    pub(crate) passed_objects: Option<usize>,
    pub(crate) clock_rate: Option<f64>,
}

impl<'map> OsuPP<'map> {
    /// Create a new performance calculator for osu!standard maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map,
            attributes: None,
            mods: 0,
            acc: None,
            combo: None,

            n300: None,
            n100: None,
            n50: None,
            n_misses: 0,
            passed_objects: None,
            clock_rate: None,
        }
    }

    /// Convert the map into another mode.
    #[inline]
    pub fn mode(self, mode: GameMode) -> AnyPP<'map> {
        match mode {
            GameMode::Osu => AnyPP::Osu(self),
            GameMode::Taiko => AnyPP::Taiko(self.into()),
            GameMode::Catch => AnyPP::Catch(self.into()),
            GameMode::Mania => AnyPP::Mania(self.into()),
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
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
    pub fn n_misses(mut self, n_misses: usize) -> Self {
        self.n_misses = n_misses;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`OsuPP`] multiple times with different `passed_objects`, you should use
    /// [`OsuGradualPerformanceAttributes`](crate::osu::OsuGradualPerformanceAttributes).
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects.replace(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    #[inline]
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Provide parameters through an [`OsuScoreState`].
    #[inline]
    pub fn state(mut self, state: OsuScoreState) -> Self {
        let OsuScoreState {
            max_combo,
            n300,
            n100,
            n50,
            n_misses,
        } = state;

        self.combo = Some(max_combo);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.n50 = Some(n50);
        self.n_misses = n_misses;

        self
    }

    /// Generate the hit results with respect to the given accuracy between `0` and `100`.
    ///
    /// Be sure to set `misses` beforehand!
    /// In case of a partial play, be also sure to set `passed_objects` beforehand!
    pub fn accuracy(mut self, acc: f64) -> Self {
        let n_objects = self.passed_objects.unwrap_or(self.map.hit_objects.len());

        let mut acc = acc / 100.0;

        if self.n100.or(self.n50).is_some() {
            let mut n100 = self.n100.unwrap_or(0);
            let mut n50 = self.n50.unwrap_or(0);

            let placed_points = 2 * n100 + n50 + self.n_misses;
            let missing_objects = n_objects - n100 - n50 - self.n_misses;
            let missing_points =
                ((6.0 * acc * n_objects as f64).round() as usize).saturating_sub(placed_points);

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

            self.n300 = Some(n300);
            self.n100 = Some(n100);
            self.n50 = Some(n50);

            acc = (6 * n300 + 2 * n100 + n50) as f64 / (6 * n_objects) as f64;
        } else {
            let misses = self.n_misses.min(n_objects);
            let target_total = (acc * n_objects as f64 * 6.0).round() as usize;
            let delta = target_total - (n_objects - misses);

            let mut n300 = delta / 5;
            let mut n100 = (delta % 5).min(n_objects - n300 - misses);
            let mut n50 = n_objects - n300 - n100 - misses;

            // Sacrifice n300s to transform n50s into n100s
            let n = n300.min(n50 / 4);
            n300 -= n;
            n100 += 5 * n;
            n50 -= 4 * n;

            self.n300 = Some(n300);
            self.n100 = Some(n100);
            self.n50 = Some(n50);

            acc = (6 * n300 + 2 * n100 + n50) as f64 / (6 * n_objects) as f64;
        }

        self.acc = Some(acc);

        self
    }

    fn assert_hitresults(self, attributes: OsuDifficultyAttributes) -> OsuPPInner {
        let mut n300 = self.n300;
        let mut n100 = self.n100;
        let mut n50 = self.n50;

        let n_objects = self.passed_objects.unwrap_or(self.map.hit_objects.len());

        if let Some(acc) = self.acc {
            let n300 = n300.unwrap_or(0);
            let n100 = n100.unwrap_or(0);
            let n50 = n50.unwrap_or(0);

            let total_hits = (n300 + n100 + n50 + self.n_misses).min(n_objects) as f64;

            let effective_misses =
                calculate_effective_misses(&attributes, self.combo, n100, n50, self.n_misses);

            OsuPPInner {
                mods: self.mods,
                combo: self.combo.unwrap_or(attributes.max_combo),
                acc,
                n300,
                n100,
                n50,
                n_misses: self.n_misses,
                total_hits,
                effective_miss_count: effective_misses,
                attributes,
            }
        } else {
            let n_objects = self.passed_objects.unwrap_or(self.map.hit_objects.len());

            let remaining = n_objects
                .saturating_sub(n300.unwrap_or(0))
                .saturating_sub(n100.unwrap_or(0))
                .saturating_sub(n50.unwrap_or(0))
                .saturating_sub(self.n_misses);

            if remaining > 0 {
                if let Some(n300) = n300.as_mut() {
                    if n100.is_none() {
                        n100 = Some(remaining);
                    } else if n50.is_none() {
                        n50 = Some(remaining);
                    } else {
                        *n300 += remaining;
                    }
                } else {
                    n300 = Some(remaining);
                }
            }

            let n300 = n300.unwrap_or(0);
            let n100 = n100.unwrap_or(0);
            let n50 = n50.unwrap_or(0);

            let numerator = n300 * 6 + n100 * 2 + n50;

            let acc = if n_objects > 0 {
                numerator as f64 / n_objects as f64 / 6.0
            } else {
                0.0
            };

            let total_hits = (n300 + n100 + n50 + self.n_misses).min(n_objects) as f64;

            let effective_misses =
                calculate_effective_misses(&attributes, self.combo, n100, n50, self.n_misses);

            OsuPPInner {
                mods: self.mods,
                combo: self.combo.unwrap_or(attributes.max_combo),
                acc,
                n300,
                n100,
                n50,
                n_misses: self.n_misses,
                total_hits,
                effective_miss_count: effective_misses,
                attributes,
            }
        }
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        let attributes = self.attributes.take().unwrap_or_else(|| {
            let mut calculator = OsuStars::new(self.map).mods(self.mods);

            if let Some(passed_objects) = self.passed_objects {
                calculator = calculator.passed_objects(passed_objects);
            }

            if let Some(clock_rate) = self.clock_rate {
                calculator = calculator.clock_rate(clock_rate);
            }

            calculator.calculate()
        });

        self.assert_hitresults(attributes).calculate()
    }
}

struct OsuPPInner {
    attributes: OsuDifficultyAttributes,
    mods: u32,
    acc: f64,
    combo: usize,

    n300: usize,
    n100: usize,
    n50: usize,
    n_misses: usize,

    total_hits: f64,
    effective_miss_count: f64,
}

impl OsuPPInner {
    fn calculate(mut self) -> OsuPerformanceAttributes {
        if self.total_hits.abs() <= f64::EPSILON {
            return OsuPerformanceAttributes {
                difficulty: self.attributes,
                ..Default::default()
            };
        }

        let mut multiplier = PERFORMANCE_BASE_MULTIPLIER;

        if self.mods.nf() {
            multiplier *= (1.0 - 0.02 * self.effective_miss_count).max(0.9);
        }

        if self.mods.so() && self.total_hits > 0.0 {
            multiplier *= 1.0 - (self.attributes.n_spinners as f64 / self.total_hits).powf(0.85);
        }

        if self.mods.rx() {
            // * https://www.desmos.com/calculator/bc9eybdthb
            // * we use OD13.3 as maximum since it's the value at which great hitwidow becomes 0
            // * this is well beyond currently maximum achievable OD which is 12.17 (DTx2 + DA with OD11)
            let (n100_mult, n50_mult) = if self.attributes.od > 0.0 {
                (
                    1.0 - (self.attributes.od / 13.33).powf(1.8),
                    1.0 - (self.attributes.od / 13.33).powi(5),
                )
            } else {
                (1.0, 1.0)
            };

            // * As we're adding Oks and Mehs to an approximated number of combo breaks the result can be
            // * higher than total hits in specific scenarios (which breaks some calculations) so we need to clamp it.
            self.effective_miss_count = (self.effective_miss_count
                + self.n100 as f64
                + n100_mult
                + self.n50 as f64 * n50_mult)
                .min(self.total_hits);
        }

        let aim_value = self.compute_aim_value();
        let speed_value = self.compute_speed_value();
        let acc_value = self.compute_accuracy_value();
        let flashlight_value = self.compute_flashlight_value();

        let pp = (aim_value.powf(1.1)
            + speed_value.powf(1.1)
            + acc_value.powf(1.1)
            + flashlight_value.powf(1.1))
        .powf(1.0 / 1.1)
            * multiplier;

        OsuPerformanceAttributes {
            difficulty: self.attributes,
            pp_acc: acc_value,
            pp_aim: aim_value,
            pp_flashlight: flashlight_value,
            pp_speed: speed_value,
            pp,
            effective_miss_count: self.effective_miss_count,
        }
    }

    fn compute_aim_value(&self) -> f64 {
        let mut aim_value =
            (5.0 * (self.attributes.aim / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let len_bonus = 0.95
            + 0.4 * (self.total_hits / 2000.0).min(1.0)
            + (self.total_hits > 2000.0) as u8 as f64 * (self.total_hits / 2000.0).log10() * 0.5;

        aim_value *= len_bonus;

        // * Penalize misses by assessing # of misses relative to the total # of objects.
        // * Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            aim_value *= 0.97
                * (1.0 - (self.effective_miss_count / self.total_hits).powf(0.775))
                    .powf(self.effective_miss_count);
        }

        aim_value *= self.get_combo_scaling_factor();

        let ar_factor = if self.mods.rx() {
            0.0
        } else if self.attributes.ar > 10.33 {
            0.3 * (self.attributes.ar - 10.33)
        } else if self.attributes.ar < 8.0 {
            0.05 * (8.0 - self.attributes.ar)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        aim_value *= 1.0 + ar_factor * len_bonus;

        if self.mods.hd() {
            // * We want to give more reward for lower AR when it comes to aim and HD. This nerfs high AR and buffs lower AR.
            aim_value *= 1.0 + 0.04 * (12.0 - self.attributes.ar);
        }

        // * We assume 15% of sliders in a map are difficult since there's no way to tell from the performance calculator.
        let estimate_diff_sliders = self.attributes.n_sliders as f64 * 0.15;

        if self.attributes.n_sliders > 0 {
            let estimate_slider_ends_dropped = ((self.n100 + self.n50 + self.n_misses)
                .min(self.attributes.max_combo - self.combo)
                as f64)
                .clamp(0.0, estimate_diff_sliders);
            let slider_nerf_factor = (1.0 - self.attributes.slider_factor)
                * (1.0 - estimate_slider_ends_dropped / estimate_diff_sliders).powi(3)
                + self.attributes.slider_factor;

            aim_value *= slider_nerf_factor;
        }

        aim_value *= self.acc;
        // * It is important to consider accuracy difficulty when scaling with accuracy.
        aim_value *= 0.98 + self.attributes.od * self.attributes.od / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        let mut speed_value =
            (5.0 * (self.attributes.speed / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let len_bonus = 0.95
            + 0.4 * (self.total_hits / 2000.0).min(1.0)
            + (self.total_hits > 2000.0) as u8 as f64 * (self.total_hits / 2000.0).log10() * 0.5;

        speed_value *= len_bonus;

        // * Penalize misses by assessing # of misses relative to the total # of objects.
        // * Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            speed_value *= 0.97
                * (1.0 - (self.effective_miss_count / self.total_hits).powf(0.775))
                    .powf(self.effective_miss_count.powf(0.875));
        }

        speed_value *= self.get_combo_scaling_factor();

        let ar_factor = if self.attributes.ar > 10.33 {
            0.3 * (self.attributes.ar - 10.33)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        speed_value *= 1.0 + ar_factor * len_bonus;

        if self.mods.hd() {
            // * We want to give more reward for lower AR when it comes to aim and HD.
            // * This nerfs high AR and buffs lower AR.
            speed_value *= 1.0 + 0.04 * (12.0 - self.attributes.ar);
        }

        // * Calculate accuracy assuming the worst case scenario
        let relevant_total_diff = self.total_hits - self.attributes.speed_note_count;
        let relevant_n300 = (self.n300 as f64 - relevant_total_diff).max(0.0);
        let relevant_n100 =
            (self.n100 as f64 - (relevant_total_diff - self.n300 as f64).max(0.0)).max(0.0);
        let relevant_n50 = (self.n50 as f64
            - (relevant_total_diff - (self.n300 + self.n100) as f64).max(0.0))
        .max(0.0);

        let relevant_acc = if self.attributes.speed_note_count.abs() <= f64::EPSILON {
            0.0
        } else {
            (relevant_n300 * 6.0 + relevant_n100 * 2.0 + relevant_n50)
                / (self.attributes.speed_note_count * 6.0)
        };

        // * Scale the speed value with accuracy and OD.
        speed_value *= (0.95 + self.attributes.od * self.attributes.od / 750.0)
            * ((self.acc + relevant_acc) / 2.0).powf((14.5 - (self.attributes.od).max(8.0)) / 2.0);

        // * Scale the speed value with # of 50s to punish doubletapping.
        speed_value *= 0.99_f64.powf(
            (self.n50 as f64 >= self.total_hits / 500.0) as u8 as f64
                * (self.n50 as f64 - self.total_hits / 500.0),
        );

        speed_value
    }

    fn compute_accuracy_value(&self) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        // * This percentage only considers HitCircles of any value - in this part
        // * of the calculation we focus on hitting the timing hit window.
        let amount_hit_objects_with_acc = self.attributes.n_circles;

        let mut better_acc_percentage = if amount_hit_objects_with_acc > 0 {
            ((self.n300 - (self.total_hits as usize - amount_hit_objects_with_acc)) * 6
                + self.n100 * 2
                + self.n50) as f64
                / (amount_hit_objects_with_acc * 6) as f64
        } else {
            0.0
        };

        // * It is possible to reach a negative accuracy with this formula. Cap it at zero - zero points.
        if better_acc_percentage < 0.0 {
            better_acc_percentage = 0.0;
        }

        // * Lots of arbitrary values from testing.
        // * Considering to use derivation from perfect accuracy in a probabilistic manner - assume normal distribution.
        let mut acc_value =
            1.52163_f64.powf(self.attributes.od) * better_acc_percentage.powi(24) * 2.83;

        // * Bonus for many hitcircles - it's harder to keep good accuracy up for longer.
        acc_value *= (amount_hit_objects_with_acc as f64 / 1000.0)
            .powf(0.3)
            .min(1.15);

        // * Increasing the accuracy value by object count for Blinds isn't ideal, so the minimum buff is given.
        if self.mods.hd() {
            acc_value *= 1.08;
        }

        if self.mods.fl() {
            acc_value *= 1.02;
        }

        acc_value
    }

    fn compute_flashlight_value(&self) -> f64 {
        if !self.mods.fl() {
            return 0.0;
        }

        let mut flashlight_value = self.attributes.flashlight * self.attributes.flashlight * 25.0;

        // * Penalize misses by assessing # of misses relative to the total # of objects. Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            flashlight_value *= 0.97
                * (1.0 - (self.effective_miss_count / self.total_hits).powf(0.775))
                    .powf(self.effective_miss_count.powf(0.875));
        }

        flashlight_value *= self.get_combo_scaling_factor();

        // * Account for shorter maps having a higher ratio of 0 combo/100 combo flashlight radius.
        flashlight_value *= 0.7
            + 0.1 * (self.total_hits / 200.0).min(1.0)
            + (self.total_hits > 200.0) as u8 as f64
                * 0.2
                * ((self.total_hits - 200.0) / 200.0).min(1.0);

        // * Scale the flashlight value with accuracy _slightly_.
        flashlight_value *= 0.5 + self.acc / 2.0;
        // * It is important to also consider accuracy difficulty when doing that.
        flashlight_value *= 0.98 + self.attributes.od * self.attributes.od / 2500.0;

        flashlight_value
    }

    fn get_combo_scaling_factor(&self) -> f64 {
        if self.attributes.max_combo == 0 {
            1.0
        } else {
            ((self.combo as f64).powf(0.8) / (self.attributes.max_combo as f64).powf(0.8)).min(1.0)
        }
    }
}

fn calculate_effective_misses(
    attrs: &OsuDifficultyAttributes,
    combo: Option<usize>,
    n100: usize,
    n50: usize,
    n_misses: usize,
) -> f64 {
    // * Guess the number of misses + slider breaks from combo
    let mut combo_based_miss_count = 0.0;

    if attrs.n_sliders > 0 {
        let full_combo_threshold = attrs.max_combo as f64 - 0.1 * attrs.n_sliders as f64;

        if let Some(score_max_combo) = combo
            .map(|combo| combo as f64)
            .filter(|&combo| combo < full_combo_threshold)
        {
            combo_based_miss_count = full_combo_threshold / score_max_combo.max(1.0);
        }
    }

    // * Clamp miss count to maximum amount of possible breaks
    combo_based_miss_count = combo_based_miss_count.min((n100 + n50 + n_misses) as f64);

    combo_based_miss_count.max(n_misses as f64)
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait OsuAttributeProvider {
    /// Provide the actual difficulty attributes.
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

impl OsuAttributeProvider for DifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<OsuDifficultyAttributes> {
        #[allow(irrefutable_let_patterns)]
        if let Self::Osu(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }
}

impl OsuAttributeProvider for PerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<OsuDifficultyAttributes> {
        #[allow(irrefutable_let_patterns)]
        if let Self::Osu(attributes) = self {
            Some(attributes.difficulty)
        } else {
            None
        }
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
        let acc = 100.0 * numerator as f64 / denominator as f64;

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
        let acc = 100.0 * numerator as f64 / denominator as f64;

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
        let attributes = OsuDifficultyAttributes::default();

        let total_objects = 1234;
        let n300 = 1000;
        let n100 = 200;
        let n50 = 30;

        let calculator = OsuPP::new(&map)
            .passed_objects(total_objects)
            .n300(n300)
            .n100(n100)
            .n50(n50)
            .assert_hitresults(attributes);

        let n_objects = calculator.n300 + calculator.n100 + calculator.n50;

        assert_eq!(
            total_objects, n_objects,
            "Expected: {} | Actual: {}",
            total_objects, n_objects
        );
    }
}
