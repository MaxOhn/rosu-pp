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

pub mod gradual;

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
    pub(crate) misses: Option<u32>,
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
    pub const fn misses(mut self, n_misses: u32) -> Self {
        self.misses = Some(n_misses);

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
    /// If you want to calculate the performance after every few objects,
    /// instead of using [`TaikoPerformance`] multiple times with different
    /// `passed_objects`, you should use [`TaikoGradualPerformance`].
    ///
    /// [`TaikoGradualPerformance`]: crate::taiko::TaikoGradualPerformance
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
            misses,
        } = state;

        self.combo = Some(max_combo);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.misses = Some(misses);

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

        let misses = self.misses.map_or(0, |n| cmp::min(n, total_result_count));
        let n_remaining = total_result_count - misses;

        let mut n300 = self.n300.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n100 = self.n100.map_or(0, |n| cmp::min(n, n_remaining));

        if let Some(acc) = self.acc {
            match (self.n300, self.n100) {
                (Some(_), Some(_)) => {
                    let remaining = total_result_count.saturating_sub(n300 + n100 + misses);

                    match priority {
                        HitResultPriority::BestCase => n300 += remaining,
                        HitResultPriority::WorstCase => n100 += remaining,
                    }
                }
                (Some(_), None) => n100 += total_result_count.saturating_sub(n300 + misses),
                (None, Some(_)) => n300 += total_result_count.saturating_sub(n100 + misses),
                (None, None) => {
                    let target_total = acc * f64::from(2 * total_result_count);

                    let mut best_dist = f64::MAX;

                    let raw_n300 = target_total - f64::from(n_remaining);
                    let min_n300 = cmp::min(n_remaining, raw_n300.floor() as u32);
                    let max_n300 = cmp::min(n_remaining, raw_n300.ceil() as u32);

                    for new300 in min_n300..=max_n300 {
                        let new100 = n_remaining - new300;
                        let dist = (acc - accuracy(new300, new100, misses)).abs();

                        if dist < best_dist {
                            best_dist = dist;
                            n300 = new300;
                            n100 = new100;
                        }
                    }
                }
            }
        } else {
            let remaining = total_result_count.saturating_sub(n300 + n100 + misses);

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

        let max_possible_combo = max_combo.saturating_sub(misses);

        let max_combo = self.combo.map_or(max_possible_combo, |combo| {
            cmp::min(combo, max_possible_combo)
        });

        TaikoScoreState {
            max_combo,
            n300,
            n100,
            misses,
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
            misses,
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
            misses,
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
            misses: None,
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
            misses: None,
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
            (1000.0 / f64::from(total_successful_hits)).max(1.0) * f64::from(self.state.misses)
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

        diff_value * acc.powf(2.0)
    }

    fn compute_accuracy_value(&self) -> f64 {
        if self.attrs.hit_window <= 0.0 {
            return 0.0;
        }

        let mut acc_value = (60.0 / self.attrs.hit_window).powf(1.1)
            * self.custom_accuracy().powf(8.0)
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

fn accuracy(n300: u32, n100: u32, misses: u32) -> f64 {
    if n300 + n100 + misses == 0 {
        return 0.0;
    }

    let numerator = 2 * n300 + n100;
    let denominator = 2 * (n300 + n100 + misses);

    f64::from(numerator) / f64::from(denominator)
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use proptest::prelude::*;

    use crate::Beatmap;

    use super::*;

    static ATTRS: OnceLock<TaikoDifficultyAttributes> = OnceLock::new();

    const MAX_COMBO: u32 = 289;

    fn attrs() -> TaikoDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let converted = Beatmap::from_path("./resources/1028484.osu")
                    .unwrap()
                    .unchecked_into_converted::<Taiko>();

                let attrs = ModeDifficulty::new().calculate(&converted);

                assert_eq!(MAX_COMBO, attrs.max_combo);

                attrs
            })
            .to_owned()
    }

    /// Checks all remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`TaikoScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate.
    fn brute_force_best(
        acc: f64,
        n300: Option<u32>,
        n100: Option<u32>,
        misses: u32,
        best_case: bool,
    ) -> TaikoScoreState {
        let misses = cmp::min(misses, MAX_COMBO);

        let mut best_state = TaikoScoreState {
            misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let n_objects = MAX_COMBO;
        let n_remaining = n_objects - misses;

        let (min_n300, max_n300) = match (n300, n100) {
            (Some(n300), _) => (cmp::min(n_remaining, n300), cmp::min(n_remaining, n300)),
            (None, Some(n100)) => (
                n_remaining.saturating_sub(n100),
                n_remaining.saturating_sub(n100),
            ),
            (None, None) => (0, n_remaining),
        };

        for new300 in min_n300..=max_n300 {
            let new100 = match n100 {
                Some(n100) => cmp::min(n_remaining, n100),
                None => n_remaining - new300,
            };

            let curr_acc = accuracy(new300, new100, misses);
            let curr_dist = (acc - curr_acc).abs();

            if curr_dist < best_dist {
                best_dist = curr_dist;
                best_state.n300 = new300;
                best_state.n100 = new100;
            }
        }

        if best_state.n300 + best_state.n100 < n_remaining {
            let remaining = n_remaining - (best_state.n300 + best_state.n100);

            if best_case {
                best_state.n300 += remaining;
            } else {
                best_state.n100 += remaining;
            }
        }

        best_state
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn hitresults(
            acc in 0.0..=1.0,
            n300 in prop::option::weighted(0.10, 0_u32..=MAX_COMBO + 10),
            n100 in prop::option::weighted(0.10, 0_u32..=MAX_COMBO + 10),
            n_misses in prop::option::weighted(0.15, 0_u32..=MAX_COMBO + 10),
            best_case in prop::bool::ANY,
        ) {
            let priority = if best_case {
                HitResultPriority::BestCase
            } else {
                HitResultPriority::WorstCase
            };

            let mut state = TaikoPerformance::from(attrs())
                .accuracy(acc * 100.0)
                .hitresult_priority(priority);

            if let Some(n300) = n300 {
                state = state.n300(n300);
            }

            if let Some(n100) = n100 {
                state = state.n100(n100);
            }

            if let Some(misses) = n_misses {
                state = state.misses(misses);
            }

            let state = state.generate_state();

            let mut expected = brute_force_best(
                acc,
                n300,
                n100,
                n_misses.unwrap_or(0),
                best_case,
            );
            expected.max_combo = MAX_COMBO.saturating_sub(n_misses.unwrap_or(0));

            assert_eq!(state, expected);
        }
    }

    #[test]
    fn hitresults_n300_misses_best() {
        let state = TaikoPerformance::from(attrs())
            .combo(100)
            .n300(150)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state();

        let expected = TaikoScoreState {
            max_combo: 100,
            n300: 150,
            n100: 137,
            misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn hitresults_misses_best() {
        let state = TaikoPerformance::from(attrs())
            .combo(100)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state();

        let expected = TaikoScoreState {
            max_combo: 100,
            n300: 287,
            n100: 0,
            misses: 2,
        };

        assert_eq!(state, expected);
    }
}
