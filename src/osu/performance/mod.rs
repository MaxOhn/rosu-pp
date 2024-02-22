use std::cmp;

use rosu_map::section::general::GameMode;

use crate::{
    any::ModeDifficulty,
    any::{HitResultPriority, ModeAttributeProvider, Performance},
    catch::CatchPerformance,
    mania::ManiaPerformance,
    taiko::TaikoPerformance,
    util::{float_ext::FloatExt, map_or_attrs::MapOrAttrs, mods::Mods},
};

use super::{
    attributes::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    convert::OsuBeatmap,
    score_state::OsuScoreState,
    Osu,
};

pub mod gradual;

/// Performance calculator on osu!standard maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct OsuPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, Osu>,
    pub(crate) mods: u32,
    pub(crate) acc: Option<f64>,
    pub(crate) combo: Option<u32>,

    pub(crate) n300: Option<u32>,
    pub(crate) n100: Option<u32>,
    pub(crate) n50: Option<u32>,
    pub(crate) n_misses: Option<u32>,
    pub(crate) passed_objects: Option<u32>,
    pub(crate) clock_rate: Option<f64>,
    pub(crate) hitresult_priority: HitResultPriority,
}

impl<'map> OsuPerformance<'map> {
    /// Create a new performance calculator for osu!standard maps.
    pub fn new(map: OsuBeatmap<'map>) -> Self {
        map.into()
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// Returns `None` if the internal beatmap was already replaced with
    /// [`OsuDifficultyAttributes`], i.e. if [`OsuPerformance::attributes`] or
    /// [`OsuPerformance::generate_state`] was called.
    ///
    /// If the given mode should be ignored in case the internal beatmap was
    /// replaced, use [`mode_or_ignore`] instead.
    ///
    /// [`mode_or_ignore`]: Self::mode_or_ignore
    pub fn try_mode(self, mode: GameMode) -> Option<Performance<'map>> {
        match mode {
            GameMode::Osu => Some(Performance::Osu(self)),
            GameMode::Taiko => TaikoPerformance::try_from(self)
                .map(Performance::Taiko)
                .ok(),
            GameMode::Catch => CatchPerformance::try_from(self)
                .map(Performance::Catch)
                .ok(),
            GameMode::Mania => ManiaPerformance::try_from(self)
                .map(Performance::Mania)
                .ok(),
        }
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// If the internal beatmap was already replaced with difficulty
    /// attributes, the map won't be modified.
    ///
    /// To see whether the internal beatmap was replaced, use [`try_mode`]
    /// instead.
    ///
    /// [`try_mode`]: Self::try_mode
    pub fn mode_or_ignore(self, mode: GameMode) -> Performance<'map> {
        match mode {
            GameMode::Osu => Performance::Osu(self),
            GameMode::Taiko => {
                TaikoPerformance::try_from(self).map_or_else(Performance::Osu, Performance::Taiko)
            }
            GameMode::Catch => {
                CatchPerformance::try_from(self).map_or_else(Performance::Osu, Performance::Catch)
            }
            GameMode::Mania => {
                ManiaPerformance::try_from(self).map_or_else(Performance::Osu, Performance::Mania)
            }
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    pub fn attributes(mut self, attributes: impl ModeAttributeProvider<Osu>) -> Self {
        if let Some(attrs) = attributes.attributes() {
            self.map_or_attrs = MapOrAttrs::Attrs(attrs);
        }

        self
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    pub const fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Specify the max combo of the play.
    pub const fn combo(mut self, combo: u32) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify how hitresults should be generated.
    ///
    /// Defauls to [`HitResultPriority::BestCase`].
    pub const fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = priority;

        self
    }

    /// Specify the amount of 300s of a play.
    pub const fn n300(mut self, n300: u32) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 100s of a play.
    pub const fn n100(mut self, n100: u32) -> Self {
        self.n100 = Some(n100);

        self
    }

    /// Specify the amount of 50s of a play.
    pub const fn n50(mut self, n50: u32) -> Self {
        self.n50 = Some(n50);

        self
    }

    /// Specify the amount of misses of a play.
    pub const fn n_misses(mut self, n_misses: u32) -> Self {
        self.n_misses = Some(n_misses);

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    #[cfg_attr(
        feature = "gradual",
        doc = "If you want to calculate the performance after every few objects, instead of
        using [`OsuPP`] multiple times with different `passed_objects`, you should use
        [`OsuGradualPerformanceAttributes`](crate::osu::OsuGradualPerformance)."
    )]
    pub const fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    pub const fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Provide parameters through an [`OsuScoreState`].
    #[allow(clippy::needless_pass_by_value)]
    pub const fn state(mut self, state: OsuScoreState) -> Self {
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
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

        self
    }

    /// Create the [`OsuScoreState`] that will be used for performance calculation.
    #[allow(clippy::too_many_lines)]
    pub fn generate_state(&mut self) -> OsuScoreState {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.generate_attributes(map);

                self.map_or_attrs.attrs_or_insert(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let max_combo = attrs.max_combo;
        let n_objects = self.passed_objects.unwrap_or(attrs.n_objects());
        let priority = self.hitresult_priority;

        let n_misses = self.n_misses.map_or(0, |n| n.min(n_objects));
        let n_remaining = n_objects - n_misses;

        let mut n300 = self.n300.map_or(0, |n| n.min(n_remaining));
        let mut n100 = self.n100.map_or(0, |n| n.min(n_remaining));
        let mut n50 = self.n50.map_or(0, |n| n.min(n_remaining));

        if let Some(acc) = self.acc {
            let target_total = acc * f64::from(6 * n_objects);

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

                    let raw_n100 = target_total - f64::from(n_remaining + 6 * n300);
                    let min_n100 = n_remaining.min(raw_n100.floor() as u32);
                    let max_n100 = n_remaining.min(raw_n100.ceil() as u32);

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

                    let raw_n300 = (target_total - f64::from(n_remaining + 2 * n100)) / 5.0;
                    let min_n300 = n_remaining.min(raw_n300.floor() as u32);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as u32);

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

                    let raw_n300 = (target_total + f64::from(2 * n_misses + n50)
                        - f64::from(2 * n_objects))
                        / 4.0;

                    let min_n300 = n_remaining.min(raw_n300.floor() as u32);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as u32);

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

                    let raw_n300 = (target_total - f64::from(n_remaining)) / 5.0;
                    let min_n300 = cmp::min(n_remaining, raw_n300.floor() as u32);
                    let max_n300 = cmp::min(n_remaining, raw_n300.ceil() as u32);

                    for new300 in min_n300..=max_n300 {
                        let raw_n100 = target_total - f64::from(n_remaining + 5 * new300);
                        let min_n100 = cmp::min(raw_n100.floor() as u32, n_remaining - new300);
                        let max_n100 = cmp::min(raw_n100.ceil() as u32, n_remaining - new300);

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
        let state = self.generate_state();

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => self.generate_attributes(map),
            MapOrAttrs::Attrs(attrs) => attrs,
        };

        let effective_miss_count = calculate_effective_misses(&attrs, &state);

        let inner = OsuPerformanceInner {
            attrs,
            mods: self.mods,
            acc: state.accuracy(),
            state,
            effective_miss_count,
        };

        inner.calculate()
    }

    fn generate_attributes(&self, map: &OsuBeatmap<'_>) -> OsuDifficultyAttributes {
        let mut calculator = ModeDifficulty::new();

        if let Some(passed_objects) = self.passed_objects {
            calculator = calculator.passed_objects(passed_objects);
        }

        if let Some(clock_rate) = self.clock_rate {
            calculator = calculator.clock_rate(clock_rate);
        }

        calculator.mods(self.mods).calculate(map)
    }

    /// Try to create [`OsuPerformance`] through a [`ModeAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`OsuBeatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`OsuPerformance::new`].
    ///
    /// Returns `None` only if the [`ModeAttributeProvider`] did not contain
    /// attributes for osu e.g. if it's [`DifficultyAttributes::Taiko`].
    ///
    /// [`DifficultyAttributes::Taiko`]: crate::any::DifficultyAttributes::Taiko
    pub fn try_from_attributes(attributes: impl ModeAttributeProvider<Osu>) -> Option<Self> {
        attributes.attributes().map(Self::from)
    }

    /// Create [`OsuPerformance`] through a [`ModeAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`OsuBeatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`OsuPerformance::new`].
    ///
    /// # Panics
    ///
    /// Panics if the [`ModeAttributeProvider`] did not contain attributes for
    /// osu e.g. if it's [`DifficultyAttributes::Taiko`].
    ///
    /// [`DifficultyAttributes::Taiko`]: crate::any::DifficultyAttributes::Taiko
    pub fn unchecked_from_attributes(attributes: impl ModeAttributeProvider<Osu>) -> Self {
        Self::try_from_attributes(attributes).expect("invalid osu attributes")
    }
}

impl<'map> From<OsuBeatmap<'map>> for OsuPerformance<'map> {
    fn from(map: OsuBeatmap<'map>) -> Self {
        Self {
            map_or_attrs: MapOrAttrs::Map(map),
            mods: 0,
            acc: None,
            combo: None,

            n300: None,
            n100: None,
            n50: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
            hitresult_priority: HitResultPriority::default(),
        }
    }
}

