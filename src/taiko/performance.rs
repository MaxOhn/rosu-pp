use std::cmp;

use crate::{
    any::ModeDifficulty,
    any::{HitResultPriority, ModeAttributeProvider},
    osu::OsuPerformance,
    util::{map_or_attrs::MapOrAttrs, mods::Mods},
};

use super::{
    attributes::{TaikoDifficultyAttributes, TaikoPerformanceAttributes},
    convert::TaikoBeatmap,
    score_state::TaikoScoreState,
    Taiko,
};

/// Performance calculator on osu!taiko maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TaikoPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, Taiko>,
    mods: u32,
    combo: Option<u32>,
    acc: Option<f64>,
    passed_objects: Option<u32>,
    clock_rate: Option<f64>,
    hitresult_priority: HitResultPriority,

    pub(crate) n300: Option<u32>,
    pub(crate) n100: Option<u32>,
    pub(crate) n_misses: Option<u32>,
}

impl<'map> TaikoPerformance<'map> {
    /// Create a new performance calculator for osu!taiko maps.
    pub fn new(map: TaikoBeatmap<'map>) -> Self {
        map.into()
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    pub fn attributes(mut self, attrs: impl ModeAttributeProvider<Taiko>) -> Self {
        if let Some(attrs) = attrs.attributes() {
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

    /// Specify the amount of misses of the play.
    pub const fn n_misses(mut self, n_misses: u32) -> Self {
        self.n_misses = Some(n_misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    #[cfg_attr(
        feature = "gradual",
        doc = "If you want to calculate the performance after every few objects, instead of
        using [`TaikoPP`] multiple times with different `passed_objects`, you should use
        [`TaikoGradualPerformanceAttributes`](crate::taiko::TaikoGradualPerformance)."
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

    /// Provide parameters through a [`TaikoScoreState`].
    #[allow(clippy::needless_pass_by_value)]
    pub const fn state(mut self, state: TaikoScoreState) -> Self {
        let TaikoScoreState {
            max_combo,
            n300,
            n100,
            n_misses,
        } = state;

        self.combo = Some(max_combo);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.n_misses = Some(n_misses);

        self
    }

    /// Create the [`TaikoScoreState`] that will be used for performance calculation.
    pub fn generate_state(&mut self) -> TaikoScoreState {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.generate_attributes(map);

                self.map_or_attrs.attrs_or_insert(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let max_combo = attrs.max_combo();

        let total_result_count = if let Some(passed_objects) = self.passed_objects {
            cmp::min(max_combo, passed_objects)
        } else {
            max_combo
        };

        let priority = self.hitresult_priority;

        let n_misses = self.n_misses.map_or(0, |n| cmp::min(n, total_result_count));
        let n_remaining = total_result_count - n_misses;

        let mut n300 = self.n300.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n100 = self.n100.map_or(0, |n| cmp::min(n, n_remaining));

        if let Some(acc) = self.acc {
            match (self.n300, self.n100) {
                (Some(_), Some(_)) => {
                    let remaining = total_result_count.saturating_sub(n300 + n100 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n300 += remaining,
                        HitResultPriority::WorstCase => n100 += remaining,
                    }
                }
                (Some(_), None) => n100 += total_result_count.saturating_sub(n300 + n_misses),
                (None, Some(_)) => n300 += total_result_count.saturating_sub(n100 + n_misses),
                (None, None) => {
                    let target_total = acc * f64::from(2 * total_result_count);

                    let mut best_dist = f64::MAX;

                    let raw_n300 = target_total - f64::from(n_remaining);
                    let min_n300 = n_remaining.min(raw_n300.floor() as u32);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as u32);

                    for new300 in min_n300..=max_n300 {
                        let new100 = n_remaining - new300;
                        let dist = (acc - accuracy(new300, new100, n_misses)).abs();

                        if dist < best_dist {
                            best_dist = dist;
                            n300 = new300;
                            n100 = new100;
                        }
                    }
                }
            }
        } else {
            let remaining = total_result_count.saturating_sub(n300 + n100 + n_misses);

            match priority {
                HitResultPriority::BestCase => match (self.n300, self.n100) {
                    (None, _) => n300 = remaining,
                    (_, None) => n100 = remaining,
                    _ => n300 += remaining,
                },
                HitResultPriority::WorstCase => match (self.n100, self.n300) {
                    (None, _) => n100 = remaining,
                    (_, None) => n300 = remaining,
                    _ => n100 += remaining,
                },
            }
        }

        let max_possible_combo = max_combo.saturating_sub(n_misses);

        let max_combo = self
            .combo
            .map_or(max_possible_combo, |combo| combo.min(max_possible_combo));

        TaikoScoreState {
            max_combo,
            n300,
            n100,
            n_misses,
        }
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> TaikoPerformanceAttributes {
        let state = self.generate_state();

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => self.generate_attributes(map),
            MapOrAttrs::Attrs(attrs) => attrs,
        };

        let inner = TaikoPerformanceInner {
            mods: self.mods,
            state,
            attrs,
        };

        inner.calculate()
    }

    fn generate_attributes(&self, map: &TaikoBeatmap<'_>) -> TaikoDifficultyAttributes {
        let mut calculator = ModeDifficulty::new();

        if let Some(passed_objects) = self.passed_objects {
            calculator = calculator.passed_objects(passed_objects);
        }

        if let Some(clock_rate) = self.clock_rate {
            calculator = calculator.clock_rate(clock_rate);
        }

        calculator.mods(self.mods).calculate(map)
    }

    /// Try to create [`TaikoPerformance`] through a [`ModeAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`TaikoBeatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`TaikoPerformance::new`].
    ///
    /// Returns `None` only if the [`ModeAttributeProvider`] did not contain
    /// attributes for taiko e.g. if it's [`DifficultyAttributes::Mania`].
    ///
    /// [`DifficultyAttributes::Mania`]: crate::any::DifficultyAttributes::Mania
    pub fn try_from_attributes(attributes: impl ModeAttributeProvider<Taiko>) -> Option<Self> {
        attributes.attributes().map(Self::from)
    }

    /// Create [`TaikoPerformance`] through a [`ModeAttributeProvider`].
    ///
    /// If you already calculated the attributes for the current map-mod
    /// combination, the [`TaikoBeatmap`] is no longer necessary to calculate
    /// performance attributes so this method can be used instead of
    /// [`TaikoPerformance::new`].
    ///
    /// # Panics
    ///
    /// Panics if the [`ModeAttributeProvider`] did not contain attributes for
    /// taiko e.g. if it's [`DifficultyAttributes::Mania`].
    ///
    /// [`DifficultyAttributes::Mania`]: crate::any::DifficultyAttributes::Mania
    pub fn unchecked_from_attributes(attributes: impl ModeAttributeProvider<Taiko>) -> Self {
        Self::try_from_attributes(attributes).expect("invalid taiko attributes")
    }
}

impl<'map> TryFrom<OsuPerformance<'map>> for TaikoPerformance<'map> {
    type Error = OsuPerformance<'map>;

    /// Try to create [`TaikoPerformance`] through [`OsuPerformance`].
    ///
    /// Returns `None` if [`OsuPerformance`] already replaced its internal
    /// beatmap with [`OsuDifficultyAttributes`], i.e. if
    /// [`OsuPerformance::attributes`] or [`OsuPerformance::generate_state`]
    /// was called.
    ///
    /// [`OsuDifficultyAttributes`]: crate::osu::OsuDifficultyAttributes

    fn try_from(mut osu: OsuPerformance<'map>) -> Result<Self, Self::Error> {
        let MapOrAttrs::Map(converted) = osu.map_or_attrs else {
            return Err(osu);
        };

        let map = match converted.try_convert() {
            Ok(map) => map,
            Err(map) => {
                osu.map_or_attrs = MapOrAttrs::Map(map);

                return Err(osu);
            }
        };

        let OsuPerformance {
            map_or_attrs: _,
            mods,
            acc,
            combo,
            n300,
            n100,
            n50: _,
            n_misses,
            passed_objects,
            clock_rate,
            hitresult_priority,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            mods,
            combo,
            acc,
            passed_objects,
            clock_rate,
            hitresult_priority,
            n300,
            n100,
            n_misses,
        })
    }
}

impl<'map> From<TaikoBeatmap<'map>> for TaikoPerformance<'map> {
    fn from(map: TaikoBeatmap<'map>) -> Self {
        Self {
            map_or_attrs: MapOrAttrs::Map(map),
            mods: 0,
            combo: None,
            acc: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
            n300: None,
            n100: None,
            hitresult_priority: HitResultPriority::default(),
        }
    }
}

impl From<TaikoDifficultyAttributes> for TaikoPerformance<'_> {
    fn from(attrs: TaikoDifficultyAttributes) -> Self {
        Self {
            map_or_attrs: MapOrAttrs::Attrs(attrs),
            mods: 0,
            combo: None,
            acc: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
            n300: None,
            n100: None,
            hitresult_priority: HitResultPriority::default(),
        }
    }
}

impl From<TaikoPerformanceAttributes> for TaikoPerformance<'_> {
    fn from(attrs: TaikoPerformanceAttributes) -> Self {
        attrs.difficulty.into()
    }
}

