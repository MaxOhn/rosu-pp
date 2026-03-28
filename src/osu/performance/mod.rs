use std::{borrow::Cow, cmp};

use rosu_map::section::general::GameMode;

use self::calculator::OsuPerformanceCalculator;

pub use self::{calculator::PERFORMANCE_BASE_MULTIPLIER, inspect::InspectOsuPerformance};

use crate::{
    Beatmap,
    any::{
        Difficulty, HitResultGenerator, HitResultPriority, InspectablePerformance,
        IntoModePerformance, IntoPerformance, Performance, hitresult_generator::Fast,
    },
    catch::CatchPerformance,
    mania::ManiaPerformance,
    model::{mode::ConvertError, mods::GameMods},
    osu::OsuHitResults,
    taiko::TaikoPerformance,
    util::map_or_attrs::MapOrAttrs,
};

use super::{
    Osu,
    attributes::OsuPerformanceAttributes,
    score_state::{OsuScoreOrigin, OsuScoreState},
};

mod calculator;
pub mod gradual;
mod hitresult_generator;
mod inspect;

/// Performance calculator on osu!standard maps.
#[derive(Clone, Debug)]
#[must_use]
pub struct OsuPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, Osu>,
    pub(crate) difficulty: Difficulty,
    pub(crate) acc: Option<f64>,
    pub(crate) combo: Option<u32>,
    pub(crate) large_tick_hits: Option<u32>,
    pub(crate) small_tick_hits: Option<u32>,
    pub(crate) slider_end_hits: Option<u32>,
    pub(crate) n300: Option<u32>,
    pub(crate) n100: Option<u32>,
    pub(crate) n50: Option<u32>,
    pub(crate) misses: Option<u32>,
    pub(crate) legacy_total_score: Option<u32>,
    pub(crate) hitresult_priority: HitResultPriority,
    pub(crate) hitresult_generator: Option<fn(InspectOsuPerformance<'_>) -> OsuHitResults>,
}

// Manual implementation because of the `hitresult_generator` function pointer
impl PartialEq for OsuPerformance<'_> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            map_or_attrs,
            difficulty,
            acc,
            combo,
            large_tick_hits,
            small_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
            hitresult_priority,
            hitresult_generator: _,
            legacy_total_score,
        } = self;

        map_or_attrs == &other.map_or_attrs
            && difficulty == &other.difficulty
            && acc == &other.acc
            && combo == &other.combo
            && large_tick_hits == &other.large_tick_hits
            && small_tick_hits == &other.small_tick_hits
            && slider_end_hits == &other.slider_end_hits
            && n300 == &other.n300
            && n100 == &other.n100
            && n50 == &other.n50
            && misses == &other.misses
            && hitresult_priority == &other.hitresult_priority
            && legacy_total_score == &other.legacy_total_score
    }
}