impl From<OsuDifficultyAttributes> for OsuPerformance<'_> {
    fn from(attrs: OsuDifficultyAttributes) -> Self {
        Self {
            map_or_attrs: MapOrAttrs::Attrs(attrs),
            mods: 0,
            acc: None,
            combo: None,

            n300: None,
            n100: None,
            n50: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
            hitresult_priority: HitResultPriority::default(),
        }
    }
}

impl From<OsuPerformanceAttributes> for OsuPerformance<'_> {
    fn from(attrs: OsuPerformanceAttributes) -> Self {
        attrs.difficulty.into()
    }
}

pub const PERFORMANCE_BASE_MULTIPLIER: f64 = 1.14;

struct OsuPerformanceInner {
    attrs: OsuDifficultyAttributes,
    mods: u32,
    acc: f64,
    state: OsuScoreState,
    effective_miss_count: f64,
}

impl OsuPerformanceInner {
    fn calculate(mut self) -> OsuPerformanceAttributes {
        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return OsuPerformanceAttributes {
                difficulty: self.attrs,
                ..Default::default()
            };
        }

        let total_hits = f64::from(total_hits);

        let mut multiplier = PERFORMANCE_BASE_MULTIPLIER;

        if self.mods.nf() {
            multiplier *= (1.0 - 0.02 * self.effective_miss_count).max(0.9);
        }

