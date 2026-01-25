use std::cmp;

use rosu_map::section::general::GameMode;

use self::calculator::TaikoPerformanceCalculator;

pub use self::inspect::InspectTaikoPerformance;

use crate::{
    Performance,
    any::{
        Difficulty, HitResultGenerator, HitResultPriority, InspectablePerformance,
        IntoModePerformance, IntoPerformance, hitresult_generator::Closest,
    },
    model::{mode::ConvertError, mods::GameMods},
    osu::OsuPerformance,
    taiko::score_state::TaikoHitResults,
    util::map_or_attrs::MapOrAttrs,
};

use super::{Taiko, attributes::TaikoPerformanceAttributes, score_state::TaikoScoreState};

mod calculator;
pub mod gradual;
mod hitresult_generator;
mod inspect;

/// Performance calculator on osu!taiko maps.
#[derive(Clone, Debug)]
#[must_use]
pub struct TaikoPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, Taiko>,
    difficulty: Difficulty,
    combo: Option<u32>,
    acc: Option<f64>,
    n300: Option<u32>,
    n100: Option<u32>,
    misses: Option<u32>,
    hitresult_priority: HitResultPriority,
    hitresult_generator: Option<fn(InspectTaikoPerformance<'_>) -> TaikoHitResults>,
}

// Manual implementation because of the `hitresult_generator` function pointer
impl PartialEq for TaikoPerformance<'_> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            map_or_attrs,
            difficulty,
            combo,
            acc,
            n300,
            n100,
            misses,
            hitresult_priority,
            hitresult_generator: _,
        } = self;

        map_or_attrs == &other.map_or_attrs
            && difficulty == &other.difficulty
            && combo == &other.combo
            && acc == &other.acc
            && n300 == &other.n300
            && n100 == &other.n100
            && misses == &other.misses
            && hitresult_priority == &other.hitresult_priority
    }
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

    /// Specify the priority of hitresults.
    pub const fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = priority;

        self
    }

    /// Specify how hitresults should be generated.
    pub fn hitresult_generator<H: HitResultGenerator<Taiko>>(self) -> TaikoPerformance<'map> {
        TaikoPerformance {
            map_or_attrs: self.map_or_attrs,
            difficulty: self.difficulty,
            acc: self.acc,
            combo: self.combo,
            n300: self.n300,
            n100: self.n100,
            misses: self.misses,
            hitresult_priority: self.hitresult_priority,
            hitresult_generator: Some(H::generate_hitresults),
        }
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
            hitresults,
        } = state;

        self.combo = Some(max_combo);

        self.hitresults(hitresults)
    }

    /// Provide parameters through [`TaikoHitResults`].
    pub const fn hitresults(mut self, hitresults: TaikoHitResults) -> Self {
        let TaikoHitResults { n300, n100, misses } = hitresults;

        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.misses = Some(misses);

        self
    }

    /// Create the [`TaikoScoreState`] that will be used for performance calculation.
    pub fn generate_state(&mut self) -> Result<TaikoScoreState, ConvertError> {
        self.map_or_attrs.insert_attrs(&self.difficulty)?;

        // SAFETY: We just calculated and inserted the attributes.
        let attrs = unsafe { self.map_or_attrs.get_attrs() };

        let inspect = Taiko::inspect_performance(self, attrs);

        let max_combo = inspect.max_combo();
        let misses = inspect.misses();

        let hitresults = match self.hitresult_generator {
            Some(generator) => generator(inspect),
            // Closest is still pretty quick for taiko so it should be fine
            // to default to it rather than Fast
            None => <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect),
        };

        let max_possible_combo = max_combo.saturating_sub(misses);

        let max_combo = self.combo.map_or(max_possible_combo, |combo| {
            cmp::min(combo, max_possible_combo)
        });

        self.combo = Some(max_combo);

        let TaikoHitResults { n300, n100, misses } = hitresults;

        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.misses = Some(misses);

        Ok(TaikoScoreState {
            max_combo,
            hitresults,
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
            hitresult_generator: None,
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
            hitresult_generator: _,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            difficulty,
            combo,
            acc,
            n300,
            n100,
            misses,
            hitresult_priority,
            hitresult_generator: None,
        })
    }
}

impl<'map, T: IntoModePerformance<'map, Taiko>> From<T> for TaikoPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use rosu_map::section::general::GameMode;

    use crate::{
        Beatmap,
        any::{DifficultyAttributes, PerformanceAttributes},
        osu::{OsuDifficultyAttributes, OsuPerformanceAttributes},
        taiko::TaikoDifficultyAttributes,
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

    #[test]
    fn hitresults_n300_misses_best() {
        let state = TaikoPerformance::from(attrs())
            .combo(100)
            .n300(150)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state()
            .unwrap();

        let expected = TaikoHitResults {
            n300: 150,
            n100: 137,
            misses: 2,
        };

        assert_eq!(state.hitresults, expected);
    }

    #[test]
    fn hitresults_misses_best() {
        let state = TaikoPerformance::from(attrs())
            .combo(100)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state()
            .unwrap();

        let expected = TaikoHitResults {
            n300: 287,
            n100: 0,
            misses: 2,
        };

        assert_eq!(state.hitresults, expected);
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

        assert!(
            map.convert_mut(GameMode::Osu, &GameMods::default())
                .is_err()
        );

        assert!(TaikoPerformance::try_new(OsuDifficultyAttributes::default()).is_none());
        assert!(TaikoPerformance::try_new(OsuPerformanceAttributes::default()).is_none());
        assert!(
            TaikoPerformance::try_new(
                DifficultyAttributes::Osu(OsuDifficultyAttributes::default())
            )
            .is_none()
        );
        assert!(
            TaikoPerformance::try_new(PerformanceAttributes::Osu(
                OsuPerformanceAttributes::default()
            ))
            .is_none()
        );
    }
}
