use std::cmp;

use crate::{
    any::{Difficulty, HitResultPriority, IntoModePerformance, IntoPerformance},
    osu::OsuPerformance,
    util::{map_or_attrs::MapOrAttrs, mods::Mods},
    Performance,
};

use super::{
    attributes::{ManiaDifficultyAttributes, ManiaPerformanceAttributes},
    score_state::ManiaScoreState,
    Mania,
};

pub mod gradual;

/// Performance calculator on osu!mania maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ManiaPerformance<'map> {
    map_or_attrs: MapOrAttrs<'map, Mania>,
    difficulty: Difficulty,
    n320: Option<u32>,
    n300: Option<u32>,
    n200: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    misses: Option<u32>,
    acc: Option<f64>,
    hitresult_priority: HitResultPriority,
}

impl<'map> ManiaPerformance<'map> {
    /// Create a new performance calculator for osu!mania maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`ManiaDifficultyAttributes`]
    /// or [`ManiaPerformanceAttributes`])
    /// - a beatmap ([`ManiaBeatmap<'map>`])
    ///
    /// If a map is given, difficulty attributes will need to be calculated
    /// internally which is a costly operation. Hence, passing attributes
    /// should be prefered.
    ///
    /// However, when passing previously calculated attributes, make sure they
    /// have been calculated for the same map and [`Difficulty`] settings.
    /// Otherwise, the final attributes will be incorrect.
    ///
    /// [`ManiaBeatmap<'map>`]: crate::mania::ManiaBeatmap
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, Mania>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu!mania maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu!mania e.g.
    /// a [`Converted`], [`DifficultyAttributes`], or [`PerformanceAttributes`]
    /// of a different mode.
    ///
    /// See [`ManiaPerformance::new`] for more information.
    ///
    /// [`Converted`]: crate::model::beatmap::Converted
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::Mania(calc) = map_or_attrs.into_performance() {
            Some(calc)
        } else {
            None
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See <https://github.com/ppy/osu-api/wiki#mods>
    pub const fn mods(mut self, mods: u32) -> Self {
        self.difficulty = self.difficulty.mods(mods);

        self
    }

    /// Use the specified settings of the given [`Difficulty`].
    pub const fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects,
    /// instead of using [`ManiaPerformance`] multiple times with different
    /// `passed_objects`, you should use [`ManiaGradualPerformance`].
    ///
    /// [`ManiaGradualPerformance`]: crate::mania::ManiaGradualPerformance
    pub const fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.difficulty = self.difficulty.passed_objects(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | 0.01    | 100     |
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.difficulty = self.difficulty.clock_rate(clock_rate);

        self
    }

    /// Override a beatmap's set HP.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn hp(mut self, hp: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.hp(hp, with_mods);

        self
    }

    /// Override a beatmap's set OD.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn od(mut self, od: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.od(od, with_mods);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Specify how hitresults should be generated.
    ///
    /// Defauls to [`HitResultPriority::BestCase`].
    pub const fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = priority;