        if self.mods.so() && total_hits > 0.0 {
            multiplier *= 1.0 - (f64::from(self.attrs.n_spinners) / total_hits).powf(0.85);
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
                + f64::from(self.state.n100)
                + n100_mult
                + f64::from(self.state.n50) * n50_mult)
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
            + f64::from(u8::from(total_hits > 2000.0)) * (total_hits / 2000.0).log10() * 0.5;

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
        let estimate_diff_sliders = f64::from(self.attrs.n_sliders) * 0.15;

        if self.attrs.n_sliders > 0 {
            let estimate_slider_ends_dropped = f64::from(
                (self.state.n100 + self.state.n50 + self.state.n_misses)
                    .min(self.attrs.max_combo.saturating_sub(self.state.max_combo)),
            )
            .clamp(0.0, estimate_diff_sliders);
            let slider_nerf_factor = (1.0 - self.attrs.slider_factor)
                * (1.0 - estimate_slider_ends_dropped / estimate_diff_sliders).powi(3)
                + self.attrs.slider_factor;

            aim_value *= slider_nerf_factor;
        }

        aim_value *= self.acc;
        // * It is important to consider accuracy difficulty when scaling with accuracy.
        aim_value *= 0.98 + self.attrs.od.powi(2) / 2500.0;

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
            + f64::from(u8::from(total_hits > 2000.0)) * (total_hits / 2000.0).log10() * 0.5;

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
        let relevant_n300 = (f64::from(self.state.n300) - relevant_total_diff).max(0.0);
        let relevant_n100 = (f64::from(self.state.n100)
            - (relevant_total_diff - f64::from(self.state.n300)).max(0.0))
        .max(0.0);
        let relevant_n50 = (f64::from(self.state.n50)
            - (relevant_total_diff - f64::from(self.state.n300 + self.state.n100)).max(0.0))
        .max(0.0);

