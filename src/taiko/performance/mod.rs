use std::cmp;

use rosu_map::section::general::GameMode;

use self::calculator::TaikoPerformanceCalculator;

use crate::{
    any::{Difficulty, HitResultPriority, IntoModePerformance, IntoPerformance},
    model::{mode::ConvertError, mods::GameMods},
    osu::OsuPerformance,
    util::map_or_attrs::MapOrAttrs,
    Performance,
};

use super::{attributes::TaikoPerformanceAttributes, score_state::TaikoScoreState, Taiko};

mod calculator;
pub mod gradual;

/// Performance calculator on osu!taiko maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TaikoPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, Taiko>,
    difficulty: Difficulty,
    combo: Option<u32>,
    acc: Option<f64>,
    hitresult_priority: HitResultPriority,
    n300: Option<u32>,
    n100: Option<u32>,
    misses: Option<u32>,
}

impl<'map> TaikoPerformance<'map> {
    /// Create a new performance calculator for osu!taiko maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`TaikoDifficultyAttributes`]
    ///   or [`TaikoPerformanceAttributes`])
    /// - a [`Beatmap`] (by reference or value)
    ///
    /// If a map is given, difficulty attributes will need to be calculated
    /// internally which is a costly operation. Hence, passing attributes
    /// should be prefered.
    ///
    /// However, when passing previously calculated attributes, make sure they
    /// have been calculated for the same map and [`Difficulty`] settings.
    /// Otherwise, the final attributes will be incorrect.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    /// [`TaikoDifficultyAttributes`]: crate::taiko::TaikoDifficultyAttributes
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, Taiko>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu!taiko maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu!taiko i.e.
    /// a [`DifficultyAttributes`] or [`PerformanceAttributes`] of a different
    /// mode.
    ///
    /// See [`TaikoPerformance::new`] for more information.
    ///
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::Taiko(calc) = map_or_attrs.into_performance() {
            Some(calc)
        } else {
            None
        }
    }

    /// Specify mods.
    ///
    /// Accepted types are
    /// - `u32`
    /// - [`rosu_mods::GameModsLegacy`]
    /// - [`rosu_mods::GameMods`]
    /// - [`rosu_mods::GameModsIntermode`]
    /// - [`&rosu_mods::GameModsIntermode`](rosu_mods::GameModsIntermode)
    ///
    /// See <https://github.com/ppy/osu-api/wiki#mods>
    pub fn mods(mut self, mods: impl Into<GameMods>) -> Self {
        self.difficulty = self.difficulty.mods(mods);

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
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Use the specified settings of the given [`Difficulty`].
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects,
    /// instead of using [`TaikoPerformance`] multiple times with different
    /// `passed_objects`, you should use [`TaikoGradualPerformance`].
    ///
    /// [`TaikoGradualPerformance`]: crate::taiko::TaikoGradualPerformance
    pub fn passed_objects(mut self, passed_objects: u32) -> Self {
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

    /// Provide parameters through a [`TaikoScoreState`].
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
    pub fn generate_state(&mut self) -> Result<TaikoScoreState, ConvertError> {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.difficulty.calculate_for_mode::<Taiko>(map)?;

                self.map_or_attrs.insert_attrs(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let max_combo = attrs.max_combo();

        let total_result_count = cmp::min(self.difficulty.get_passed_objects() as u32, max_combo);

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
                        HitResultPriority::BestCase | HitResultPriority::Fastest => {
                            n300 += remaining;
                        }
                        HitResultPriority::WorstCase => n100 += remaining,
                    }
                }
                (Some(_), None) => n100 += total_result_count.saturating_sub(n300 + misses),
                (None, Some(_)) => n300 += total_result_count.saturating_sub(n100 + misses),
                (None, None) => {
                    let target_total = acc * f64::from(2 * total_result_count);

                    if let HitResultPriority::Fastest = priority {
                        n300 = u32::min(
                            n_remaining,
                            (f64::round_ties_even(target_total) as u32).saturating_sub(n_remaining),
                        );
                        n100 = total_result_count.saturating_sub(n300 + misses);
                    } else {
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
            }
        } else {
            let remaining = total_result_count.saturating_sub(n300 + n100 + misses);

            match priority {
                HitResultPriority::BestCase | HitResultPriority::Fastest => {
                    match (self.n300, self.n100) {
                        (None, _) => n300 = remaining,
                        (_, None) => n100 = remaining,
                        _ => n300 += remaining,
                    }
                }
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

        self.combo = Some(max_combo);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.misses = Some(misses);

        Ok(TaikoScoreState {
            max_combo,
            n300,
            n100,
            misses,
        })
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> Result<TaikoPerformanceAttributes, ConvertError> {
        let state = self.generate_state()?;

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Attrs(attrs) => attrs,
            MapOrAttrs::Map(ref map) => self.difficulty.calculate_for_mode::<Taiko>(map)?,
        };

        Ok(TaikoPerformanceCalculator::new(attrs, self.difficulty.get_mods(), state).calculate())
    }

    pub(crate) const fn from_map_or_attrs(map_or_attrs: MapOrAttrs<'map, Taiko>) -> Self {
        Self {
            map_or_attrs,
            difficulty: Difficulty::new(),
            combo: None,
            acc: None,
            misses: None,
            n300: None,
            n100: None,
            hitresult_priority: HitResultPriority::DEFAULT,
        }
    }
}

impl<'map> TryFrom<OsuPerformance<'map>> for TaikoPerformance<'map> {
    type Error = OsuPerformance<'map>;

    /// Try to create [`TaikoPerformance`] through [`OsuPerformance`].
    ///
    /// Returns `None` if [`OsuPerformance`] does not contain a beatmap, i.e.
    /// if it was constructed through attributes or
    /// [`OsuPerformance::generate_state`] was called.
    fn try_from(mut osu: OsuPerformance<'map>) -> Result<Self, Self::Error> {
        let mods = osu.difficulty.get_mods();

        let map = match OsuPerformance::try_convert_map(osu.map_or_attrs, GameMode::Taiko, mods) {
            Ok(map) => map,
            Err(map_or_attrs) => {
                osu.map_or_attrs = map_or_attrs;

                return Err(osu);
            }
        };

        let OsuPerformance {
            map_or_attrs: _,
            difficulty,
            acc,
            combo,
            large_tick_hits: _,
            small_tick_hits: _,
            slider_end_hits: _,
            n300,
            n100,
            n50: _,
            misses,
            hitresult_priority,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            difficulty,
            combo,
            acc,
            hitresult_priority,
            n300,
            n100,
            misses,
        })
    }
}

