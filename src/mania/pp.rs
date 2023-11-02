use std::borrow::Cow;

use super::{ManiaDifficultyAttributes, ManiaPerformanceAttributes, ManiaScoreState, ManiaStars};
use crate::{
    Beatmap, DifficultyAttributes, GameMode, HitResultPriority, Mods, OsuPP, PerformanceAttributes,
};

/// Performance calculator on osu!mania maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{ManiaPP, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let pp_result = ManiaPP::new(&map)
///     .mods(64) // DT
///     .n_misses(1)
///     .accuracy(98.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = ManiaPP::new(&map)
///     .attributes(pp_result) // reusing previous results for performance
///     .mods(8 + 64) // has to be the same to reuse attributes
///     .n320(2000)
///     .n300(500)
///     .n200(200)
///     .n100(100)
///     .n50(10)
///     .n_misses(1)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct ManiaPP<'map> {
    map: Cow<'map, Beatmap>,
    is_convert: bool,
    attributes: Option<ManiaDifficultyAttributes>,
    mods: u32,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,

    pub(crate) n320: Option<usize>,
    pub(crate) n300: Option<usize>,
    pub(crate) n200: Option<usize>,
    pub(crate) n100: Option<usize>,
    pub(crate) n50: Option<usize>,
    pub(crate) n_misses: Option<usize>,

    acc: Option<f64>,
    hitresult_priority: Option<HitResultPriority>,
}

impl<'map> ManiaPP<'map> {
    /// Create a new performance calculator for osu!mania maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        let map = map.convert_mode(GameMode::Mania);

        Self {
            is_convert: matches!(map, Cow::Owned(_)),
            map,
            attributes: None,
            mods: 0,
            passed_objects: None,
            clock_rate: None,
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            n_misses: None,
            acc: None,
            hitresult_priority: None,
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    #[inline]
    pub fn attributes(mut self, attrs: impl ManiaAttributeProvider) -> Self {
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

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`ManiaPP`] multiple times with different `passed_objects`, you should use
    /// [`ManiaGradualPerformanceAttributes`](crate::mania::ManiaGradualPerformance).
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

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    #[inline]
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc / 100.0);

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

    /// Specify the amount of 320s of a play.
    #[inline]
    pub fn n320(mut self, n320: usize) -> Self {
        self.n320 = Some(n320);

        self
    }