        let relevant_acc = if self.attrs.speed_note_count.eq(0.0) {
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
            f64::from(u8::from(f64::from(self.state.n50) >= total_hits / 500.0))
                * (f64::from(self.state.n50) - total_hits / 500.0),
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
                f64::from((self.state.n300 - sub) * 6 + self.state.n100 * 2 + self.state.n50)
                    / f64::from(amount_hit_objects_with_acc * 6)
            }
        } else {
            0.0
        };

        // * Lots of arbitrary values from testing.
        // * Considering to use derivation from perfect accuracy in a probabilistic manner - assume normal distribution.
        let mut acc_value = 1.52163_f64.powf(self.attrs.od) * better_acc_percentage.powi(24) * 2.83;

        // * Bonus for many hitcircles - it's harder to keep good accuracy up for longer.
        acc_value *= (f64::from(amount_hit_objects_with_acc) / 1000.0)
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

        let mut flashlight_value = self.attrs.flashlight.powi(2) * 25.0;

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
            + f64::from(u8::from(total_hits > 200.0))
                * 0.2
                * ((total_hits - 200.0) / 200.0).min(1.0);

        // * Scale the flashlight value with accuracy _slightly_.
        flashlight_value *= 0.5 + self.acc / 2.0;
        // * It is important to also consider accuracy difficulty when doing that.
        flashlight_value *= 0.98 + self.attrs.od.powi(2) / 2500.0;

        flashlight_value
    }

    fn get_combo_scaling_factor(&self) -> f64 {
        if self.attrs.max_combo == 0 {
            1.0
        } else {
            (f64::from(self.state.max_combo).powf(0.8) / f64::from(self.attrs.max_combo).powf(0.8))
                .min(1.0)
        }
    }

    const fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }
}

fn calculate_effective_misses(attrs: &OsuDifficultyAttributes, state: &OsuScoreState) -> f64 {
    // * Guess the number of misses + slider breaks from combo
    let mut combo_based_miss_count = 0.0;

    if attrs.n_sliders > 0 {
        let full_combo_threshold = f64::from(attrs.max_combo) - 0.1 * f64::from(attrs.n_sliders);

        if f64::from(state.max_combo) < full_combo_threshold {
            combo_based_miss_count = full_combo_threshold / f64::from(state.max_combo).max(1.0);
        }
    }

    // * Clamp miss count to maximum amount of possible breaks
    combo_based_miss_count =
        combo_based_miss_count.min(f64::from(state.n100 + state.n50 + state.n_misses));

    combo_based_miss_count.max(f64::from(state.n_misses))
}

fn accuracy(n300: u32, n100: u32, n50: u32, n_misses: u32) -> f64 {
    if n300 + n100 + n50 + n_misses == 0 {
        return 0.0;
    }

    let numerator = 6 * n300 + 2 * n100 + n50;
    let denominator = 6 * (n300 + n100 + n50 + n_misses);

    f64::from(numerator) / f64::from(denominator)
}
