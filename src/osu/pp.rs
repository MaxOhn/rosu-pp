use super::{
    OsuDifficultyAttributes, OsuPerformanceAttributes, OsuScoreState, PERFORMANCE_BASE_MULTIPLIER,
};
use crate::{
    AnyPP, Beatmap, DifficultyAttributes, GameMode, HitResultPriority, Mods, OsuStars,
    PerformanceAttributes,
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
///     .accuracy(98.5)
///     .n_misses(1)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = OsuPP::new(&map)
///     .attributes(pp_result) // reusing previous results for performance
///     .mods(8 + 64)  // has to be the same to reuse attributes
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
    pub(crate) n_misses: Option<usize>,
    pub(crate) passed_objects: Option<usize>,
    pub(crate) clock_rate: Option<f64>,
    pub(crate) hitresult_priority: Option<HitResultPriority>,
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
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
            hitresult_priority: None,
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
            self.attributes = Some(attributes);
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
        self.combo = Some(combo);

        self
    }

    /// Specify how hitresults should be generated.
    ///
    /// Defauls to [`HitResultPriority::BestCase`].
    #[inline]
    pub fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = Some(priority);

        self
    }

    /// Specify the amount of 300s of a play.
    #[inline]
    pub fn n300(mut self, n300: usize) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 100s of a play.
    #[inline]
    pub fn n100(mut self, n100: usize) -> Self {
        self.n100 = Some(n100);

        self
    }

    /// Specify the amount of 50s of a play.
    #[inline]
    pub fn n50(mut self, n50: usize) -> Self {
        self.n50 = Some(n50);

        self
    }

    /// Specify the amount of misses of a play.
    #[inline]
    pub fn n_misses(mut self, n_misses: usize) -> Self {
        self.n_misses = Some(n_misses);

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`OsuPP`] multiple times with different `passed_objects`, you should use
    /// [`OsuGradualPerformanceAttributes`](crate::osu::OsuGradualPerformance).
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects = Some(passed_objects);

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
        self.n_misses = Some(n_misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    #[inline]
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

        self
    }

    fn generate_hitresults(&self, max_combo: usize) -> OsuScoreState {
        let n_objects = self.passed_objects.unwrap_or(self.map.hit_objects.len());
        let priority = self.hitresult_priority.unwrap_or_default();

        let n_misses = self.n_misses.map_or(0, |n| n.min(n_objects));
        let n_remaining = n_objects - n_misses;

        let mut n300 = self.n300.map_or(0, |n| n.min(n_remaining));
        let mut n100 = self.n100.map_or(0, |n| n.min(n_remaining));
        let mut n50 = self.n50.map_or(0, |n| n.min(n_remaining));

        if let Some(acc) = self.acc {
            let target_total = acc * (6 * n_objects) as f64;

            match (self.n300, self.n100, self.n50) {
                (Some(_), Some(_), Some(_)) => {
                    let remaining = n_objects.saturating_sub(n300 + n100 + n50 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n300 += remaining,
                        HitResultPriority::WorstCase => n50 += remaining,
                    }
                }
                (Some(_), Some(_), None) => n50 = n_objects.saturating_sub(n300 + n100 + n_misses),
                (Some(_), None, Some(_)) => n100 = n_objects.saturating_sub(n300 + n50 + n_misses),
                (None, Some(_), Some(_)) => n300 = n_objects.saturating_sub(n100 + n50 + n_misses),
                (Some(_), None, None) => {
                    let mut best_dist = f64::MAX;

                    n300 = n300.min(n_remaining);
                    let n_remaining = n_remaining - n300;

                    let raw_n100 = target_total - (n_remaining + 6 * n300) as f64;
                    let min_n100 = n_remaining.min(raw_n100.floor() as usize);
                    let max_n100 = n_remaining.min(raw_n100.ceil() as usize);

                    for new100 in min_n100..=max_n100 {
                        let new50 = n_remaining - new100;
                        let dist = (acc - accuracy(n300, new100, new50, n_misses)).abs();

                        if dist < best_dist {
                            best_dist = dist;
                            n100 = new100;
                            n50 = new50;
                        }
                    }
                }
                (None, Some(_), None) => {
                    let mut best_dist = f64::MAX;

                    n100 = n100.min(n_remaining);
                    let n_remaining = n_remaining - n100;

                    let raw_n300 = (target_total - (n_remaining + 2 * n100) as f64) / 5.0;
                    let min_n300 = n_remaining.min(raw_n300.floor() as usize);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as usize);

                    for new300 in min_n300..=max_n300 {
                        let new50 = n_remaining - new300;
                        let curr_dist = (acc - accuracy(new300, n100, new50, n_misses)).abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n300 = new300;
                            n50 = new50;
                        }
                    }
                }
                (None, None, Some(_)) => {
                    let mut best_dist = f64::MAX;

                    n50 = n50.min(n_remaining);
                    let n_remaining = n_remaining - n50;

                    let raw_n300 =
                        (target_total + (2 * n_misses + n50) as f64 - (2 * n_objects) as f64) / 4.0;

                    let min_n300 = n_remaining.min(raw_n300.floor() as usize);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as usize);

                    for new300 in min_n300..=max_n300 {
                        let new100 = n_remaining - new300;
                        let curr_dist = (acc - accuracy(new300, new100, n50, n_misses)).abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n300 = new300;
                            n100 = new100;
                        }
                    }
                }
                (None, None, None) => {
                    let mut best_dist = f64::MAX;

                    let raw_n300 = (target_total - n_remaining as f64) / 5.0;
                    let min_n300 = n_remaining.min(raw_n300.floor() as usize);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as usize);

                    for new300 in min_n300..=max_n300 {
                        let raw_n100 = target_total - (n_remaining + 5 * new300) as f64;
                        let min_n100 = (raw_n100.floor() as usize).min(n_remaining - new300);
                        let max_n100 = (raw_n100.ceil() as usize).min(n_remaining - new300);

                        for new100 in min_n100..=max_n100 {
                            let new50 = n_remaining - new300 - new100;
                            let curr_dist = (acc - accuracy(new300, new100, new50, n_misses)).abs();

                            if curr_dist < best_dist {
                                best_dist = curr_dist;
                                n300 = new300;
                                n100 = new100;
                                n50 = new50;
                            }
                        }
                    }

                    match priority {
                        HitResultPriority::BestCase => {
                            // Shift n50 to n100 by sacrificing n300
                            let n = n300.min(n50 / 4);
                            n300 -= n;
                            n100 += 5 * n;
                            n50 -= 4 * n;
                        }
                        HitResultPriority::WorstCase => {
                            // Shift n100 to n50 by gaining n300
                            let n = n100 / 5;
                            n300 += n;
                            n100 -= 5 * n;
                            n50 += 4 * n;
                        }
                    }
                }
            }
        } else {
            let remaining = n_objects.saturating_sub(n300 + n100 + n50 + n_misses);

            match priority {
                HitResultPriority::BestCase => match (self.n300, self.n100, self.n50) {
                    (None, ..) => n300 = remaining,
                    (_, None, _) => n100 = remaining,
                    (.., None) => n50 = remaining,
                    _ => n300 += remaining,
                },
                HitResultPriority::WorstCase => match (self.n50, self.n100, self.n300) {
                    (None, ..) => n50 = remaining,
                    (_, None, _) => n100 = remaining,
                    (.., None) => n300 = remaining,
                    _ => n50 += remaining,
                },
            }
        }

        let max_possible_combo = max_combo.saturating_sub(n_misses);

        let max_combo = self
            .combo
            .map_or(max_possible_combo, |combo| combo.min(max_possible_combo));

        OsuScoreState {
            max_combo,
            n300,
            n100,
            n50,
            n_misses,
        }
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        let attrs = self.attributes.take().unwrap_or_else(|| {
            let mut calculator = OsuStars::new(self.map).mods(self.mods);

            if let Some(passed_objects) = self.passed_objects {
                calculator = calculator.passed_objects(passed_objects);
            }

            if let Some(clock_rate) = self.clock_rate {
                calculator = calculator.clock_rate(clock_rate);
            }

            calculator.calculate()
        });

        let state = self.generate_hitresults(attrs.max_combo);
        let effective_miss_count = calculate_effective_misses(&attrs, &state);

        let inner = OsuPpInner {
            attrs,
            mods: self.mods,
            acc: state.accuracy(),
            state,
            effective_miss_count,
        };

        inner.calculate()
    }
}