    /// Specify the amount of 300s of a play.
    #[inline]
    pub fn n300(mut self, n300: usize) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 200s of a play.
    #[inline]
    pub fn n200(mut self, n200: usize) -> Self {
        self.n200 = Some(n200);

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

    /// Specify whether the map is a convert i.e. an osu!standard map.
    ///
    /// This only needs to be specified if the map was converted manually beforehand.
    #[inline]
    pub fn is_convert(mut self, is_convert: bool) -> Self {
        self.is_convert = is_convert;

        self
    }

    /// Provide parameters through an [`ManiaScoreState`].
    #[inline]
    pub fn state(mut self, state: ManiaScoreState) -> Self {
        let ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            n_misses,
        } = state;

        self.n320 = Some(n320);
        self.n300 = Some(n300);
        self.n200 = Some(n200);
        self.n100 = Some(n100);
        self.n50 = Some(n50);
        self.n_misses = Some(n_misses);

        self
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(self) -> ManiaPerformanceAttributes {
        let attrs = self.attributes.unwrap_or_else(|| {
            let mut calculator = ManiaStars::new(self.map.as_ref())
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

        let inner = ManiaPpInner {
            attrs,
            mods: self.mods,
            state: self.generate_hitresults(),
        };

        inner.calculate()
    }

    fn generate_hitresults(&self) -> ManiaScoreState {
        let n_objects = self.passed_objects.unwrap_or(self.map.hit_objects.len());
        let priority = self.hitresult_priority.unwrap_or_default();

        let n_misses = self.n_misses.map_or(0, |n| n.min(n_objects));
        let n_remaining = n_objects - n_misses;

        let mut n320 = self.n320.map_or(0, |n| n.min(n_remaining));
        let mut n300 = self.n300.map_or(0, |n| n.min(n_remaining));
        let mut n200 = self.n200.map_or(0, |n| n.min(n_remaining));
        let mut n100 = self.n100.map_or(0, |n| n.min(n_remaining));
        let mut n50 = self.n50.map_or(0, |n| n.min(n_remaining));

        if let Some(acc) = self.acc {
            let target_total = acc * (6 * n_objects) as f64;

            match (self.n320, self.n300, self.n200, self.n100, self.n50) {
                // All hitresults given
                (Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                    let remaining =
                        n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n320 += remaining,
                        HitResultPriority::WorstCase => n50 += remaining,
                    }
                }

                // All but one hitresults given
                (None, Some(_), Some(_), Some(_), Some(_)) => {
                    n320 = n_objects.saturating_sub(n300 + n200 + n100 + n50 + n_misses)
                }
                (Some(_), None, Some(_), Some(_), Some(_)) => {
                    n300 = n_objects.saturating_sub(n320 + n200 + n100 + n50 + n_misses)
                }
                (Some(_), Some(_), None, Some(_), Some(_)) => {
                    n200 = n_objects.saturating_sub(n320 + n300 + n100 + n50 + n_misses)
                }
                (Some(_), Some(_), Some(_), None, Some(_)) => {
                    n100 = n_objects.saturating_sub(n320 + n300 + n200 + n50 + n_misses)
                }
                (Some(_), Some(_), Some(_), Some(_), None) => {
                    n50 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n_misses);
                }

                // n200, n100, and n50 given
                (None, None, Some(_), Some(_), Some(_)) => {
                    let n_remaining =
                        n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n320 = n_remaining,
                        HitResultPriority::WorstCase => n300 = n_remaining,
                    }
                }

                // n100 and n50 given
                (.., None, Some(_), Some(_)) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n100 + n50 + n_misses);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (n320 + n300, n320 + n300),
                        (Some(_), None) => (n320, n320),
                        (None, Some(_)) => (n300, n300),
                        (None, None) => {
                            let raw_n3x0 = (target_total - (4 * n_remaining) as f64
                                + (2 * n100 + 3 * n50) as f64)
                                / 2.0;
                            let min_n3x0 = (raw_n3x0.floor() as usize)
                                .min(n_remaining.saturating_sub(n100 + n50));
                            let max_n3x0 = (raw_n3x0.ceil() as usize)
                                .min(n_remaining.saturating_sub(n100 + n50));

                            (min_n3x0, max_n3x0)
                        }
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let new200 = n_remaining.saturating_sub(new3x0 + n100 + n50);
                        let curr_dist =
                            (acc - accuracy(new3x0, 0, new200, n100, n50, n_misses)).abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n3x0 = new3x0;
                            n200 = new200;
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }
                }