struct TaikoPerformanceInner {
    attrs: TaikoDifficultyAttributes,
    mods: u32,
    state: TaikoScoreState,
}

impl TaikoPerformanceInner {
    fn calculate(self) -> TaikoPerformanceAttributes {
        // * The effectiveMissCount is calculated by gaining a ratio for totalSuccessfulHits
        // * and increasing the miss penalty for shorter object counts lower than 1000.
        let total_successful_hits = self.total_successful_hits();

        let effective_miss_count = if total_successful_hits > 0 {
            (1000.0 / f64::from(total_successful_hits)).max(1.0) * f64::from(self.state.n_misses)
        } else {
            0.0
        };

        let mut multiplier = 1.13;

        if self.mods.hd() {
            multiplier *= 1.075;
        }

        if self.mods.ez() {
            multiplier *= 0.975;
        }

        let diff_value = self.compute_difficulty_value(effective_miss_count);
        let acc_value = self.compute_accuracy_value();

        let pp = (diff_value.powf(1.1) + acc_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

        TaikoPerformanceAttributes {
            difficulty: self.attrs,
            pp,
            pp_acc: acc_value,
            pp_difficulty: diff_value,
            effective_miss_count,
        }
    }

    fn compute_difficulty_value(&self, effective_miss_count: f64) -> f64 {
        let attrs = &self.attrs;
        let exp_base = 5.0 * (attrs.stars / 0.115).max(1.0) - 4.0;
        let mut diff_value = exp_base.powf(2.25) / 1150.0;

        let len_bonus = 1.0 + 0.1 * (f64::from(attrs.max_combo) / 1500.0).min(1.0);
        diff_value *= len_bonus;

        diff_value *= 0.986_f64.powf(effective_miss_count);

        if self.mods.ez() {
            diff_value *= 0.985;
        }

        if self.mods.hd() {
            diff_value *= 1.025;
        }

        if self.mods.hr() {
            diff_value *= 1.05;
        }

        if self.mods.fl() {
            diff_value *= 1.05 * len_bonus;
        }

        let acc = self.custom_accuracy();

        diff_value * acc.powi(2)
    }

    fn compute_accuracy_value(&self) -> f64 {
        if self.attrs.hit_window <= 0.0 {
            return 0.0;
        }

        let mut acc_value = (60.0 / self.attrs.hit_window).powf(1.1)
            * self.custom_accuracy().powi(8)
            * self.attrs.stars.powf(0.4)
            * 27.0;

        let len_bonus = (self.total_hits() / 1500.0).powf(0.3).min(1.15);
        acc_value *= len_bonus;

        // * Slight HDFL Bonus for accuracy. A clamp is used to prevent against negative values
        if self.mods.hd() && self.mods.fl() {
            acc_value *= (1.075 * len_bonus).max(1.05);
        }

        acc_value
    }

    const fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    const fn total_successful_hits(&self) -> u32 {
        self.state.n300 + self.state.n100
    }

    fn custom_accuracy(&self) -> f64 {
        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = self.state.n300 * 300 + self.state.n100 * 150;
        let denominator = total_hits * 300;

        f64::from(numerator) / f64::from(denominator)
    }
}

fn accuracy(n300: u32, n100: u32, n_misses: u32) -> f64 {
    if n300 + n100 + n_misses == 0 {
        return 0.0;
    }

    let numerator = 2 * n300 + n100;
    let denominator = 2 * (n300 + n100 + n_misses);

    f64::from(numerator) / f64::from(denominator)
}