        self
    }

    /// Specify the amount of 320s of a play.
    pub const fn n320(mut self, n320: u32) -> Self {
        self.n320 = Some(n320);

        self
    }

    /// Specify the amount of 300s of a play.
    pub const fn n300(mut self, n300: u32) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 200s of a play.
    pub const fn n200(mut self, n200: u32) -> Self {
        self.n200 = Some(n200);

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
    pub const fn misses(mut self, n_misses: u32) -> Self {
        self.misses = Some(n_misses);

        self
    }

    /// Provide parameters through an [`ManiaScoreState`].
    #[allow(clippy::needless_pass_by_value)]
    pub const fn state(mut self, state: ManiaScoreState) -> Self {
        let ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            misses,
        } = state;

        self.n320 = Some(n320);
        self.n300 = Some(n300);
        self.n200 = Some(n200);
        self.n100 = Some(n100);
        self.n50 = Some(n50);
        self.misses = Some(misses);

        self
    }

    /// Create the [`ManiaScoreState`] that will be used for performance calculation.
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub fn generate_state(&mut self) -> ManiaScoreState {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.difficulty.with_mode().calculate(map);

                self.map_or_attrs.insert_attrs(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let n_objects = cmp::min(self.difficulty.get_passed_objects() as u32, attrs.n_objects);

        let priority = self.hitresult_priority;

        let misses = self.misses.map_or(0, |n| cmp::min(n, n_objects));
        let n_remaining = n_objects - misses;

        let mut n320 = self.n320.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n300 = self.n300.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n200 = self.n200.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n100 = self.n100.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n50 = self.n50.map_or(0, |n| cmp::min(n, n_remaining));

        if let Some(acc) = self.acc {
            let target_total = acc * f64::from(6 * n_objects);

            match (self.n320, self.n300, self.n200, self.n100, self.n50) {
                // All hitresults given
                (Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                    let remaining =
                        n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + misses);

                    match priority {
                        HitResultPriority::BestCase => n320 += remaining,
                        HitResultPriority::WorstCase => n50 += remaining,
                    }
                }

                // All but one hitresults given
                (None, Some(_), Some(_), Some(_), Some(_)) => {
                    n320 = n_objects.saturating_sub(n300 + n200 + n100 + n50 + misses);
                }
                (Some(_), None, Some(_), Some(_), Some(_)) => {
                    n300 = n_objects.saturating_sub(n320 + n200 + n100 + n50 + misses);
                }
                (Some(_), Some(_), None, Some(_), Some(_)) => {
                    n200 = n_objects.saturating_sub(n320 + n300 + n100 + n50 + misses);
                }
                (Some(_), Some(_), Some(_), None, Some(_)) => {
                    n100 = n_objects.saturating_sub(n320 + n300 + n200 + n50 + misses);
                }
                (Some(_), Some(_), Some(_), Some(_), None) => {
                    n50 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + misses);
                }

                // n200, n100, and n50 given
                (None, None, Some(_), Some(_), Some(_)) => {
                    let n_remaining =
                        n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + misses);

                    match priority {
                        HitResultPriority::BestCase => n320 = n_remaining,
                        HitResultPriority::WorstCase => n300 = n_remaining,
                    }
                }

                // n100 and n50 given
                (.., None, Some(_), Some(_)) => {
                    let mut best_dist = f64::INFINITY;
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n100 + n50 + misses);

                    let raw_n3x0 = (target_total - f64::from(4 * n_remaining)
                        + f64::from(2 * n100 + 3 * n50))
                        / 2.0;
                    let min_n3x0 = cmp::min(
                        raw_n3x0.floor() as u32,
                        n_remaining.saturating_sub(n100 + n50),
                    );
                    let max_n3x0 = cmp::min(
                        raw_n3x0.ceil() as u32,
                        n_remaining.saturating_sub(n100 + n50),
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (n320 + n300, n320 + n300),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let new200 = n_remaining.saturating_sub(new3x0 + n100 + n50);
                        let curr_dist =
                            (acc - accuracy(new3x0, 0, new200, n100, n50, misses)).abs();

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
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n50 + misses);

                    let raw_n3x0 = (target_total - f64::from(2 * (n_remaining + n200) - n50)) / 4.0;
                    let min_n3x0 = cmp::min(
                        raw_n3x0.floor() as u32,
                        n_remaining.saturating_sub(n200 + n50),
                    );
                    let max_n3x0 = cmp::min(
                        raw_n3x0.ceil() as u32,
                        n_remaining.saturating_sub(n200 + n50),
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (n320 + n300, n320 + n300),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let new100 = n_remaining.saturating_sub(new3x0 + n200 + n50);
                        let curr_dist =
                            (acc - accuracy(new3x0, 0, n200, new100, n50, misses)).abs();

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
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + misses);

                    let raw_n3x0 = (target_total - f64::from(n_remaining + 3 * n200 + n100)) / 5.0;
                    let min_n3x0 = cmp::min(
                        raw_n3x0.floor() as u32,
                        n_remaining.saturating_sub(n200 + n100),
                    );
                    let max_n3x0 = cmp::min(
                        raw_n3x0.ceil() as u32,
                        n_remaining.saturating_sub(n200 + n100),
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (n320 + n300, n320 + n300),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let new50 = n_remaining.saturating_sub(new3x0 + n200 + n100);
                        let curr_dist =
                            (acc - accuracy(new3x0, 0, n200, n100, new50, misses)).abs();

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
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + misses);

                    let min_n3x0 = cmp::min(
                        ((target_total - f64::from(2 * (n_remaining + n200))) / 4.0).floor() as u32,
                        n_remaining - n200,
                    );

                    let max_n3x0 = cmp::min(
                        ((target_total - f64::from(n_remaining + 3 * n200)) / 5.0).ceil() as u32,
                        n_remaining - n200,
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (
                            cmp::min(n_remaining, n320 + n300),
                            cmp::min(n_remaining, n320 + n300),
                        ),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let raw_n100 =
                            target_total - f64::from(n_remaining + 5 * new3x0 + 3 * n200);
                        let min_n100 = cmp::min(
                            raw_n100.floor() as u32,
                            n_remaining.saturating_sub(new3x0 + n200),
                        );
                        let max_n100 = cmp::min(
                            raw_n100.ceil() as u32,
                            n_remaining.saturating_sub(new3x0 + n200),
                        );

                        for new100 in min_n100..=max_n100 {
                            let new50 = n_remaining.saturating_sub(new3x0 + n200 + new100);
                            let curr_dist =
                                (acc - accuracy(new3x0, 0, n200, new100, new50, misses)).abs();

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
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n100 + misses);

                    let min_n3x0 = cmp::min(
                        (acc * f64::from(3 * n_remaining) - f64::from(2 * n_remaining - n100))
                            .floor() as u32,
                        n_remaining - n100,
                    );

                    let max_n3x0 = cmp::min(
                        ((target_total - f64::from(n_remaining + n100)) / 5.0).ceil() as u32,
                        n_remaining - n100,
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (
                            cmp::min(n_remaining, n320 + n300),
                            cmp::min(n_remaining, n320 + n300),
                        ),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let raw_n200 =
                            (target_total - f64::from(n_remaining + 5 * new3x0 + n100)) / 3.0;
                        let min_n200 = cmp::min(
                            raw_n200.floor() as u32,
                            n_remaining.saturating_sub(new3x0 + n100),
                        );
                        let max_n200 = cmp::min(
                            raw_n200.ceil() as u32,
                            n_remaining.saturating_sub(new3x0 + n100),
                        );

                        for new200 in min_n200..=max_n200 {
                            let new50 = n_remaining.saturating_sub(new3x0 + new200 + n100);
                            let curr_dist =
                                (acc - accuracy(new3x0, 0, new200, n100, new50, misses)).abs();

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
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n50 + misses);

                    let min_n3x0 = cmp::min(
                        ((target_total - f64::from(4 * n_remaining - 3 * n50)) / 2.0).floor()
                            as u32,
                        n_remaining - n50,
                    );

                    let max_n3x0 = cmp::min(
                        ((target_total - f64::from(2 * n_remaining - n50)) / 4.0).ceil() as u32,
                        n_remaining - n50,
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (
                            cmp::min(n_remaining, n320 + n300),
                            cmp::min(n_remaining, n320 + n300),
                        ),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let raw_n200 = (target_total - f64::from(2 * n_remaining + 4 * new3x0)
                            + f64::from(n50))
                            / 2.0;
                        let min_n200 = cmp::min(
                            raw_n200.floor() as u32,
                            n_remaining.saturating_sub(new3x0 + n50),
                        );
                        let max_n200 = cmp::min(
                            raw_n200.ceil() as u32,
                            n_remaining.saturating_sub(new3x0 + n50),
                        );

                        for new200 in min_n200..=max_n200 {
                            let new100 = n_remaining.saturating_sub(new3x0 + new200 + n50);
                            let curr_dist =
                                (acc - accuracy(new3x0, 0, new200, new100, n50, misses)).abs();

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
                    let mut n3x0 = n_objects.saturating_sub(n320 + n300 + n200 + n100 + misses);

                    let min_n3x0 = cmp::min(
                        ((target_total - f64::from(4 * n_remaining)) / 5.0).floor() as u32,
                        n_remaining,
                    );

                    let max_n3x0 = cmp::min(
                        ((target_total - f64::from(n_remaining)) / 5.0)
                            .min(acc * f64::from(3 * n_objects) - f64::from(n_remaining))
                            .ceil() as u32,
                        n_remaining,
                    );

                    let (min_n3x0, max_n3x0) = match (self.n320, self.n300) {
                        (Some(_), Some(_)) => (
                            cmp::min(n_remaining, n320 + n300),
                            cmp::min(n_remaining, n320 + n300),
                        ),
                        (Some(_), None) => (cmp::max(min_n3x0, n320), cmp::max(max_n3x0, n320)),
                        (None, Some(_)) => (cmp::max(min_n3x0, n300), cmp::max(max_n3x0, n300)),
                        (None, None) => (min_n3x0, max_n3x0),
                    };

                    for new3x0 in min_n3x0..=max_n3x0 {
                        let min_n200 = cmp::min(
                            (acc * f64::from(3 * n_objects) - f64::from(n_remaining + 2 * new3x0))
                                .floor() as u32,
                            n_remaining - new3x0,
                        );

                        let max_n200 = cmp::min(
                            ((target_total - f64::from(n_remaining + 5 * new3x0)) / 3.0).ceil()
                                as u32,
                            n_remaining - new3x0,
                        );

                        for new200 in min_n200..=max_n200 {
                            let raw_n100 =
                                target_total - f64::from(n_remaining + 5 * new3x0 + 3 * new200);
                            let min_n100 =
                                cmp::min(raw_n100.floor() as u32, n_remaining - (new3x0 + new200));
                            let max_n100 =
                                cmp::min(raw_n100.ceil() as u32, n_remaining - (new3x0 + new200));

                            for new100 in min_n100..=max_n100 {
                                let new50 = n_remaining - new3x0 - new200 - new100;
                                let curr_acc = accuracy(new3x0, 0, new200, new100, new50, misses);
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
            let remaining = n_objects.saturating_sub(n320 + n300 + n200 + n100 + n50 + misses);

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
            misses,
        }
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> ManiaPerformanceAttributes {
        let state = self.generate_state();

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => self.difficulty.with_mode().calculate(map),
            MapOrAttrs::Attrs(attrs) => attrs,
        };

        let inner = ManiaPerformanceInner {
            mods: self.difficulty.get_mods(),
            attrs,
            state,
        };

        inner.calculate()
    }

    pub(crate) const fn from_map_or_attrs(map_or_attrs: MapOrAttrs<'map, Mania>) -> Self {
        Self {
            map_or_attrs,
            difficulty: Difficulty::new(),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: None,
            acc: None,
            hitresult_priority: HitResultPriority::DEFAULT,
        }
    }
}

impl<'map> TryFrom<OsuPerformance<'map>> for ManiaPerformance<'map> {
    type Error = OsuPerformance<'map>;

    /// Try to create [`ManiaPerformance`] through [`OsuPerformance`].
    ///
    /// Returns `None` if [`OsuPerformance`] does not contain a beatmap, i.e.
    /// if it was constructed through attributes or
    /// [`OsuPerformance::generate_state`] was called.
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
            difficulty,
            acc,
            combo: _,
            n300,
            n100,
            n50,
            misses,
            hitresult_priority,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            difficulty,
            n320: None,
            n300,
            n200: None,
            n100,
            n50,
            misses,
            acc,
            hitresult_priority,
        })
    }
}

impl<'map, T: IntoModePerformance<'map, Mania>> From<T> for ManiaPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

struct ManiaPerformanceInner {
    attrs: ManiaDifficultyAttributes,
    mods: u32,
    state: ManiaScoreState,
}

impl ManiaPerformanceInner {
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

    const fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    fn calculate_custom_accuracy(&self) -> f64 {
        let ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            misses: _,
        } = &self.state;

        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        custom_accuracy(*n320, *n300, *n200, *n100, *n50, total_hits)
    }
}

