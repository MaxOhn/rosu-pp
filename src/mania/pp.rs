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
        Self {
            map: map.convert_mode(GameMode::Mania),
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
    /// [`ManiaGradualPerformanceAttributes`](crate::mania::ManiaGradualPerformanceAttributes).
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

        let mut n320 = self.n320.unwrap_or(0);
        let mut n300 = self.n300.unwrap_or(0);
        let mut n200 = self.n200.unwrap_or(0);
        let mut n100 = self.n100.unwrap_or(0);
        let mut n50 = self.n50.unwrap_or(0);
        let n_misses = self.n_misses.unwrap_or(0);

        if let Some(acc) = self.acc {
            let target_total = (acc * (n_objects * 6) as f64).round() as usize;

            match (self.n320, self.n300, self.n200, self.n100, self.n50) {
                (Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                    let remaining =
                        n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n320 += remaining,
                        HitResultPriority::WorstCase => n50 += remaining,
                    }
                }
                (Some(_), None, Some(_), Some(_), Some(_)) => {
                    n300 = n_objects.saturating_sub(n320 + n200 + n100 + n50 + n_misses)
                }
                (None, Some(_), Some(_), Some(_), Some(_)) => {
                    n320 = n_objects.saturating_sub(n300 + n200 + n100 + n50 + n_misses)
                }
                (Some(_), _, Some(_), Some(_), None) | (_, Some(_), Some(_), Some(_), None) => {
                    n50 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n_misses);
                }
                (Some(_), _, _, None, None) | (_, Some(_), _, None, None) => {
                    let n3x0 = n320 + n300;
                    let delta = (target_total - n_objects.saturating_sub(n_misses))
                        .saturating_sub(n3x0 * 5 + n200 * 3);

                    n100 = delta % 5;
                    n50 = n_objects.saturating_sub(n3x0 + n200 + n100 + n_misses);

                    let curr_total = 6 * n3x0 + 4 * n200 + 2 * n100 + n50;

                    if curr_total < target_total {
                        let n = (target_total - curr_total).min(n50);
                        n50 -= n;
                        n100 += n;
                    } else {
                        let n = (curr_total - target_total).min(n100);
                        n100 -= n;
                        n50 += n;
                    }
                }
                (Some(_), _, None, Some(_), None) | (_, Some(_), None, Some(_), None) => {
                    let n3x0 = n320 + n300;
                    let delta = (target_total - n_objects.saturating_sub(n_misses))
                        .saturating_sub(n3x0 * 5 + n100);

                    n200 = delta / 3;
                    n50 = n_objects.saturating_sub(n3x0 + n200 + n100 + n_misses);
                }
                (Some(_), _, None, None, Some(_)) | (_, Some(_), None, None, Some(_)) => {
                    let remaining = n_objects.saturating_sub(n320 + n300 + n50 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n100 = remaining,
                        HitResultPriority::WorstCase => n200 = remaining,
                    }
                }
                (Some(_), _, None, Some(_), Some(_)) | (_, Some(_), None, Some(_), Some(_)) => {
                    n200 = n_objects.saturating_sub(n320 + n300 + n100 + n50 + n_misses);
                }
                (Some(_), _, Some(_), None, Some(_)) | (_, Some(_), Some(_), None, Some(_)) => {
                    n100 = n_objects.saturating_sub(n320 + n300 + n200 + n50 + n_misses);
                }
                (None, None, Some(_), Some(_), Some(_)) => {
                    let remaining = n_objects.saturating_sub(n200 + n100 + n50 + n_misses);

                    match priority {
                        HitResultPriority::BestCase => n320 = remaining,
                        HitResultPriority::WorstCase => n300 = remaining,
                    }
                }
                (None, None, None, Some(_), Some(_)) => {
                    let delta =
                        (target_total - n_objects.saturating_sub(n_misses)).saturating_sub(n100);

                    match priority {
                        HitResultPriority::BestCase => n320 = delta / 5,
                        HitResultPriority::WorstCase => n300 = delta / 5,
                    }

                    n200 = n_objects.saturating_sub(n320 + n100 + n50 + n_misses);

                    let curr_total = 6 * (n320 + n300) + 4 * n200 + 2 * n100 + n50;

                    if curr_total < target_total {
                        let n = n200.min((target_total - curr_total) / 2);
                        n200 -= n;

                        match priority {
                            HitResultPriority::BestCase => n320 += n,
                            HitResultPriority::WorstCase => n300 += n,
                        }
                    } else {
                        let n = (n320 + n300).min((curr_total - target_total) / 2);
                        n200 += n;

                        match priority {
                            HitResultPriority::BestCase => n320 -= n,
                            HitResultPriority::WorstCase => n300 -= n,
                        }
                    }
                }
                (None, None, Some(_), None, None) => {
                    let delta = (target_total - n_objects.saturating_sub(n_misses))
                        .saturating_sub(n200 * 3);

                    match priority {
                        HitResultPriority::BestCase => n320 = delta / 5,
                        HitResultPriority::WorstCase => n300 = delta / 5,
                    }

                    n100 = delta % 5;
                    n50 = n_objects.saturating_sub(n320 + n200 + n100 + n_misses);

                    let curr_total = 6 * (n320 + n300) + 4 * n200 * 2 * n100 + n50;

                    if curr_total < target_total {
                        let n = (target_total - curr_total).min(n50);
                        n50 -= n;
                        n100 += n;
                    } else {
                        let n = (curr_total - target_total).min(n100);
                        n100 -= n;
                        n50 += n;
                    }

                    if let HitResultPriority::BestCase = priority {
                        // Shift n50 to n100
                        let n = n320.min(n50 / 4);

                        n320 -= n;
                        n100 += 5 * n;
                        n50 -= 4 * n;
                    }
                }
                (None, None, _, Some(_), None) => {
                    let delta = (target_total - n_objects.saturating_sub(n_misses))
                        .saturating_sub(n200 * 3 + n100);

                    match priority {
                        HitResultPriority::BestCase => n320 = delta / 5,
                        HitResultPriority::WorstCase => n300 = delta / 5,
                    }

                    n50 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n_misses);
                }
                (None, None, _, None, Some(_)) => {
                    let delta =
                        target_total - n_objects.saturating_sub(n_misses).saturating_sub(n200 * 3);

                    match priority {
                        HitResultPriority::BestCase => n320 = delta / 5,
                        HitResultPriority::WorstCase => n300 = delta / 5,
                    }

                    n100 = delta % 5;
                    n100 += n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + n_misses);

                    let curr_total = 6 * (n320 + n300) + 4 * n200 + 2 * n100 + n50;

                    if curr_total < target_total {
                        let n = n100.min((target_total - curr_total) / 4);
                        n100 -= n;

                        match priority {
                            HitResultPriority::BestCase => n320 += n,
                            HitResultPriority::WorstCase => n300 += n,
                        }
                    } else {
                        let n = (n320 + n300).min((curr_total - target_total) / 4);
                        n100 += n;

                        match priority {
                            HitResultPriority::BestCase => n320 -= n,
                            HitResultPriority::WorstCase => n300 -= n,
                        }
                    }
                }
                (None, None, None, None, None) => {
                    let delta = target_total - n_objects.saturating_sub(n_misses);

                    match priority {
                        HitResultPriority::BestCase => n320 = delta / 5,
                        HitResultPriority::WorstCase => n300 = delta / 5,
                    }

                    n100 = delta % 5;
                    n50 = n_objects.saturating_sub(n320 + n300 + n100 + n_misses);

                    if let HitResultPriority::BestCase = priority {
                        // Shift n50 to n100
                        let n = n320.min(n50 / 4);
                        n320 -= n;
                        n100 += 5 * n;
                        n50 -= 4 * n;
                    }
                }
            }
        } else {
            let remaining = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + n_misses);

            match priority {
                HitResultPriority::BestCase => {
                    if self.n320.is_none() {
                        n320 = remaining;
                    } else if self.n300.is_none() {
                        n300 = remaining;
                    } else if self.n200.is_none() {
                        n200 = remaining;
                    } else if self.n100.is_none() {
                        n100 = remaining;
                    } else if self.n50.is_none() {
                        n50 = remaining;
                    } else {
                        n320 += remaining;
                    }
                }
                HitResultPriority::WorstCase => {
                    if self.n50.is_none() {
                        n50 = remaining;
                    } else if self.n100.is_none() {
                        n100 = remaining;
                    } else if self.n200.is_none() {
                        n200 = remaining;
                    } else if self.n300.is_none() {
                        n300 = remaining;
                    } else if self.n320.is_none() {
                        n320 = remaining;
                    } else {
                        n50 += remaining;
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

        let numerator = *n320 * 32 + *n300 * 30 + *n200 * 20 + *n100 * 10 + *n50 * 5;
        let denominator = total_hits * 32;

        numerator as f64 / denominator as f64
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

        Self {
            map: map.convert_mode(GameMode::Mania),
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

    fn test_data() -> (Beatmap, ManiaDifficultyAttributes) {
        let path = "./maps/1974394.osu";
        let map = Beatmap::from_path(path).unwrap();

        let attrs = ManiaDifficultyAttributes {
            stars: 4.824631127426499,
            hit_window: 40.0,
            max_combo: 5064,
        };

        (map, attrs)
    }

    #[test]
    fn hitresults_acc_n320_n200_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n320(2600)
            .n200(400)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2600,
            n300: 0,
            n200: 400,
            n100: 49,
            n50: 187,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n320_n300_n200_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n320(2250)
            .n300(500)
            .n200(100)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2250,
            n300: 500,
            n200: 100,
            n100: 199,
            n50: 187,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n320_n300_n100_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n320(2000)
            .n300(500)
            .n100(100)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2000,
            n300: 500,
            n200: 549,
            n100: 100,
            n50: 87,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n320_n100_n50_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n320(2700)
            .n100(200)
            .n50(10)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2700,
            n300: 0,
            n200: 326,
            n100: 200,
            n50: 10,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n320_n50_n_misses_worst() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n320(2000)
            .n50(50)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2000,
            n300: 0,
            n200: 1186,
            n100: 0,
            n50: 50,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n100_n50_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n100(200)
            .n50(50)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2546,
            n300: 0,
            n200: 440,
            n100: 200,
            n50: 50,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n200_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n200(500)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2503,
            n300: 0,
            n200: 500,
            n100: 230,
            n50: 3,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n200_n100_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n200(500)
            .n100(200)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2509,
            n300: 0,
            n200: 500,
            n100: 200,
            n50: 27,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n50_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n50(200)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2804,
            n300: 0,
            n200: 0,
            n100: 232,
            n50: 200,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n200_n100_n50_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n200(500)
            .n100(300)
            .n50(100)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2336,
            n300: 0,
            n200: 500,
            n100: 300,
            n50: 100,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_acc_n_misses_worst() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .accuracy(90.0)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 0,
            n300: 2849,
            n200: 0,
            n100: 4,
            n50: 383,
            n_misses: 2,
        };

        assert_eq!(
            state,
            expected,
            "{}% vs {}%",
            state.accuracy(),
            expected.accuracy()
        );
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_n320_n_misses_best() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .n320(2000)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 2000,
            n300: 1236,
            n200: 0,
            n100: 0,
            n50: 0,
            n_misses: 2,
        };

        assert_eq!(state, expected);
        assert_eq!(state.total_hits(), 3238);
    }

    #[test]
    fn hitresults_n100_n50_n_misses_worst() {
        let (map, attrs) = test_data();

        let state = ManiaPP::new(&map)
            .attributes(attrs)
            .n100(500)
            .n50(100)
            .n_misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_hitresults();

        let expected = ManiaScoreState {
            n320: 0,
            n300: 0,
            n200: 2636,
            n100: 500,
            n50: 100,
            n_misses: 2,
        };

        assert_eq!(state, expected);
        assert_eq!(state.total_hits(), 3238);
    }
}