                // n200 and n50 given
                (.., Some(_), None, Some(_)) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n50 + n_misses);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (n320 + n300, n320 + n300),
                        (Some(_), None) => (n320, n320),
                        (None, Some(_)) => (n300, n300),
                        (None, None) => {
                            let raw_n3x0 =
                                (target_total - (2 * (n_remaining + n200) - n50) as f64) / 4.0;
                            let min_n3x0 = (raw_n3x0.floor() as usize)
                                .min(n_remaining.saturating_sub(n200 + n50));
                            let max_n3x0 = (raw_n3x0.ceil() as usize)
                                .min(n_remaining.saturating_sub(n200 + n50));

                            (min_n3x0, max_n3x0)
                        }
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let new100 = n_remaining.saturating_sub(new3x0 + n200 + n50);
                        let curr_dist =
                            (acc - accuracy(new3x0, 0, n200, new100, n50, n_misses)).abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n3x0 = new3x0;
                            n100 = new100;
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }
                }

                // n200 and n100 given
                (.., Some(_), Some(_), None) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n_misses);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (n320 + n300, n320 + n300),
                        (Some(_), None) => (n320, n320),
                        (None, Some(_)) => (n300, n300),
                        (None, None) => {
                            let raw_n3x0 =
                                (target_total - (n_remaining + 3 * n200 + n100) as f64) / 5.0;
                            let min_n3x0 = (raw_n3x0.floor() as usize)
                                .min(n_remaining.saturating_sub(n200 + n100));
                            let max_n3x0 = (raw_n3x0.ceil() as usize)
                                .min(n_remaining.saturating_sub(n200 + n100));

                            (min_n3x0, max_n3x0)
                        }
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let new50 = n_remaining.saturating_sub(new3x0 + n200 + n100);
                        let curr_dist =
                            (acc - accuracy(new3x0, 0, n200, n100, new50, n_misses)).abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n3x0 = new3x0;
                            n50 = new50;
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }
                }

                // n200 given
                (.., Some(_), None, None) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n_misses);

                    let min_n3x0 = (((target_total - (2 * (n_remaining + n200)) as f64) / 4.0)
                        .floor() as usize)
                        .min(n_remaining - n200);

                    let max_n3x0 = (((target_total - (n_remaining + 3 * n200) as f64) / 5.0).ceil()
                        as usize)
                        .min(n_remaining - n200);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => {
                            (n_remaining.min(n320 + n300), n_remaining.min(n320 + n300))
                        }
                        (Some(_), None) => (min_n3x0.max(n320), max_n3x0.max(n320)),
                        (None, Some(_)) => (min_n3x0.max(n300), max_n3x0.max(n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let raw_n100 = target_total - (n_remaining + 5 * new3x0 + 3 * n200) as f64;
                        let min_n100 = (raw_n100.floor() as usize)
                            .min(n_remaining.saturating_sub(new3x0 + n200));
                        let max_n100 = (raw_n100.ceil() as usize)
                            .min(n_remaining.saturating_sub(new3x0 + n200));

                        for new100 in min_n100..=max_n100 {
                            let new50 = n_remaining.saturating_sub(new3x0 + n200 + new100);
                            let curr_dist =
                                (acc - accuracy(new3x0, 0, n200, new100, new50, n_misses)).abs();

                            if curr_dist < best_dist {
                                best_dist = curr_dist;
                                n3x0 = new3x0;
                                n100 = new100;
                                n50 = new50;
                            }
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }
                }

                // n100 given
                (.., None, Some(_), None) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n100 + n_misses);

                    let min_n3x0 = ((acc * (3 * n_remaining) as f64
                        - (2 * n_remaining - n100) as f64)
                        .floor() as usize)
                        .min(n_remaining - n100);

                    let max_n3x0 = (((target_total - (n_remaining + n100) as f64) / 5.0).ceil()
                        as usize)
                        .min(n_remaining - n100);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => {
                            (n_remaining.min(n320 + n300), n_remaining.min(n320 + n300))
                        }
                        (Some(_), None) => (min_n3x0.max(n320), max_n3x0.max(n320)),
                        (None, Some(_)) => (min_n3x0.max(n300), max_n3x0.max(n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let raw_n200 =
                            (target_total - (n_remaining + 5 * new3x0 + n100) as f64) / 3.0;
                        let min_n200 = (raw_n200.floor() as usize)
                            .min(n_remaining.saturating_sub(new3x0 + n100));
                        let max_n200 = (raw_n200.ceil() as usize)
                            .min(n_remaining.saturating_sub(new3x0 + n100));

                        for new200 in min_n200..=max_n200 {
                            let new50 = n_remaining.saturating_sub(new3x0 + new200 + n100);
                            let curr_dist =
                                (acc - accuracy(new3x0, 0, new200, n100, new50, n_misses)).abs();

                            if curr_dist < best_dist {
                                best_dist = curr_dist;
                                n3x0 = new3x0;
                                n200 = new200;
                                n50 = new50;
                            }
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }
                }

                // n50 given
                (.., None, None, Some(_)) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n50 + n_misses);

                    let min_n3x0 = (((target_total - (4 * n_remaining - 3 * n50) as f64) / 2.0)
                        .floor() as usize)
                        .min(n_remaining - n50);

                    let max_n3x0 = (((target_total - (2 * n_remaining - n50) as f64) / 4.0).ceil()
                        as usize)
                        .min(n_remaining - n50);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => {
                            (n_remaining.min(n320 + n300), n_remaining.min(n320 + n300))
                        }
                        (Some(_), None) => (min_n3x0.max(n320), max_n3x0.max(n320)),
                        (None, Some(_)) => (min_n3x0.max(n300), max_n3x0.max(n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let raw_n200 = (target_total - (2 * n_remaining + 4 * new3x0) as f64
                            + n50 as f64)
                            / 2.0;
                        let min_n200 = (raw_n200.floor() as usize)
                            .min(n_remaining.saturating_sub(new3x0 + n50));
                        let max_n200 = (raw_n200.ceil() as usize)
                            .min(n_remaining.saturating_sub(new3x0 + n50));

                        for new200 in min_n200..=max_n200 {
                            let new100 = n_remaining.saturating_sub(new3x0 + new200 + n50);
                            let curr_dist =
                                (acc - accuracy(new3x0, 0, new200, new100, n50, n_misses)).abs();

                            if curr_dist < best_dist {
                                best_dist = curr_dist;
                                n3x0 = new3x0;
                                n200 = new200;
                                n100 = new100;
                            }
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }

                    if self.n320.is_none() {
                        if let HitResultPriority::BestCase = priority {
                            // Distribute n200 onto n320 and n100
                            let n = n200 / 2;
                            n320 += n;
                            n200 -= 2 * n;
                            n100 += n;
                        }
                    }
                }

                // Neither n200, n100, nor n50 given
                (.., None, None, None) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n_misses);

                    let min_n3x0 = (((target_total - (4 * n_remaining) as f64) / 5.0).floor()
                        as usize)
                        .min(n_remaining);

                    let max_n3x0 = (((target_total - n_remaining as f64) / 5.0)
                        .min(acc * (3 * n_objects) as f64 - n_remaining as f64)
                        .ceil() as usize)
                        .min(n_remaining);

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => {
                            (n_remaining.min(n320 + n300), n_remaining.min(n320 + n300))
                        }
                        (Some(_), None) => (min_n3x0.max(n320), max_n3x0.max(n320)),
                        (None, Some(_)) => (min_n3x0.max(n300), max_n3x0.max(n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let min_n200 = ((acc * (3 * n_objects) as f64
                            - (n_remaining + 2 * new3x0) as f64)
                            .floor() as usize)
                            .min(n_remaining - new3x0);

                        let max_n200 = (((target_total - (n_remaining + 5 * new3x0) as f64) / 3.0)
                            .ceil() as usize)
                            .min(n_remaining - new3x0);

                        for new200 in min_n200..=max_n200 {
                            let raw_n100 =
                                target_total - (n_remaining + 5 * new3x0 + 3 * new200) as f64;
                            let min_n100 =
                                (raw_n100.floor() as usize).min(n_remaining - (new3x0 + new200));
                            let max_n100 =
                                (raw_n100.ceil() as usize).min(n_remaining - (new3x0 + new200));

                            for new100 in min_n100..=max_n100 {
                                let new50 = n_remaining - new3x0 - new200 - new100;
                                let curr_acc = accuracy(new3x0, 0, new200, new100, new50, n_misses);
                                let curr_dist = (acc - curr_acc).abs();

                                if curr_dist < best_dist {
                                    best_dist = curr_dist;
                                    n3x0 = new3x0;
                                    n200 = new200;
                                    n100 = new100;
                                    n50 = new50;
                                }
                            }
                        }
                    }

                    match (self.n320, self.n300) {
                        (None, None) => match priority {
                            HitResultPriority::BestCase => n320 = n3x0,
                            HitResultPriority::WorstCase => n300 = n3x0,
                        },
                        (Some(_), None) => n300 = n3x0 - n320,
                        (None, Some(_)) => n320 = n3x0 - n300,
                        _ => {}
                    }

                    if self.n320.is_none() {
                        if let HitResultPriority::BestCase = priority {
                            // Distribute n200 onto n320 and n100
                            let n = n200 / 2;
                            n320 += n;
                            n200 -= 2 * n;
                            n100 += n;
                        }
                    }
                }
            }
        } else {
            let remaining = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + n_misses);

            match priority {
                HitResultPriority::BestCase => {
                    match (self.n320, self.n300, self.n200, self.n100, self.n50) {
                        (None, ..) => n320 = remaining,
                        (_, None, ..) => n300 = remaining,
                        (_, _, None, ..) => n200 = remaining,
                        (.., None, _) => n100 = remaining,
                        (.., None) => n50 = remaining,
                        _ => n320 += remaining,
                    }
                }
                HitResultPriority::WorstCase => {
                    match (self.n50, self.n100, self.n200, self.n300, self.n320) {
                        (None, ..) => n50 = remaining,
                        (_, None, ..) => n100 = remaining,
                        (_, _, None, ..) => n200 = remaining,
                        (.., None, _) => n300 = remaining,
                        (.., None) => n320 = remaining,
                        _ => n50 += remaining,
                    }
                }
            }
        }

        ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            n_misses,
        }
    }
}