impl<'map> OsuPerformance<'map> {
    /// Create a new performance calculator for osu! maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`OsuDifficultyAttributes`]
    ///   or [`OsuPerformanceAttributes`])
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
    /// [`OsuDifficultyAttributes`]: crate::osu::OsuDifficultyAttributes
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, Osu>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu! maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu! i.e.
    /// a [`DifficultyAttributes`] or [`PerformanceAttributes`] of a different
    /// mode.
    ///
    /// See [`OsuPerformance::new`] for more information.
    ///
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::Osu(calc) = map_or_attrs.into_performance() {
            Some(calc)
        } else {
            None
        }
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// Returns `Err(self)` if no beatmap is contained, i.e. if this
    /// [`OsuPerformance`] was created through attributes or
    /// [`OsuPerformance::generate_state`] was called.
    ///
    /// If the given mode should be ignored in case of an error, use
    /// [`mode_or_ignore`] instead.
    ///
    /// [`mode_or_ignore`]: Self::mode_or_ignore
    #[expect(
        clippy::result_large_err,
        reason = "Ok-variant is still larger in size"
    )]
    pub fn try_mode(self, mode: GameMode) -> Result<Performance<'map>, Self> {
        match mode {
            GameMode::Osu => Ok(Performance::Osu(self)),
            GameMode::Taiko => TaikoPerformance::try_from(self).map(Performance::Taiko),
            GameMode::Catch => CatchPerformance::try_from(self).map(Performance::Catch),
            GameMode::Mania => ManiaPerformance::try_from(self).map(Performance::Mania),
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

    /// Specify the priority of hitresults.
    ///
    /// `HitResultPriority::BestCase` sacrifices 300s and n100s to reduce n50s.
    /// `HitResultPriority::WorstCase` does the opposite.
    pub const fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = priority;

        self
    }

    /// Specify how hitresults should be generated.
    pub fn hitresult_generator<H: HitResultGenerator<Osu>>(self) -> OsuPerformance<'map> {
        OsuPerformance {
            map_or_attrs: self.map_or_attrs,
            difficulty: self.difficulty,
            acc: self.acc,
            combo: self.combo,
            large_tick_hits: self.large_tick_hits,
            small_tick_hits: self.small_tick_hits,
            slider_end_hits: self.slider_end_hits,
            n300: self.n300,
            n100: self.n100,
            n50: self.n50,
            misses: self.misses,
            hitresult_priority: self.hitresult_priority,
            hitresult_generator: Some(H::generate_hitresults),
            legacy_total_score: self.legacy_total_score,
        }
    }

    /// Whether the calculated attributes belong to an osu!lazer or osu!stable
    /// score.
    ///
    /// Defaults to `true`.
    ///
    /// This affects internal accuracy calculation because lazer considers
    /// slider heads for accuracy whereas stable does not.
    pub fn lazer(mut self, lazer: bool) -> Self {
        self.difficulty = self.difficulty.lazer(lazer);

        self
    }

    /// Specify the amount of "large tick" hits.
    ///
    /// The meaning depends on the kind of score:
    /// - if set on osu!stable, this value is irrelevant and can be `0`
    /// - if set on osu!lazer *with* slider accuracy, this value is the amount
    ///   of hit slider ticks and repeats
    /// - if set on osu!lazer *without* slider accuracy, this value is the
    ///   amount of hit slider heads, ticks, and repeats
    pub const fn large_tick_hits(mut self, large_tick_hits: u32) -> Self {
        self.large_tick_hits = Some(large_tick_hits);

        self
    }

    /// Specify the amount of "small tick" hits.
    ///
    /// Only relevant for osu!lazer scores without slider accuracy. In that
    /// case, this value is the amount of slider tail hits.
    pub const fn small_tick_hits(mut self, small_tick_hits: u32) -> Self {
        self.small_tick_hits = Some(small_tick_hits);

        self
    }

    /// Specify the amount of hit slider ends.
    ///
    /// Only relevant for osu!lazer scores with slider accuracy.
    pub const fn slider_end_hits(mut self, slider_end_hits: u32) -> Self {
        self.slider_end_hits = Some(slider_end_hits);

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
    pub const fn misses(mut self, n_misses: u32) -> Self {
        self.misses = Some(n_misses);

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
    /// instead of using [`OsuPerformance`] multiple times with different
    /// `passed_objects`, you should use [`OsuGradualPerformance`].
    ///
    /// [`OsuGradualPerformance`]: crate::osu::OsuGradualPerformance
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

    /// Override a beatmap's set AR.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn ar(mut self, ar: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.ar(ar, with_mods);

        self
    }

    /// Override a beatmap's set CS.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn cs(mut self, cs: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.cs(cs, with_mods);

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

    /// Specify the legacy total score.
    pub const fn legacy_total_score(mut self, legacy_total_score: u32) -> Self {
        self.legacy_total_score = Some(legacy_total_score);

        self
    }

    /// Provide parameters through an [`OsuScoreState`].
    pub const fn state(mut self, state: OsuScoreState) -> Self {
        let OsuScoreState {
            max_combo,
            hitresults,
            legacy_total_score,
        } = state;

        self.combo = Some(max_combo);
        self.legacy_total_score = legacy_total_score;

        self.hitresults(hitresults)
    }

    /// Provide parameters through [`OsuHitResults`].
    #[expect(clippy::needless_pass_by_value, reason = "more convenient")]
    pub const fn hitresults(mut self, hitresults: OsuHitResults) -> Self {
        let OsuHitResults {
            large_tick_hits,
            small_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
        } = hitresults;

        self.large_tick_hits = Some(large_tick_hits);
        self.small_tick_hits = Some(small_tick_hits);
        self.slider_end_hits = Some(slider_end_hits);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.n50 = Some(n50);
        self.misses = Some(misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Create the [`OsuScoreState`] that will be used for performance calculation.
    pub fn generate_state(&mut self) -> Result<OsuScoreState, ConvertError> {
        self.map_or_attrs.insert_attrs(&self.difficulty)?;

        // SAFETY: We just calculated and inserted the attributes.
        let attrs = unsafe { self.map_or_attrs.get_attrs() };

        let inspect = Osu::inspect_performance(self, attrs);

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();

        let mut hitresults = match self.hitresult_generator {
            Some(generator) => generator(inspect),
            None => <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect),
        };

        let remain = total_hits.saturating_sub(hitresults.total_hits());

        match self.hitresult_priority {
            HitResultPriority::BestCase => match (self.n300, self.n100, self.n50) {
                (None, ..) => hitresults.n300 += remain,
                (_, None, _) => hitresults.n100 += remain,
                (.., None) => hitresults.n50 += remain,
                _ => hitresults.n300 += remain,
            },
            HitResultPriority::WorstCase => match (self.n50, self.n100, self.n300) {
                (None, ..) => hitresults.n50 += remain,
                (_, None, _) => hitresults.n100 += remain,
                (.., None) => hitresults.n300 += remain,
                _ => hitresults.n50 += remain,
            },
        }

        let max_possible_combo = attrs.max_combo.saturating_sub(misses);

        let max_combo = self.combo.map_or(max_possible_combo, |combo| {
            cmp::min(combo, max_possible_combo)
        });

        self.combo = Some(max_combo);

        let OsuHitResults {
            large_tick_hits,
            small_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
        } = &hitresults;

        self.slider_end_hits = Some(*slider_end_hits);
        self.large_tick_hits = Some(*large_tick_hits);
        self.small_tick_hits = Some(*small_tick_hits);
        self.n300 = Some(*n300);
        self.n100 = Some(*n100);
        self.n50 = Some(*n50);
        self.misses = Some(*misses);

        Ok(OsuScoreState {
            max_combo,
            hitresults,
            legacy_total_score: self.legacy_total_score,
        })
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> Result<OsuPerformanceAttributes, ConvertError> {
        let state = self.generate_state()?;

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Attrs(attrs) => attrs,
            MapOrAttrs::Map(ref map) => self.difficulty.calculate_for_mode::<Osu>(map)?,
        };

        let mods = self.difficulty.get_mods();
        let lazer = self.difficulty.get_lazer();
        let using_classic_slider_acc = mods.no_slider_head_acc(lazer);

        let origin = match (lazer, using_classic_slider_acc) {
            (false, _) => OsuScoreOrigin::Stable,
            (true, false) => OsuScoreOrigin::WithSliderAcc {
                max_large_ticks: attrs.n_large_ticks,
                max_slider_ends: attrs.n_sliders,
            },
            (true, true) => OsuScoreOrigin::WithoutSliderAcc {
                max_large_ticks: attrs.n_sliders + attrs.n_large_ticks,
                max_small_ticks: attrs.n_sliders,
            },
        };

        let acc = state.hitresults.accuracy(origin);

        let inner =
            OsuPerformanceCalculator::new(attrs, mods, acc, state, using_classic_slider_acc);

        Ok(inner.calculate())
    }

    pub(crate) const fn from_map_or_attrs(map_or_attrs: MapOrAttrs<'map, Osu>) -> Self {
        Self {
            map_or_attrs,
            difficulty: Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: None,
            misses: None,
            hitresult_priority: HitResultPriority::DEFAULT,
            hitresult_generator: None,
            legacy_total_score: None,
        }
    }

    #[expect(
        clippy::result_large_err,
        reason = "Result type serves more as 'Either'"
    )]
    pub(crate) fn try_convert_map(
        map_or_attrs: MapOrAttrs<'map, Osu>,
        mode: GameMode,
        mods: &GameMods,
    ) -> Result<Cow<'map, Beatmap>, MapOrAttrs<'map, Osu>> {
        let MapOrAttrs::Map(map) = map_or_attrs else {
            return Err(map_or_attrs);
        };

        match map {
            Cow::Borrowed(map) => match map.convert_ref(mode, mods) {
                Ok(map) => Ok(map),
                Err(_) => Err(MapOrAttrs::Map(Cow::Borrowed(map))),
            },
            Cow::Owned(mut map) => {
                if map.convert_mut(mode, mods).is_err() {
                    return Err(MapOrAttrs::Map(Cow::Owned(map)));
                }

                Ok(Cow::Owned(map))
            }
        }
    }
}