impl<'map, T: IntoModePerformance<'map, Taiko>> From<T> for TaikoPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
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
    use rosu_map::section::general::GameMode;

    use crate::{
        any::{DifficultyAttributes, PerformanceAttributes},
        osu::{OsuDifficultyAttributes, OsuPerformanceAttributes},
        taiko::TaikoDifficultyAttributes,
        Beatmap,
    };

    use super::*;

    static ATTRS: OnceLock<TaikoDifficultyAttributes> = OnceLock::new();

    const MAX_COMBO: u32 = 289;

    fn beatmap() -> Beatmap {
        Beatmap::from_path("./resources/1028484.osu").unwrap()
    }

    fn attrs() -> TaikoDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let map = beatmap();
                let attrs = Difficulty::new().calculate_for_mode::<Taiko>(&map).unwrap();

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

            let first = state.generate_state().unwrap();
            let state = state.generate_state().unwrap();
            assert_eq!(first, state);

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
            .generate_state()
            .unwrap();

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
            .generate_state()
            .unwrap();

        let expected = TaikoScoreState {
            max_combo: 100,
            n300: 287,
            n100: 0,
            misses: 2,
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn create() {
        let mut map = beatmap();

        let _ = TaikoPerformance::new(TaikoDifficultyAttributes::default());
        let _ = TaikoPerformance::new(TaikoPerformanceAttributes::default());
        let _ = TaikoPerformance::new(&map);
        let _ = TaikoPerformance::new(map.clone());

        let _ = TaikoPerformance::try_new(TaikoDifficultyAttributes::default()).unwrap();
        let _ = TaikoPerformance::try_new(TaikoPerformanceAttributes::default()).unwrap();
        let _ = TaikoPerformance::try_new(DifficultyAttributes::Taiko(
            TaikoDifficultyAttributes::default(),
        ))
        .unwrap();
        let _ = TaikoPerformance::try_new(PerformanceAttributes::Taiko(
            TaikoPerformanceAttributes::default(),
        ))
        .unwrap();
        let _ = TaikoPerformance::try_new(&map).unwrap();
        let _ = TaikoPerformance::try_new(map.clone()).unwrap();

        let _ = TaikoPerformance::from(TaikoDifficultyAttributes::default());
        let _ = TaikoPerformance::from(TaikoPerformanceAttributes::default());
        let _ = TaikoPerformance::from(&map);
        let _ = TaikoPerformance::from(map.clone());

        let _ = TaikoDifficultyAttributes::default().performance();
        let _ = TaikoPerformanceAttributes::default().performance();

        assert!(map
            .convert_mut(GameMode::Osu, &GameMods::default())
            .is_err());

        assert!(TaikoPerformance::try_new(OsuDifficultyAttributes::default()).is_none());
        assert!(TaikoPerformance::try_new(OsuPerformanceAttributes::default()).is_none());
        assert!(TaikoPerformance::try_new(DifficultyAttributes::Osu(
            OsuDifficultyAttributes::default()
        ))
        .is_none());
        assert!(TaikoPerformance::try_new(PerformanceAttributes::Osu(
            OsuPerformanceAttributes::default()
        ))
        .is_none());
    }
}