struct ManiaPpInner {
    attrs: ManiaDifficultyAttributes,
    mods: u32,
    state: ManiaScoreState,
}

impl ManiaPpInner {
    fn calculate(self) -> ManiaPerformanceAttributes {
        // * Arbitrary initial value for scaling pp in order to standardize distributions across game modes.
        // * The specific number has no intrinsic meaning and can be adjusted as needed.
        let mut multiplier = 8.0;

        if self.mods.nf() {
            multiplier *= 0.75;
        }

        if self.mods.ez() {
            multiplier *= 0.5;
        }

        let difficulty_value = self.compute_difficulty_value();
        let pp = difficulty_value * multiplier;

        ManiaPerformanceAttributes {
            difficulty: self.attrs,
            pp,
            pp_difficulty: difficulty_value,
        }
    }

    fn compute_difficulty_value(&self) -> f64 {
        // * Star rating to pp curve
        (self.attrs.stars - 0.15).max(0.05).powf(2.2)
             // * From 80% accuracy, 1/20th of total pp is awarded per additional 1% accuracy
             * (5.0 * self.calculate_custom_accuracy() - 4.0).max(0.0)
             // * Length bonus, capped at 1500 notes
             * (1.0 + 0.1 * (self.total_hits() / 1500.0).min(1.0))
    }

    fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    fn calculate_custom_accuracy(&self) -> f64 {
        let ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            n_misses: _,
        } = &self.state;

        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        custom_accuracy(*n320, *n300, *n200, *n100, *n50, total_hits)
    }
}

impl<'map> From<OsuPP<'map>> for ManiaPP<'map> {
    #[inline]
    fn from(osu: OsuPP<'map>) -> Self {
        let OsuPP {
            map,
            attributes: _,
            mods,
            acc,
            combo: _,
            n300,
            n100,
            n50,
            n_misses,
            passed_objects,
            clock_rate,
            hitresult_priority,
        } = osu;

        let map = map.convert_mode(GameMode::Mania);

        Self {
            is_convert: matches!(map, Cow::Owned(_)),
            map,
            attributes: None,
            mods,
            passed_objects,
            clock_rate,
            n320: None,
            n300,
            n200: None,
            n100,
            n50,
            n_misses,
            acc,
            hitresult_priority,
        }
    }
}

fn custom_accuracy(
    n320: usize,
    n300: usize,
    n200: usize,
    n100: usize,
    n50: usize,
    total_hits: usize,
) -> f64 {
    let numerator = n320 * 32 + n300 * 30 + n200 * 20 + n100 * 10 + n50 * 5;
    let denominator = total_hits * 32;

    numerator as f64 / denominator as f64
}