impl<'map, T: IntoModePerformance<'map, Osu>> From<T> for OsuPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use crate::{
        Beatmap,
        any::{DifficultyAttributes, PerformanceAttributes},
        osu::OsuDifficultyAttributes,
        taiko::{TaikoDifficultyAttributes, TaikoPerformanceAttributes},
    };

    use super::*;

    static ATTRS: OnceLock<OsuDifficultyAttributes> = OnceLock::new();

    const N_OBJECTS: u32 = 601;
    const N_SLIDERS: u32 = 293;
    const N_SLIDER_TICKS: u32 = 15;

    fn beatmap() -> Beatmap {
        Beatmap::from_path("./resources/2785319.osu").unwrap()
    }

    fn attrs() -> OsuDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let map = beatmap();
                let attrs = Difficulty::new().calculate_for_mode::<Osu>(&map).unwrap();

                assert_eq!(
                    (attrs.n_circles, attrs.n_sliders, attrs.n_spinners),
                    (307, 293, 1)
                );
                assert_eq!(
                    attrs.n_circles + attrs.n_sliders + attrs.n_spinners,
                    N_OBJECTS,
                );
                assert_eq!(attrs.n_sliders, N_SLIDERS);
                assert_eq!(attrs.n_large_ticks, N_SLIDER_TICKS);

                attrs
            })
            .to_owned()
    }

    #[test]
    fn hitresults_n300_n100_misses_best() {
        let state = OsuPerformance::from(attrs())
            .combo(500)
            .lazer(true)
            .n300(300)
            .n100(20)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state()
            .unwrap();

        let expected = OsuHitResults {
            large_tick_hits: N_SLIDER_TICKS,
            small_tick_hits: 0,
            slider_end_hits: N_SLIDERS,
            n300: 300,
            n100: 20,
            n50: 279,
            misses: 2,
        };

        assert_eq!(state.hitresults, expected);
    }

    #[test]
    fn hitresults_n300_n50_misses_best() {
        let state = OsuPerformance::from(attrs())
            .lazer(false)
            .combo(500)
            .n300(300)
            .n50(10)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state()
            .unwrap();

        let expected = OsuHitResults {
            large_tick_hits: 0,
            small_tick_hits: 0,
            slider_end_hits: 0,
            n300: 300,
            n100: 289,
            n50: 10,
            misses: 2,
        };

        assert_eq!(state.hitresults, expected);
    }

    #[test]
    fn hitresults_n50_misses_worst() {
        let state = OsuPerformance::from(attrs())
            .lazer(true)
            .combo(500)
            .n50(10)
            .misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_state()
            .unwrap();

        let expected = OsuHitResults {
            large_tick_hits: N_SLIDER_TICKS,
            small_tick_hits: 0,
            slider_end_hits: N_SLIDERS,
            n300: 0,
            n100: 589,
            n50: 10,
            misses: 2,
        };

        assert_eq!(state.hitresults, expected);
    }

    #[test]
    fn hitresults_n300_n100_n50_misses_worst() {
        let state = OsuPerformance::from(attrs())
            .lazer(false)
            .combo(500)
            .n300(300)
            .n100(50)
            .n50(10)
            .misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_state()
            .unwrap();

        let expected = OsuHitResults {
            large_tick_hits: 0,
            small_tick_hits: 0,
            slider_end_hits: 0,
            n300: 300,
            n100: 50,
            n50: 249,
            misses: 2,
        };

        assert_eq!(state.hitresults, expected);
    }

    #[test]
    fn create() {
        let mut map = beatmap();

        let _ = OsuPerformance::new(OsuDifficultyAttributes::default());
        let _ = OsuPerformance::new(OsuPerformanceAttributes::default());
        let _ = OsuPerformance::new(&map);
        let _ = OsuPerformance::new(map.clone());

        let _ = OsuPerformance::try_new(OsuDifficultyAttributes::default()).unwrap();
        let _ = OsuPerformance::try_new(OsuPerformanceAttributes::default()).unwrap();
        let _ =
            OsuPerformance::try_new(DifficultyAttributes::Osu(OsuDifficultyAttributes::default()))
                .unwrap();
        let _ = OsuPerformance::try_new(PerformanceAttributes::Osu(
            OsuPerformanceAttributes::default(),
        ))
        .unwrap();
        let _ = OsuPerformance::try_new(&map).unwrap();
        let _ = OsuPerformance::try_new(map.clone()).unwrap();

        let _ = OsuPerformance::from(OsuDifficultyAttributes::default());
        let _ = OsuPerformance::from(OsuPerformanceAttributes::default());
        let _ = OsuPerformance::from(&map);
        let _ = OsuPerformance::from(map.clone());

        let _ = OsuDifficultyAttributes::default().performance();
        let _ = OsuPerformanceAttributes::default().performance();

        map.convert_mut(GameMode::Taiko, &GameMods::default())
            .unwrap();

        assert!(OsuPerformance::try_new(TaikoDifficultyAttributes::default()).is_none());
        assert!(OsuPerformance::try_new(TaikoPerformanceAttributes::default()).is_none());
        assert!(
            OsuPerformance::try_new(DifficultyAttributes::Taiko(
                TaikoDifficultyAttributes::default()
            ))
            .is_none()
        );
        assert!(
            OsuPerformance::try_new(PerformanceAttributes::Taiko(
                TaikoPerformanceAttributes::default()
            ))
            .is_none()
        );
        assert!(OsuPerformance::try_new(&map).is_none());
        assert!(OsuPerformance::try_new(map).is_none());
    }
}
