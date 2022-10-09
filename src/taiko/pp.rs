use std::borrow::Cow;

use super::{TaikoDifficultyAttributes, TaikoPerformanceAttributes, TaikoScoreState, TaikoStars};
use crate::{
    beatmap::BeatmapHitWindows, Beatmap, DifficultyAttributes, GameMode, Mods, OsuPP,
    PerformanceAttributes,
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
///     .misses(1)
///     .accuracy(98.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = TaikoPP::new(&map)
///     .attributes(pp_result)  // reusing previous results for performance
///     .mods(8 + 64)           // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct TaikoPP<'map> {
    pub(crate) map: Cow<'map, Beatmap>,
    attributes: Option<TaikoDifficultyAttributes>,
    mods: u32,
    combo: Option<usize>,
    acc: f64,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,

    pub(crate) n300: Option<usize>,
    pub(crate) n100: Option<usize>,
    pub(crate) n_misses: usize,
}

impl<'map> TaikoPP<'map> {
    /// Create a new performance calculator for osu!taiko maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map: Cow::Borrowed(map),
            attributes: None,
            mods: 0,
            combo: None,
            acc: 1.0,
            n_misses: 0,
            passed_objects: None,
            clock_rate: None,
            n300: None,
            n100: None,
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
        self.n_misses = n_misses.min(self.map.n_circles as usize);

        self
    }

    /// Set the accuracy between 0.0 and 100.0.
    #[inline]
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = acc / 100.0;
        self.n300.take();
        self.n100.take();

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`TaikoPP`] multiple times with different `passed_objects`, you should use
    /// [`TaikoGradualPerformanceAttributes`](crate::taiko::TaikoGradualPerformanceAttributes).
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
        self.n_misses = n_misses;

        self
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> TaikoPerformanceAttributes {
        let attributes = self.attributes.take().unwrap_or_else(|| {
            let mut calculator = TaikoStars::new(self.map.as_ref())
                .mods(self.mods)
                .is_convert(matches!(self.map, Cow::Owned(_)));

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

    fn assert_hitresults(&'map self, attributes: TaikoDifficultyAttributes) -> TaikoPPInner<'map> {
        let total_result_count = attributes.max_combo();
        let misses = self.n_misses;

        let (n300, n100) = match (self.n300, self.n100) {
            (Some(n300), Some(n100)) => {
                let n300 = n300.min(total_result_count - misses);
                let n100 = n100.min(total_result_count - n300 - misses);

                let given = n300 + n100 + misses;
                let missing = total_result_count - given;

                (n300 + missing, n100)
            }
            (Some(n300), None) => {
                let n300 = n300.min(total_result_count - misses);

                let n100 = total_result_count
                    .saturating_sub(n300)
                    .saturating_sub(misses);

                (n300, n100)
            }
            (None, Some(n100)) => {
                let n100 = n100.min(total_result_count - misses);

                let n300 = total_result_count
                    .saturating_sub(n100)
                    .saturating_sub(misses);

                (n300, n100)
            }
            (None, None) => {
                let target_total = (self.acc * (total_result_count * 2) as f64) as usize;

                let n300 = target_total - total_result_count.saturating_sub(misses);
                let n100 = total_result_count
                    .saturating_sub(n300)
                    .saturating_sub(misses);

                (n300, n100)
            }
        };

        let acc = (2 * n300 + n100) as f64 / (2 * (n300 + n100 + misses)) as f64;

        TaikoPPInner {
            map: self.map.as_ref(),
            attributes,
            acc,
            n300,
            n100,
            mods: self.mods,
            n_misses: misses,
            clock_rate: self.clock_rate.unwrap_or_else(|| self.mods.clock_rate()),
        }
    }
}

struct TaikoPPInner<'map> {
    map: &'map Beatmap,
    attributes: TaikoDifficultyAttributes,
    mods: u32,
    acc: f64,
    n_misses: usize,
    clock_rate: f64,
    n300: usize,
    n100: usize,
}

impl<'map> TaikoPPInner<'map> {
    fn calculate(self) -> TaikoPerformanceAttributes {
        // * The effectiveMissCount is calculated by gaining a ratio for totalSuccessfulHits
        // * and increasing the miss penalty for shorter object counts lower than 1000.
        let total_successful_hits = self.total_successful_hits();

        let effective_miss_count = if total_successful_hits > 0 {
            (1000.0 / (total_successful_hits as f64)).max(1.0) * self.n_misses as f64
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
            difficulty: self.attributes,
            pp,
            pp_acc: acc_value,
            pp_difficulty: diff_value,
            effective_miss_count,
        }
    }

    fn compute_difficulty_value(&self, effective_miss_count: f64) -> f64 {
        let attrs = &self.attributes;
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

        diff_value * self.acc * self.acc
    }

    #[inline]
    fn compute_accuracy_value(&self) -> f64 {
        let BeatmapHitWindows { od: hit_window, .. } = self
            .map
            .attributes()
            .mods(self.mods)
            .clock_rate(self.clock_rate)
            .hit_windows();

        if hit_window <= 0.0 {
            return 0.0;
        }

        let mut acc_value = (60.0 / hit_window).powf(1.1)
            * self.acc.powi(8)
            * self.attributes.stars.powf(0.4)
            * 27.0;

        let len_bonus = (self.total_hits() as f64 / 1500.0).powf(0.3).min(1.15);
        acc_value *= len_bonus;

        // * Slight HDFL Bonus for accuracy. A clamp is used to prevent against negative values
        if self.mods.hd() && self.mods.fl() {
            acc_value *= (1.075 * len_bonus).max(1.05);
        }

        acc_value
    }

    fn total_hits(&self) -> usize {
        self.n300 + self.n100 + self.n_misses
    }

    fn total_successful_hits(&self) -> usize {
        self.n300 + self.n100
    }
}

impl<'map> From<OsuPP<'map>> for TaikoPP<'map> {
    #[inline]
    fn from(osu: OsuPP<'map>) -> Self {
        let OsuPP {
            map,
            mods,
            acc,
            combo,
            n300,
            n100,
            n_misses,
            passed_objects,
            clock_rate,
            ..
        } = osu;

        Self {
            map: map.convert_mode(GameMode::Taiko),
            attributes: None,
            mods,
            combo,
            acc: acc.unwrap_or(1.0),
            passed_objects,
            clock_rate,
            n300,
            n100,
            n_misses,
        }
    }
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