#[allow(unused)]
fn accuracy(
    n320: usize,
    n300: usize,
    n200: usize,
    n100: usize,
    n50: usize,
    n_misses: usize,
) -> f64 {
    let numerator = 6 * (n320 + n300) + 4 * n200 + 2 * n100 + n50;
    let denominator = 6 * (n320 + n300 + n200 + n100 + n50 + n_misses);

    numerator as f64 / denominator as f64
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait ManiaAttributeProvider {
    /// Provide the actual difficulty attributes.
    fn attributes(self) -> Option<ManiaDifficultyAttributes>;
}

impl ManiaAttributeProvider for ManiaDifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<ManiaDifficultyAttributes> {
        Some(self)
    }
}

impl ManiaAttributeProvider for ManiaPerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<ManiaDifficultyAttributes> {
        Some(self.difficulty)
    }
}

impl ManiaAttributeProvider for DifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<ManiaDifficultyAttributes> {
        if let Self::Mania(attrs) = self {
            Some(attrs)
        } else {
            None
        }
    }
}

impl ManiaAttributeProvider for PerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<ManiaDifficultyAttributes> {
        if let Self::Mania(attrs) = self {
            Some(attrs.difficulty)
        } else {
            None
        }
    }
}

#[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Beatmap;
    use proptest::{option, prelude::*};
    use std::{cmp::Ordering, sync::OnceLock};

    static DATA: OnceLock<(Beatmap, ManiaDifficultyAttributes)> = OnceLock::new();

    const N_OBJECTS: usize = 594;

    fn test_data() -> (&'static Beatmap, ManiaDifficultyAttributes) {
        let (map, attrs) = DATA.get_or_init(|| {
            let path = "./maps/1638954.osu";
            let map = Beatmap::from_path(path).unwrap();

            let attrs = ManiaStars::new(&map).calculate();

            let expected = ManiaDifficultyAttributes {
                stars: 3.441830819988125,
                hit_window: 40.0,
                max_combo: 956,
            };

            assert_eq!(attrs, expected);
            assert_eq!(
                N_OBJECTS,
                (map.n_circles + map.n_sliders + map.n_spinners) as usize
            );

            (map, attrs)
        });

        (map, attrs.to_owned())
    }

    /// Checks most remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`ManiaScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate.
    /// Only slight optimizations have been applied so that it doesn't run unreasonably long.
    fn brute_force_best(
        acc: f64,
        n320: Option<usize>,
        n300: Option<usize>,
        n200: Option<usize>,
        n100: Option<usize>,
        n50: Option<usize>,
        n_misses: usize,
        best_case: bool,
    ) -> ManiaScoreState {
        let mut best_state = ManiaScoreState {
            n_misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;
        let mut best_custom_acc = 0.0;

        let n_remaining = N_OBJECTS - n_misses;

        let multiple_given = (n320.is_some() as usize
            + n300.is_some() as usize
            + n200.is_some() as usize
            + n100.is_some() as usize
            + n50.is_some() as usize)
            > 1;

        let max_left = N_OBJECTS
            .saturating_sub(n200.unwrap_or(0) + n100.unwrap_or(0) + n50.unwrap_or(0) + n_misses);

        let min_n3x0 = max_left
            .min((acc * (3 * N_OBJECTS) as f64 - (2 * n_remaining) as f64).floor() as usize);

        let max_n3x0 = max_left
            .min(((acc * (6 * N_OBJECTS) as f64 - n_remaining as f64) / 5.0).ceil() as usize);

        let (min_n3x0, max_n3x0) = match (n320, n300) {
            (Some(n320), Some(n300)) => {
                (n_remaining.min(n320 + n300), n_remaining.min(n320 + n300))
            }
            (Some(n320), None) => (
                n_remaining.min(n320).max(min_n3x0),
                max_n3x0.max(n320.min(n_remaining)),
            ),
            (None, Some(n300)) => (
                n_remaining.min(n300).max(min_n3x0),
                max_n3x0.max(n300.min(n_remaining)),
            ),
            (None, None) => (min_n3x0, max_n3x0),
        };

        for new3x0 in min_n3x0..=max_n3x0 {
            let max_left =
                n_remaining.saturating_sub(new3x0 + n100.unwrap_or(0) + n50.unwrap_or(0));

            let (min_n200, max_n200) = match (n200, n100, n50) {
                (Some(n200), ..) if multiple_given => {
                    (n_remaining.min(n200), n_remaining.min(n200))
                }
                (Some(n200), ..) => (max_left.min(n200), max_left.min(n200)),
                (None, Some(_), Some(_)) => (max_left, max_left),
                _ => (0, max_left),
            };

            for new200 in min_n200..=max_n200 {
                let max_left = n_remaining.saturating_sub(new3x0 + new200 + n50.unwrap_or(0));

                let (min_n100, max_n100) = match (n100, n50) {
                    (Some(n100), _) if multiple_given => {
                        (n_remaining.min(n100), n_remaining.min(n100))
                    }
                    (Some(n100), _) => (max_left.min(n100), max_left.min(n100)),
                    (None, Some(_)) => (max_left, max_left),
                    (None, None) => (0, max_left),
                };

                for new100 in min_n100..=max_n100 {
                    let max_left = n_remaining.saturating_sub(new3x0 + new200 + new100);

                    let new50 = match n50 {
                        Some(n50) if multiple_given => n_remaining.min(n50),
                        Some(n50) => max_left.min(n50),
                        None => max_left,
                    };

                    let (new320, new300) = match (n320, n300) {
                        (Some(n320), Some(n300)) => (n_remaining.min(n320), n_remaining.min(n300)),
                        (Some(n320), None) => {
                            (n320.min(n_remaining), new3x0 - n320.min(n_remaining))
                        }
                        (None, Some(n300)) => {
                            (new3x0 - n300.min(n_remaining), n300.min(n_remaining))
                        }
                        (None, None) if best_case => (new3x0, 0),
                        (None, None) => (0, new3x0),
                    };

                    let curr_acc = accuracy(new320, new300, new200, new100, new50, n_misses);
                    let curr_dist = (acc - curr_acc).abs();

                    let curr_custom_acc =
                        custom_accuracy(new320, new300, new200, new100, new50, N_OBJECTS);

                    match curr_dist.partial_cmp(&best_dist).expect("non-NaN") {
                        Ordering::Less => {
                            best_dist = curr_dist;
                            best_custom_acc = curr_custom_acc;
                            best_state.n320 = new320;
                            best_state.n300 = new300;
                            best_state.n200 = new200;
                            best_state.n100 = new100;
                            best_state.n50 = new50;
                        }
                        Ordering::Equal if curr_custom_acc < best_custom_acc => {
                            best_custom_acc = curr_custom_acc;
                            best_state.n320 = new320;
                            best_state.n300 = new300;
                            best_state.n200 = new200;
                            best_state.n100 = new100;
                            best_state.n50 = new50;
                        }
                        _ => {}
                    }
                }
            }
        }

        if best_state.n320 + best_state.n300 + best_state.n200 + best_state.n100 + best_state.n50
            < n_remaining
        {
            let n_remaining = n_remaining
                - (best_state.n320
                    + best_state.n300
                    + best_state.n200
                    + best_state.n100
                    + best_state.n50);

            if best_case {
                match (n320, n300, n200, n100, n50) {
                    (None, ..) => best_state.n320 += n_remaining,
                    (_, None, ..) => best_state.n300 += n_remaining,
                    (_, _, None, ..) => best_state.n200 += n_remaining,
                    (.., None, _) => best_state.n100 += n_remaining,
                    (.., None) => best_state.n50 += n_remaining,
                    _ => best_state.n320 += n_remaining,
                }
            } else {
                match (n50, n100, n200, n300, n320) {
                    (None, ..) => best_state.n50 += n_remaining,
                    (_, None, ..) => best_state.n100 += n_remaining,
                    (_, _, None, ..) => best_state.n200 += n_remaining,
                    (.., None, _) => best_state.n300 += n_remaining,
                    (.., None) => best_state.n320 += n_remaining,
                    _ => best_state.n50 += n_remaining,
                }
            }
        }

        if best_case {
            if n320.is_none() && n200.is_none() && n100.is_none() {
                let n = best_state.n200 / 2;
                best_state.n320 += n;
                best_state.n200 -= 2 * n;
                best_state.n100 += n;
            }

            if n100.is_none() && n50.is_none() {
                let n = if n320.is_none() && n300.is_none() {
                    let n = (best_state.n320 + best_state.n300).min(best_state.n50 / 4);

                    let removed320 = best_state.n320.min(n);
                    let removed300 = n - removed320;

                    best_state.n320 -= removed320;
                    best_state.n300 -= removed300;

                    n
                } else if n320.is_none() {
                    let n = best_state.n320.min(best_state.n50 / 4);
                    best_state.n320 -= n;

                    n
                } else if n300.is_none() {
                    let n = best_state.n300.min(best_state.n50 / 4);
                    best_state.n300 -= n;

                    n
                } else {
                    0
                };

                best_state.n100 += 5 * n;
                best_state.n50 -= 4 * n;
            }
        } else if n320.is_none() && n200.is_none() && n100.is_none() {
            let n = best_state.n320.min(best_state.n100);
            best_state.n320 -= n;
            best_state.n200 += 2 * n;
            best_state.n100 -= n;
        }

        best_state
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn mania_hitresults(
            acc in 0.0..=1.0,
            n320 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n300 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n200 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n100 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n50 in option::weighted(0.10, 0_usize..=N_OBJECTS),
            n_misses in option::weighted(0.15, 0_usize..=N_OBJECTS),
            best_case in prop::bool::ANY,
        ) {
            let (map, attrs) = test_data();

            let priority = if best_case {
                HitResultPriority::BestCase
            } else {
                HitResultPriority::WorstCase
            };

            let mut state = ManiaPP::new(map)
                .attributes(attrs)
                .accuracy(acc * 100.0)
                .hitresult_priority(priority);

            if let Some(n320) = n320 {
                state = state.n320(n320);
            }

            if let Some(n300) = n300 {
                state = state.n300(n300);
            }

            if let Some(n200) = n200 {
                state = state.n200(n200);
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

            let hitresults = state.generate_hitresults();

            let expected = brute_force_best(
                acc,
                n320,
                n300,
                n200,
                n100,
                n50,
                n_misses.unwrap_or(0),
                best_case,
            );

            assert_eq!(hitresults, expected);
        }
    }

    #[test]
    fn hitresults_n320_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .n320(500)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 500,
            n300: 92,
            n200: 0,
            n100: 0,
            n50: 0,
            n_misses: 2,
        };

        assert_eq!(state, expected);
        assert_eq!(state.total_hits(), N_OBJECTS);
    }

    #[test]
    fn hitresults_n100_n50_n_misses_worst() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .n100(200)
            .n50(50)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 0,
            n300: 0,
            n200: 342,
            n100: 200,
            n50: 50,
            n_misses: 2,
        };

        assert_eq!(state, expected);
        assert_eq!(state.total_hits(), N_OBJECTS);
    }
}
