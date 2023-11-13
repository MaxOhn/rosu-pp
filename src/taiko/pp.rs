use std::borrow::Cow;

use super::{TaikoDifficultyAttributes, TaikoPerformanceAttributes, TaikoScoreState, TaikoStars};
use crate::{
    Beatmap, DifficultyAttributes, GameMode, HitResultPriority, Mods, OsuPP, PerformanceAttributes,
};

/// Performance calculator on osu!taiko maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{TaikoPP, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let pp_result = TaikoPP::new(&map)
///     .mods(8 + 64) // HDDT
///     .combo(1234)
///     .accuracy(98.5)
///     .n_misses(1)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = TaikoPP::new(&map)
///     .attributes(pp_result) // reusing previous results for performance
///     .mods(8 + 64) // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct TaikoPP<'map> {
    pub(crate) map: Cow<'map, Beatmap>,
    is_convert: bool,
    attributes: Option<TaikoDifficultyAttributes>,
    mods: u32,
    combo: Option<usize>,
    acc: Option<f64>,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,
    hitresult_priority: Option<HitResultPriority>,

    pub(crate) n300: Option<usize>,
    pub(crate) n100: Option<usize>,
    pub(crate) n_misses: Option<usize>,
}

impl<'map> TaikoPP<'map> {
    /// Create a new performance calculator for osu!taiko maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        let map = map.convert_mode(GameMode::Taiko);