struct OsuPpInner {
    attrs: OsuDifficultyAttributes,
    mods: u32,
    acc: f64,
    state: OsuScoreState,
    effective_miss_count: f64,
}

impl OsuPpInner {
    fn calculate(mut self) -> OsuPerformanceAttributes {
        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return OsuPerformanceAttributes {
                difficulty: self.attrs,
                ..Default::default()
            };
        }

        let total_hits = total_hits as f64;

        let mut multiplier = PERFORMANCE_BASE_MULTIPLIER;

        if self.mods.nf() {
            multiplier *= (1.0 - 0.02 * self.effective_miss_count).max(0.9);
        }

        if self.mods.so() && total_hits > 0.0 {
            multiplier *= 1.0 - (self.attrs.n_spinners as f64 / total_hits).powf(0.85);
        }

        if self.mods.rx() {
            // * https://www.desmos.com/calculator/bc9eybdthb
            // * we use OD13.3 as maximum since it's the value at which great hitwidow becomes 0
            // * this is well beyond currently maximum achievable OD which is 12.17 (DTx2 + DA with OD11)
            let (n100_mult, n50_mult) = if self.attrs.od > 0.0 {
                (
                    1.0 - (self.attrs.od / 13.33).powf(1.8),
                    1.0 - (self.attrs.od / 13.33).powi(5),
                )
            } else {
                (1.0, 1.0)
            };

            // * As we're adding Oks and Mehs to an approximated number of combo breaks the result can be
            // * higher than total hits in specific scenarios (which breaks some calculations) so we need to clamp it.
            self.effective_miss_count = (self.effective_miss_count
                + self.state.n100 as f64
                + n100_mult
                + self.state.n50 as f64 * n50_mult)
                .min(total_hits);
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
            difficulty: self.attrs,
            pp_acc: acc_value,
            pp_aim: aim_value,
            pp_flashlight: flashlight_value,
            pp_speed: speed_value,
            pp,
            effective_miss_count: self.effective_miss_count,
        }
    }

    fn compute_aim_value(&self) -> f64 {
        let mut aim_value = (5.0 * (self.attrs.aim / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let total_hits = self.total_hits();

        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f64 * (total_hits / 2000.0).log10() * 0.5;

        aim_value *= len_bonus;

        // * Penalize misses by assessing # of misses relative to the total # of objects.
        // * Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            aim_value *= 0.97
                * (1.0 - (self.effective_miss_count / total_hits).powf(0.775))
                    .powf(self.effective_miss_count);
        }

        aim_value *= self.get_combo_scaling_factor();

        let ar_factor = if self.mods.rx() {
            0.0
        } else if self.attrs.ar > 10.33 {
            0.3 * (self.attrs.ar - 10.33)
        } else if self.attrs.ar < 8.0 {
            0.05 * (8.0 - self.attrs.ar)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        aim_value *= 1.0 + ar_factor * len_bonus;

        if self.mods.hd() {
            // * We want to give more reward for lower AR when it comes to aim and HD. This nerfs high AR and buffs lower AR.
            aim_value *= 1.0 + 0.04 * (12.0 - self.attrs.ar);
        }

        // * We assume 15% of sliders in a map are difficult since there's no way to tell from the performance calculator.
        let estimate_diff_sliders = self.attrs.n_sliders as f64 * 0.15;

        if self.attrs.n_sliders > 0 {
            let estimate_slider_ends_dropped =
                ((self.state.n100 + self.state.n50 + self.state.n_misses)
                    .min(self.attrs.max_combo.saturating_sub(self.state.max_combo))
                    as f64)
                    .clamp(0.0, estimate_diff_sliders);
            let slider_nerf_factor = (1.0 - self.attrs.slider_factor)
                * (1.0 - estimate_slider_ends_dropped / estimate_diff_sliders).powi(3)
                + self.attrs.slider_factor;

            aim_value *= slider_nerf_factor;
        }

        aim_value *= self.acc;
        // * It is important to consider accuracy difficulty when scaling with accuracy.
        aim_value *= 0.98 + self.attrs.od * self.attrs.od / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        let mut speed_value =
            (5.0 * (self.attrs.speed / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let total_hits = self.total_hits();

        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f64 * (total_hits / 2000.0).log10() * 0.5;

        speed_value *= len_bonus;

        // * Penalize misses by assessing # of misses relative to the total # of objects.
        // * Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            speed_value *= 0.97
                * (1.0 - (self.effective_miss_count / total_hits).powf(0.775))
                    .powf(self.effective_miss_count.powf(0.875));
        }

        speed_value *= self.get_combo_scaling_factor();

        let ar_factor = if self.attrs.ar > 10.33 {
            0.3 * (self.attrs.ar - 10.33)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        speed_value *= 1.0 + ar_factor * len_bonus;

        if self.mods.hd() {
            // * We want to give more reward for lower AR when it comes to aim and HD.
            // * This nerfs high AR and buffs lower AR.
            speed_value *= 1.0 + 0.04 * (12.0 - self.attrs.ar);
        }

        // * Calculate accuracy assuming the worst case scenario
        let relevant_total_diff = total_hits - self.attrs.speed_note_count;
        let relevant_n300 = (self.state.n300 as f64 - relevant_total_diff).max(0.0);
        let relevant_n100 = (self.state.n100 as f64
            - (relevant_total_diff - self.state.n300 as f64).max(0.0))
        .max(0.0);
        let relevant_n50 = (self.state.n50 as f64
            - (relevant_total_diff - (self.state.n300 + self.state.n100) as f64).max(0.0))
        .max(0.0);

        let relevant_acc = if self.attrs.speed_note_count.abs() <= f64::EPSILON {
            0.0
        } else {
            (relevant_n300 * 6.0 + relevant_n100 * 2.0 + relevant_n50)
                / (self.attrs.speed_note_count * 6.0)
        };

        // * Scale the speed value with accuracy and OD.
        speed_value *= (0.95 + self.attrs.od * self.attrs.od / 750.0)
            * ((self.acc + relevant_acc) / 2.0).powf((14.5 - (self.attrs.od).max(8.0)) / 2.0);

        // * Scale the speed value with # of 50s to punish doubletapping.
        speed_value *= 0.99_f64.powf(
            (self.state.n50 as f64 >= total_hits / 500.0) as u8 as f64
                * (self.state.n50 as f64 - total_hits / 500.0),
        );

        speed_value
    }

    fn compute_accuracy_value(&self) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        // * This percentage only considers HitCircles of any value - in this part
        // * of the calculation we focus on hitting the timing hit window.
        let amount_hit_objects_with_acc = self.attrs.n_circles;

        let better_acc_percentage = if amount_hit_objects_with_acc > 0 {
            let sub = self.state.total_hits() - amount_hit_objects_with_acc;

            // * It is possible to reach a negative accuracy with this formula. Cap it at zero - zero points.
            if self.state.n300 < sub {
                0.0
            } else {
                ((self.state.n300 - sub) * 6 + self.state.n100 * 2 + self.state.n50) as f64
                    / (amount_hit_objects_with_acc * 6) as f64
            }
        } else {
            0.0
        };

        // * Lots of arbitrary values from testing.
        // * Considering to use derivation from perfect accuracy in a probabilistic manner - assume normal distribution.
        let mut acc_value = 1.52163_f64.powf(self.attrs.od) * better_acc_percentage.powi(24) * 2.83;

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

        let mut flashlight_value = self.attrs.flashlight * self.attrs.flashlight * 25.0;

        let total_hits = self.total_hits();

        // * Penalize misses by assessing # of misses relative to the total # of objects. Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            flashlight_value *= 0.97
                * (1.0 - (self.effective_miss_count / total_hits).powf(0.775))
                    .powf(self.effective_miss_count.powf(0.875));
        }

        flashlight_value *= self.get_combo_scaling_factor();

        // * Account for shorter maps having a higher ratio of 0 combo/100 combo flashlight radius.
        flashlight_value *= 0.7
            + 0.1 * (total_hits / 200.0).min(1.0)
            + (total_hits > 200.0) as u8 as f64 * 0.2 * ((total_hits - 200.0) / 200.0).min(1.0);

        // * Scale the flashlight value with accuracy _slightly_.
        flashlight_value *= 0.5 + self.acc / 2.0;
        // * It is important to also consider accuracy difficulty when doing that.
        flashlight_value *= 0.98 + self.attrs.od * self.attrs.od / 2500.0;

        flashlight_value
    }

    fn get_combo_scaling_factor(&self) -> f64 {
        if self.attrs.max_combo == 0 {
            1.0
        } else {
            ((self.state.max_combo as f64).powf(0.8) / (self.attrs.max_combo as f64).powf(0.8))
                .min(1.0)
        }
    }

    fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }
}

