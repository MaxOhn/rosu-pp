use std::borrow::Cow;

use super::{ManiaDifficultyAttributes, ManiaPerformanceAttributes, ManiaScoreState, ManiaStars};
use crate::{Beatmap, DifficultyAttributes, GameMode, Mods, OsuPP, PerformanceAttributes};

// TODO: update
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
///     .score(765_432)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = ManiaPP::new(&map)
///     .attributes(pp_result)  // reusing previous results for performance
///     .mods(8 + 64)           // has to be the same to reuse attributes
///     .score(950_000)
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

    n320: Option<usize>,
    n300: Option<usize>,
    n200: Option<usize>,
    n100: Option<usize>,
    n50: Option<usize>,
    n_misses: Option<usize>,

    acc: Option<f64>,
    hitresult_priority: Option<ManiaHitResultPriority>,
}

impl<'map> ManiaPP<'map> {
    /// Create a new performance calculator for osu!mania maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map: Cow::Borrowed(map),
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

    // TODO: update
    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// Be sure you also set [`score`](ManiaPP::score) or the final values
    /// won't be correct because it will incorrectly assume a score of 1,000,000.
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

    #[inline]
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc);

        self
    }

    #[inline]
    pub fn hitresult_priority(mut self, priority: ManiaHitResultPriority) -> Self {
        self.hitresult_priority = Some(priority);

        self
    }

    #[inline]
    pub fn n320(mut self, n320: usize) -> Self {
        self.n320 = Some(n320);

        self
    }

    #[inline]
    pub fn n300(mut self, n300: usize) -> Self {
        self.n300 = Some(n300);

        self
    }

    #[inline]
    pub fn n200(mut self, n200: usize) -> Self {
        self.n200 = Some(n200);

        self
    }

    #[inline]
    pub fn n100(mut self, n100: usize) -> Self {
        self.n100 = Some(n100);

        self
    }

    #[inline]
    pub fn n50(mut self, n50: usize) -> Self {
        self.n50 = Some(n50);

        self
    }

    #[inline]
    pub fn n_misses(mut self, n_misses: usize) -> Self {
        self.n_misses = Some(n_misses);

        self
    }

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
            // TODO: handle converts
            let mut calculator = ManiaStars::new(self.map.as_ref()).mods(self.mods);

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
            clock_rate: self.clock_rate.unwrap_or_else(|| self.mods.clock_rate()),
            state: self.generate_hitresults(),
        };

        inner.calculate()
    }

    fn generate_hitresults(&self) -> ManiaScoreState {
        let n_objects = self.map.hit_objects.len();
        let priority = self.hitresult_priority.unwrap_or_default();

        let mut state = ManiaScoreState {
            n320: self.n320.unwrap_or(0),
            n300: self.n300.unwrap_or(0),
            n200: self.n200.unwrap_or(0),
            n100: self.n100.unwrap_or(0),
            n50: self.n50.unwrap_or(0),
            n_misses: self.n_misses.unwrap_or(0),
        };

        if let Some(acc) = self.acc {
            let target_total = (acc * (n_objects * 6) as f64).round() as usize;

            let mut delta = target_total.saturating_sub(n_objects.saturating_sub(state.n_misses));

            if self.n50.is_some() {
                delta /= 2;
            }

            if self.n100.is_some() {
                delta /= 2;
            }

            if let Some(n320) = self.n320 {
                delta = delta.saturating_sub(n320 * 6);
            } else {
                state.n320 = delta / 5;
            }

            if self.n100.is_none() {
                state.n100 = delta % 5;
            }

            state.n50 += n_objects.saturating_sub(state.total_hits() - state.n50);

            if let ManiaHitResultPriority::BestCase = priority {
                // Shift n50 to n200
                if self.n320.or(self.n300).or(self.n200).or(self.n50).is_none() {
                    let n = (state.n320 + state.n300).min(state.n50 / 2);

                    if n <= state.n300 {
                        state.n300 -= n;
                    } else {
                        state.n320 -= n - state.n300;
                        state.n300 = 0;
                    };

                    state.n200 += 2 * n;
                    state.n50 -= n;
                }

                // Shift n50 to n100
                if self.n320.or(self.n300).or(self.n100).or(self.n50).is_none() {
                    let n = (state.n320 + state.n300).min(state.n50 / 4);

                    if n <= state.n300 {
                        state.n300 -= n;
                    } else {
                        state.n320 -= n - state.n300;
                        state.n300 = 0;
                    };

                    state.n100 += 5 * n;
                    state.n50 -= 4 * n;
                }
            }
        } else {
            let remaining = n_objects.saturating_sub(state.total_hits());

            match priority {
                ManiaHitResultPriority::BestCase => {
                    if self.n320.is_none() {
                        state.n320 = remaining;
                    } else if self.n300.is_none() {
                        state.n300 = remaining;
                    } else if self.n200.is_none() {
                        state.n200 = remaining;
                    } else if self.n100.is_none() {
                        state.n100 = remaining;
                    } else if self.n50.is_none() {
                        state.n50 = remaining;
                    } else {
                        state.n320 = remaining;
                    }
                }
                ManiaHitResultPriority::WorstCase => {
                    if self.n50.is_none() {
                        state.n50 = remaining;
                    } else if self.n100.is_none() {
                        state.n100 = remaining;
                    } else if self.n200.is_none() {
                        state.n200 = remaining;
                    } else if self.n300.is_none() {
                        state.n300 = remaining;
                    } else if self.n320.is_none() {
                        state.n320 = remaining;
                    } else {
                        state.n50 = remaining;
                    }
                }
            }
        }

        state
    }
}

struct ManiaPpInner {
    attrs: ManiaDifficultyAttributes,
    mods: u32,
    clock_rate: f64,
    state: ManiaScoreState,
}

impl ManiaPpInner {
    fn calculate(self) -> ManiaPerformanceAttributes {
        // * Arbitrary initial value for scaling pp in order to standardize distributions across game modes.
        // * The specific number has no intrinsic meaning and can be adjusted as needed.
        let mut multiplier = 0.8;

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
        // Star rating to pp curve
        (self.attrs.stars - 0.15).max(0.05).powf(2.2)
             // From 80% accuracy, 1/20th of total pp is awarded per additional 1% accuracy
             * (5.0 * self.custom_accuracy() - 4.0).max(0.0)
             // Length bonus, capped at 1500 notes
             * (1.0 + 0.1 * (self.total_hits() / 1500.0).min(1.0))
    }

    fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    fn custom_accuracy(&self) -> f64 {
        let ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            n_misses,
        } = &self.state;

        let numerator = *n320 * 320 + *n300 * 300 + *n200 * 200 + *n100 * 100 + *n50 * 50;
        let denominator = self.total_hits() * 320.0;

        numerator as f64 / denominator
    }
}

/// While generating hitresults that weren't specific, decide how they should be distributed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ManiaHitResultPriority {
    /// Prioritize good hitresults over bad ones
    BestCase,
    /// Prioritize bad hitresults over good ones
    WorstCase,
}

impl Default for ManiaHitResultPriority {
    #[inline]
    fn default() -> Self {
        Self::BestCase
    }
}

impl<'map> From<OsuPP<'map>> for ManiaPP<'map> {
    #[inline]
    fn from(osu: OsuPP<'map>) -> Self {
        let OsuPP {
            map,
            mods,
            passed_objects,
            clock_rate,
            ..
        } = osu;

        Self {
            map: map.convert_mode(GameMode::Mania),
            attributes: None,
            mods,
            passed_objects,
            clock_rate,
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