        Self {
            is_convert: matches!(map, Cow::Owned(_)),
            map,
            attributes: None,
            mods: 0,
            combo: None,
            acc: None,
            n_misses: None,
            passed_objects: None,
            clock_rate: None,
            n300: None,
            n100: None,
            hitresult_priority: None,
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    #[inline]
    pub fn attributes(mut self, attrs: impl TaikoAttributeProvider) -> Self {
        if let Some(attrs) = attrs.attributes() {
            self.attributes = Some(attrs);
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

    /// Specify the amount of misses of the play.
    #[inline]
    pub fn n_misses(mut self, n_misses: usize) -> Self {
        self.n_misses = Some(n_misses.min(self.map.n_circles as usize));

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    #[inline]
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`TaikoPP`] multiple times with different `passed_objects`, you should use
    /// [`TaikoGradualPerformanceAttributes`](crate::taiko::TaikoGradualPerformance).
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

    /// Specify whether the map is a convert i.e. an osu!standard map.
    ///
    /// This only needs to be specified if the map was converted manually beforehand.
    #[inline]
    pub fn is_convert(mut self, is_convert: bool) -> Self {
        self.is_convert = is_convert;

        self
    }

    /// Provide parameters through a [`TaikoScoreState`].
    #[inline]
    pub fn state(mut self, state: TaikoScoreState) -> Self {
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
        let max_combo = match self.attributes {
            Some(ref attrs) => attrs.max_combo,
            None => self.attributes.insert(self.generate_attributes()).max_combo,
        };

        let total_result_count = if let Some(passed_objects) = self.passed_objects {
            max_combo.min(passed_objects)
        } else {
            max_combo
        };

        let priority = self.hitresult_priority.unwrap_or_default();

        let n_misses = self.n_misses.map_or(0, |n| n.min(total_result_count));
        let n_remaining = total_result_count - n_misses;

        let mut n300 = self.n300.map_or(0, |n| n.min(n_remaining));
        let mut n100 = self.n100.map_or(0, |n| n.min(n_remaining));

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
                    let target_total = acc * (2 * total_result_count) as f64;

                    let mut best_dist = f64::MAX;

                    let raw_n300 = target_total - n_remaining as f64;
                    let min_n300 = n_remaining.min(raw_n300.floor() as usize);
                    let max_n300 = n_remaining.min(raw_n300.ceil() as usize);

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

        let attrs = self
            .attributes
            .take()
            .unwrap_or_else(|| self.generate_attributes());

        let inner = TaikoPpInner {
            mods: self.mods,
            state,
            attrs,
        };

        inner.calculate()
    }

    fn generate_attributes(&self) -> TaikoDifficultyAttributes {
        let mut calculator = TaikoStars::new(self.map.as_ref())
            .mods(self.mods)
            .is_convert(self.is_convert);

        if let Some(passed_objects) = self.passed_objects {
            calculator = calculator.passed_objects(passed_objects);
        }

        if let Some(clock_rate) = self.clock_rate {
            calculator = calculator.clock_rate(clock_rate);
        }

        calculator.calculate()
    }
}

struct TaikoPpInner {
    attrs: TaikoDifficultyAttributes,
    mods: u32,
    state: TaikoScoreState,
}

impl TaikoPpInner {
    fn calculate(self) -> TaikoPerformanceAttributes {
        // * The effectiveMissCount is calculated by gaining a ratio for totalSuccessfulHits
        // * and increasing the miss penalty for shorter object counts lower than 1000.
        let total_successful_hits = self.total_successful_hits();

        let effective_miss_count = if total_successful_hits > 0 {
            (1000.0 / (total_successful_hits as f64)).max(1.0) * self.state.n_misses as f64
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

        let len_bonus = 1.0 + 0.1 * (attrs.max_combo as f64 / 1500.0).min(1.0);
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

        diff_value * acc * acc
    }

    #[inline]
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

    fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    fn total_successful_hits(&self) -> usize {
        self.state.n300 + self.state.n100
    }

    fn custom_accuracy(&self) -> f64 {
        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = self.state.n300 * 300 + self.state.n100 * 150;
        let denominator = total_hits * 300;

        numerator as f64 / denominator as f64
    }
}

impl<'map> From<OsuPP<'map>> for TaikoPP<'map> {
    #[inline]
    fn from(osu: OsuPP<'map>) -> Self {
        let OsuPP {
            map,
            attributes: _,
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

        let map = map.convert_mode(GameMode::Taiko);

        Self {
            is_convert: matches!(map, Cow::Owned(_)),
            map,
            attributes: None,
            mods,
            combo,
            acc,
            passed_objects,
            clock_rate,
            hitresult_priority,
            n300,
            n100,
            n_misses,
        }
    }
}

fn accuracy(n300: usize, n100: usize, n_misses: usize) -> f64 {
    if n300 + n100 + n_misses == 0 {
        return 0.0;
    }

    let numerator = 2 * n300 + n100;
    let denominator = 2 * (n300 + n100 + n_misses);

    numerator as f64 / denominator as f64
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait TaikoAttributeProvider {
    /// Provide the actual difficulty attributes.
    fn attributes(self) -> Option<TaikoDifficultyAttributes>;
}

impl TaikoAttributeProvider for TaikoDifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<TaikoDifficultyAttributes> {
        Some(self)
    }
}

impl TaikoAttributeProvider for TaikoPerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<TaikoDifficultyAttributes> {
        Some(self.difficulty)
    }
}

impl TaikoAttributeProvider for DifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<TaikoDifficultyAttributes> {
        #[allow(irrefutable_let_patterns)]
        if let Self::Taiko(attributes) = self {
            Some(attributes)
        } else {
            None
        }
    }
}

impl TaikoAttributeProvider for PerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<TaikoDifficultyAttributes> {
        #[allow(irrefutable_let_patterns)]
        if let Self::Taiko(attributes) = self {
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

    static DATA: OnceLock<(Beatmap, TaikoDifficultyAttributes)> = OnceLock::new();

    const MAX_COMBO: usize = 289;

    fn test_data() -> (&'static Beatmap, TaikoDifficultyAttributes) {
        let (map, attrs) = DATA.get_or_init(|| {
            let path = "./maps/1028484.osu";
            let map = Beatmap::from_path(path).unwrap();
            let attrs = TaikoStars::new(&map).calculate();

            assert_eq!(MAX_COMBO, attrs.max_combo);

            (map, attrs)
        });

        (map, attrs.to_owned())
    }

    /// Checks all remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`TaikoScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate.
    fn brute_force_best(
        acc: f64,
        n300: Option<usize>,
        n100: Option<usize>,
        n_misses: usize,
        best_case: bool,
    ) -> TaikoScoreState {
        let n_misses = n_misses.min(MAX_COMBO);

        let mut best_state = TaikoScoreState {
            n_misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;

        let n_objects = MAX_COMBO;
        let n_remaining = n_objects - n_misses;

        let (min_n300, max_n300) = match (n300, n100) {
            (Some(n300), _) => (n_remaining.min(n300), n_remaining.min(n300)),
            (None, Some(n100)) => (
                n_remaining.saturating_sub(n100),
                n_remaining.saturating_sub(n100),
            ),
            (None, None) => (0, n_remaining),
        };

        for new300 in min_n300..=max_n300 {
            let new100 = match n100 {
                Some(n100) => n_remaining.min(n100),
                None => n_remaining - new300,
            };

            let curr_acc = accuracy(new300, new100, n_misses);
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
        #![proptest_config(ProptestConfig::with_cases(20_000))]
        #[test]
        fn taiko_hitresults(
            acc in 0.0..=1.0,
            n300 in option::weighted(0.10, 0_usize..=MAX_COMBO + 10),
            n100 in option::weighted(0.10, 0_usize..=MAX_COMBO + 10),
            n_misses in option::weighted(0.15, 0_usize..=MAX_COMBO + 10),
            best_case in prop::bool::ANY,
        ) {
            let (map, attrs) = test_data();

            let priority = if best_case {
                HitResultPriority::BestCase
            } else {
                HitResultPriority::WorstCase
            };

            let mut state = TaikoPP::new(map)
                .attributes(attrs)
                .accuracy(acc * 100.0)
                .hitresult_priority(priority);

            if let Some(n300) = n300 {
                state = state.n300(n300);
            }

            if let Some(n100) = n100 {
                state = state.n100(n100);
            }

            if let Some(n_misses) = n_misses {
                state = state.n_misses(n_misses);
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
    fn hitresults_n300_n_misses_best() {
        let (map, attrs) = test_data();

        let state = TaikoPP::new(map)
            .attributes(attrs)
            .combo(100)
            .n300(150)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state();

        let expected = TaikoScoreState {
            max_combo: 100,
            n300: 150,
            n100: 137,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn hitresults_n_misses_best() {
        let (map, attrs) = test_data();

        let state = TaikoPP::new(map)
            .attributes(attrs)
            .combo(100)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state();

        let expected = TaikoScoreState {
            max_combo: 100,
            n300: 287,
            n100: 0,
            n_misses: 2,
        };

        assert_eq!(state, expected);
    }
}