fn calculate_effective_misses(attrs: &OsuDifficultyAttributes, state: &OsuScoreState) -> f64 {
    // * Guess the number of misses + slider breaks from combo
    let mut combo_based_miss_count = 0.0;

    if attrs.n_sliders > 0 {
        let full_combo_threshold = attrs.max_combo as f64 - 0.1 * attrs.n_sliders as f64;

        if (state.max_combo as f64) < full_combo_threshold {
            combo_based_miss_count = full_combo_threshold / (state.max_combo as f64).max(1.0);
        }
    }

    // * Clamp miss count to maximum amount of possible breaks
    combo_based_miss_count =
        combo_based_miss_count.min((state.n100 + state.n50 + state.n_misses) as f64);

    combo_based_miss_count.max(state.n_misses as f64)
}

fn accuracy(n300: usize, n100: usize, n50: usize, n_misses: usize) -> f64 {
    debug_assert_ne!(n300 + n100 + n50 + n_misses, 0);

    let numerator = 6 * n300 + 2 * n100 + n50;
    let denominator = 6 * (n300 + n100 + n50 + n_misses);

    numerator as f64 / denominator as f64
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

#[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
#[cfg(test)]
mod test {
    use super::*;
    use crate::Beatmap;
    use proptest::{option, prelude::*};
    use std::sync::OnceLock;

    static DATA: OnceLock<(Beatmap, OsuDifficultyAttributes)> = OnceLock::new();

    const N_OBJECTS: usize = 601;

    fn test_data() -> (&'static Beatmap, OsuDifficultyAttributes) {
        let (map, attrs) = DATA.get_or_init(|| {
            let path = "./maps/2785319.osu";
            let map = Beatmap::from_path(path).unwrap();

            let attrs = OsuDifficultyAttributes {
                aim: 2.8693628443424104,
                speed: 2.533869745015772,
                flashlight: 2.288770487900865,
                slider_factor: 0.9803052946037858,
                speed_note_count: 210.36373973116545,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 5.669858729379631,
                max_combo: 909,
            };

            assert_eq!(
                N_OBJECTS,
                attrs.n_circles + attrs.n_sliders + attrs.n_spinners
            );

            (map, attrs)
        });

        (map, attrs.to_owned())
    }

    /// Checks all remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`OsuScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate.
    fn brute_force_best(
        acc: f64,
        n300: Option<usize>,
        n100: Option<usize>,
        n50: Option<usize>,
        n_misses: usize,
        best_case: bool,
    ) -> OsuScoreState {
        let mut best_state = OsuScoreState {
            n_misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let n_remaining = N_OBJECTS - n_misses;

        let (min_n300, max_n300) = match (n300, n100, n50) {
            (Some(n300), ..) => (n_remaining.min(n300), n_remaining.min(n300)),
            (None, Some(n100), Some(n50)) => (
                n_remaining.saturating_sub(n100 + n50),
                n_remaining.saturating_sub(n100 + n50),
            ),
            (None, ..) => (
                0,
                n_remaining.saturating_sub(n100.unwrap_or(0) + n50.unwrap_or(0)),
            ),
        };

        for new300 in min_n300..=max_n300 {
            let (min_n100, max_n100) = match (n100, n50) {
                (Some(n100), _) => (n_remaining.min(n100), n_remaining.min(n100)),
                (None, Some(n50)) => (
                    n_remaining.saturating_sub(new300 + n50),
                    n_remaining.saturating_sub(new300 + n50),
                ),
                (None, None) => (0, n_remaining - new300),
            };

            for new100 in min_n100..=max_n100 {
                let new50 = match n50 {
                    Some(n50) => n_remaining.min(n50),
                    None => n_remaining.saturating_sub(new300 + new100),
                };

                let curr_acc = accuracy(new300, new100, new50, n_misses);
                let curr_dist = (acc - curr_acc).abs();

                if curr_dist < best_dist {
                    best_dist = curr_dist;
                    best_state.n300 = new300;
                    best_state.n100 = new100;
                    best_state.n50 = new50;
                }
            }
        }

        if best_state.n300 + best_state.n100 + best_state.n50 < n_remaining {
            let remaining = n_remaining - (best_state.n300 + best_state.n100 + best_state.n50);

            if best_case {
                best_state.n300 += remaining;
            } else {
                best_state.n50 += remaining;
            }
        }

        if n300.is_none() && n100.is_none() && n50.is_none() {
            if best_case {
                let n = best_state.n300.min(best_state.n50 / 4);
                best_state.n300 -= n;
                best_state.n100 += 5 * n;
                best_state.n50 -= 4 * n;
            } else {
                let n = best_state.n100 / 5;
                best_state.n300 += n;
                best_state.n100 -= 5 * n;
                best_state.n50 += 4 * n;
            }
        }

        best_state
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn osu_hitresults(
            acc in 0.0..=1.0,
            n300 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n100 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n50 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n_misses in option::weighted(0.15, 0_usize..=N_OBJECTS),
            best_case in prop::bool::ANY,
        ) {
            let (map, attrs) = test_data();
            let max_combo = attrs.max_combo();

            let priority = if best_case {
                HitResultPriority::BestCase
            } else {
                HitResultPriority::WorstCase
            };

            let mut state = OsuPP::new(map)
                .attributes(attrs)
                .accuracy(acc * 100.0)
                .hitresult_priority(priority);

            if let Some(n300) = n300 {
                state = state.n300(n300);
            }

            if let Some(n100) = n100 {
                state = state.n100(n100);
            }

            if let Some(n50) = n50 {
                state = state.n50(n50);
            }

            if let Some(n_misses) = n_misses {
                state = state.n_misses(n_misses);
            }

            let hitresults = state.generate_hitresults(max_combo);

            let mut expected = brute_force_best(
                acc,
                n300,
                n100,
                n50,
                n_misses.unwrap_or(0),
                best_case,
            );
            expected.max_combo = max_combo.saturating_sub(n_misses.unwrap_or(0));

            assert_eq!(hitresults, expected);
        }
    }

    #[test]
    fn hitresults_n300_n100_n_misses_best() {
        let (map, attrs) = test_data();
        let max_combo = attrs.max_combo();

        let state = OsuPP::new(map)
            .attributes(attrs)
            .combo(500)
            .n300(300)
            .n100(20)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults(max_combo);

        let expected = OsuScoreState {
            max_combo: 500,
            n300: 300,
            n100: 20,
            n50: 279,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn hitresults_n300_n50_n_misses_best() {
        let (map, attrs) = test_data();
        let max_combo = attrs.max_combo();

        let state = OsuPP::new(map)
            .attributes(attrs)
            .combo(500)
            .n300(300)
            .n50(10)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults(max_combo);

        let expected = OsuScoreState {
            max_combo: 500,
            n300: 300,
            n100: 289,
            n50: 10,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn hitresults_n50_n_misses_worst() {
        let (map, attrs) = test_data();
        let max_combo = attrs.max_combo();

        let state = OsuPP::new(map)
            .attributes(attrs)
            .combo(500)
            .n50(10)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_hitresults(max_combo);

        let expected = OsuScoreState {
            max_combo: 500,
            n300: 0,
            n100: 589,
            n50: 10,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn hitresults_n300_n100_n50_n_misses_worst() {
        let (map, attrs) = test_data();
        let max_combo = attrs.max_combo();

        let state = OsuPP::new(map)
            .attributes(attrs)
            .combo(500)
            .n300(300)
            .n100(50)
            .n50(10)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_hitresults(max_combo);

        let expected = OsuScoreState {
            max_combo: 500,
            n300: 300,
            n100: 50,
            n50: 249,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }
}