fn custom_accuracy(n320: u32, n300: u32, n200: u32, n100: u32, n50: u32, total_hits: u32) -> f64 {
    let numerator = n320 * 32 + n300 * 30 + n200 * 20 + n100 * 10 + n50 * 5;
    let denominator = total_hits * 32;

    f64::from(numerator) / f64::from(denominator)
}

fn accuracy(n320: u32, n300: u32, n200: u32, n100: u32, n50: u32, misses: u32) -> f64 {
    let numerator = 6 * (n320 + n300) + 4 * n200 + 2 * n100 + n50;
    let denominator = 6 * (n320 + n300 + n200 + n100 + n50 + misses);

    f64::from(numerator) / f64::from(denominator)
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, sync::OnceLock};

    use proptest::prelude::*;
    use rosu_map::section::general::GameMode;

    use crate::{
        any::{DifficultyAttributes, PerformanceAttributes},
        osu::{Osu, OsuDifficultyAttributes, OsuPerformanceAttributes},
        Beatmap,
    };

    use super::*;

    static ATTRS: OnceLock<ManiaDifficultyAttributes> = OnceLock::new();

    const N_OBJECTS: u32 = 594;

    fn beatmap() -> Beatmap {
        Beatmap::from_path("./resources/1638954.osu").unwrap()
    }

    fn attrs() -> ManiaDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let converted = beatmap().unchecked_into_converted::<Mania>();
                let attrs = Difficulty::new().with_mode().calculate(&converted);

                assert_eq!(N_OBJECTS, converted.hit_objects.len() as u32);

                attrs
            })
            .to_owned()
    }

    /// Checks most remaining hitresult combinations w.r.t. the given parameters
    /// and returns the [`ManiaScoreState`] that matches `acc` the best.
    ///
    /// Very slow but accurate. Only slight optimizations have been applied so
    /// that it doesn't run unreasonably long.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn brute_force_best(
        acc: f64,
        n320: Option<u32>,
        n300: Option<u32>,
        n200: Option<u32>,
        n100: Option<u32>,
        n50: Option<u32>,
        misses: u32,
        best_case: bool,
    ) -> ManiaScoreState {
        let misses = cmp::min(misses, N_OBJECTS);

        let mut best_state = ManiaScoreState {
            misses,
            ..Default::default()
        };

        let mut best_dist = f64::INFINITY;
        let mut best_custom_acc = 0.0;

        let n_remaining = N_OBJECTS - misses;

        let multiple_given = (usize::from(n320.is_some())
            + usize::from(n300.is_some())
            + usize::from(n200.is_some())
            + usize::from(n100.is_some())
            + usize::from(n50.is_some()))
            > 1;

        let max_left = N_OBJECTS
            .saturating_sub(n200.unwrap_or(0) + n100.unwrap_or(0) + n50.unwrap_or(0) + misses);

        let min_n3x0 = cmp::min(
            max_left,
            (acc * f64::from(3 * N_OBJECTS) - f64::from(2 * n_remaining)).floor() as u32,
        );

        let max_n3x0 = cmp::min(
            max_left,
            ((acc * f64::from(6 * N_OBJECTS) - f64::from(n_remaining)) / 5.0).ceil() as u32,
        );

        let (min_n3x0, max_n3x0) = match (n320, n300) {
            (Some(n320), Some(n300)) => (
                cmp::min(n_remaining, n320 + n300),
                cmp::min(n_remaining, n320 + n300),
            ),
            (Some(n320), None) => (
                cmp::max(cmp::min(n_remaining, n320), min_n3x0),
                cmp::max(max_n3x0, cmp::min(n320, n_remaining)),
            ),
            (None, Some(n300)) => (
                cmp::max(cmp::min(n_remaining, n300), min_n3x0),
                cmp::max(max_n3x0, cmp::min(n300, n_remaining)),
            ),
            (None, None) => (min_n3x0, max_n3x0),
        };

        for new3x0 in min_n3x0..=max_n3x0 {
            let max_left =
                n_remaining.saturating_sub(new3x0 + n100.unwrap_or(0) + n50.unwrap_or(0));

            let (min_n200, max_n200) = match (n200, n100, n50) {
                (Some(n200), ..) if multiple_given => {
                    (cmp::min(n_remaining, n200), cmp::min(n_remaining, n200))
                }
                (Some(n200), ..) => (cmp::min(max_left, n200), cmp::min(max_left, n200)),
                (None, Some(_), Some(_)) => (max_left, max_left),
                _ => (0, max_left),
            };

            for new200 in min_n200..=max_n200 {
                let max_left = n_remaining.saturating_sub(new3x0 + new200 + n50.unwrap_or(0));

                let (min_n100, max_n100) = match (n100, n50) {
                    (Some(n100), _) if multiple_given => {
                        (cmp::min(n_remaining, n100), cmp::min(n_remaining, n100))
                    }
                    (Some(n100), _) => (cmp::min(max_left, n100), cmp::min(max_left, n100)),
                    (None, Some(_)) => (max_left, max_left),
                    (None, None) => (0, max_left),
                };

                for new100 in min_n100..=max_n100 {
                    let max_left = n_remaining.saturating_sub(new3x0 + new200 + new100);

                    let new50 = match n50 {
                        Some(n50) if multiple_given => cmp::min(n_remaining, n50),
                        Some(n50) => cmp::min(max_left, n50),
                        None => max_left,
                    };

                    let (new320, new300) = match (n320, n300) {
                        (Some(n320), Some(n300)) => {
                            (cmp::min(n_remaining, n320), cmp::min(n_remaining, n300))
                        }
                        (Some(n320), None) => (
                            cmp::min(n320, n_remaining),
                            new3x0 - cmp::min(n320, n_remaining),
                        ),
                        (None, Some(n300)) => (
                            new3x0 - cmp::min(n300, n_remaining),
                            cmp::min(n300, n_remaining),
                        ),
                        (None, None) if best_case => (new3x0, 0),
                        (None, None) => (0, new3x0),
                    };

                    let curr_acc = accuracy(new320, new300, new200, new100, new50, misses);
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
                    let n = cmp::min(best_state.n320 + best_state.n300, best_state.n50 / 4);

                    let removed320 = cmp::min(best_state.n320, n);
                    let removed300 = n - removed320;

                    best_state.n320 -= removed320;
                    best_state.n300 -= removed300;

                    n
                } else if n320.is_none() {
                    let n = cmp::min(best_state.n320, best_state.n50 / 4);
                    best_state.n320 -= n;

                    n
                } else if n300.is_none() {
                    let n = cmp::min(best_state.n300, best_state.n50 / 4);
                    best_state.n300 -= n;

                    n
                } else {
                    0
                };

                best_state.n100 += 5 * n;
                best_state.n50 -= 4 * n;
            }
        } else if n320.is_none() && n200.is_none() && n100.is_none() {
            let n = cmp::min(best_state.n320, best_state.n100);
            best_state.n320 -= n;
            best_state.n200 += 2 * n;
            best_state.n100 -= n;
        }

        best_state
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn mania_hitresults(
            acc in 0.0..=1.0,
            n320 in prop::option::weighted(0.10, 0_u32..=N_OBJECTS + 10),
            n300 in prop::option::weighted(0.10, 0_u32..=N_OBJECTS + 10),
            n200 in prop::option::weighted(0.10, 0_u32..=N_OBJECTS + 10),
            n100 in prop::option::weighted(0.10, 0_u32..=N_OBJECTS + 10),
            n50 in prop::option::weighted(0.10, 0_u32..=N_OBJECTS + 10),
            n_misses in prop::option::weighted(0.15, 0_u32..=N_OBJECTS + 10),
            best_case in prop::bool::ANY,
        ) {
            let priority = if best_case {
                HitResultPriority::BestCase
            } else {
                HitResultPriority::WorstCase
            };

            let mut state = ManiaPerformance::from(attrs())
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

            if let Some(misses) = n_misses {
                state = state.misses(misses);
            }

            let state = state.generate_state();

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

            assert_eq!(state, expected);
        }
    }

    #[test]
    fn hitresults_n320_misses_best() {
        let state = ManiaPerformance::from(attrs())
            .n320(500)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state();

        let expected = ManiaScoreState {
            n320: 500,
            n300: 92,
            n200: 0,
            n100: 0,
            n50: 0,
            misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn hitresults_n100_n50_misses_worst() {
        let state = ManiaPerformance::from(attrs())
            .n100(200)
            .n50(50)
            .misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_state();

        let expected = ManiaScoreState {
            n320: 0,
            n300: 0,
            n200: 342,
            n100: 200,
            n50: 50,
            misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn create() {
        let mut map = beatmap();
        let converted = map.unchecked_as_converted();

        let _ = ManiaPerformance::new(ManiaDifficultyAttributes::default());
        let _ = ManiaPerformance::new(ManiaPerformanceAttributes::default());
        let _ = ManiaPerformance::new(&converted);
        let _ = ManiaPerformance::new(converted.as_owned());

        let _ = ManiaPerformance::try_new(ManiaDifficultyAttributes::default()).unwrap();
        let _ = ManiaPerformance::try_new(ManiaPerformanceAttributes::default()).unwrap();
        let _ = ManiaPerformance::try_new(DifficultyAttributes::Mania(
            ManiaDifficultyAttributes::default(),
        ))
        .unwrap();
        let _ = ManiaPerformance::try_new(PerformanceAttributes::Mania(
            ManiaPerformanceAttributes::default(),
        ))
        .unwrap();
        let _ = ManiaPerformance::try_new(&converted).unwrap();
        let _ = ManiaPerformance::try_new(converted.as_owned()).unwrap();

        let _ = ManiaPerformance::from(ManiaDifficultyAttributes::default());
        let _ = ManiaPerformance::from(ManiaPerformanceAttributes::default());
        let _ = ManiaPerformance::from(&converted);
        let _ = ManiaPerformance::from(converted);

        let _ = ManiaDifficultyAttributes::default().performance();
        let _ = ManiaPerformanceAttributes::default().performance();

        map.mode = GameMode::Osu;
        let converted = map.unchecked_as_converted::<Osu>();

        assert!(ManiaPerformance::try_new(OsuDifficultyAttributes::default()).is_none());
        assert!(ManiaPerformance::try_new(OsuPerformanceAttributes::default()).is_none());
        assert!(ManiaPerformance::try_new(DifficultyAttributes::Osu(
            OsuDifficultyAttributes::default()
        ))
        .is_none());
        assert!(ManiaPerformance::try_new(PerformanceAttributes::Osu(
            OsuPerformanceAttributes::default()
        ))
        .is_none());
        assert!(ManiaPerformance::try_new(&converted).is_none());
        assert!(ManiaPerformance::try_new(converted).is_none());
    }
}
