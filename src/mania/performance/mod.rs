use rosu_map::section::general::GameMode;

use self::calculator::ManiaPerformanceCalculator;

pub use self::inspect::InspectManiaPerformance;

use crate::{
    Performance,
    any::{
        Difficulty, HitResultGenerator, HitResultPriority, InspectablePerformance,
        IntoModePerformance, IntoPerformance, hitresult_generator::Fast,
    },
    mania::ManiaHitResults,
    model::{mode::ConvertError, mods::GameMods},
    osu::OsuPerformance,
    util::map_or_attrs::MapOrAttrs,
};

use super::{Mania, attributes::ManiaPerformanceAttributes, score_state::ManiaScoreState};

mod calculator;
pub mod gradual;
mod hitresult_generator;
mod inspect;

/// Performance calculator on osu!mania maps.
#[derive(Clone, Debug)]
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
    hitresult_generator: Option<fn(InspectManiaPerformance<'_>) -> ManiaHitResults>,
}

// Manual implementation because of the `hitresult_generator` function pointer
impl PartialEq for ManiaPerformance<'_> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            map_or_attrs,
            difficulty,
            n320,
            n300,
            n200,
            n100,
            n50,
            misses,
            acc,
            hitresult_priority,
            hitresult_generator: _,
        } = self;

        map_or_attrs == &other.map_or_attrs
            && difficulty == &other.difficulty
            && n320 == &other.n320
            && n300 == &other.n300
            && n200 == &other.n200
            && n100 == &other.n100
            && n50 == &other.n50
            && misses == &other.misses
            && acc == &other.acc
            && hitresult_priority == &other.hitresult_priority
    }
}

impl<'map> ManiaPerformance<'map> {
    /// Create a new performance calculator for osu!mania maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`ManiaDifficultyAttributes`]
    ///   or [`ManiaPerformanceAttributes`])
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
    /// [`ManiaDifficultyAttributes`]: crate::mania::ManiaDifficultyAttributes
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, Mania>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu!mania maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu!mania i.e.
    /// a [`DifficultyAttributes`] or [`PerformanceAttributes`] of a different
    /// mode.
    ///
    /// See [`ManiaPerformance::new`] for more information.
    ///
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::Mania(calc) = map_or_attrs.into_performance() {
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

    /// Use the specified settings of the given [`Difficulty`].
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
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

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Specify the priority of hitresults.
    pub const fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = priority;

        self
    }

    /// Specify how hitresults should be generated.
    pub fn hitresult_generator<H: HitResultGenerator<Mania>>(self) -> ManiaPerformance<'map> {
        ManiaPerformance {
            map_or_attrs: self.map_or_attrs,
            difficulty: self.difficulty,
            n320: self.n320,
            n300: self.n300,
            n200: self.n200,
            n100: self.n100,
            n50: self.n50,
            misses: self.misses,
            acc: self.acc,
            hitresult_priority: self.hitresult_priority,
            hitresult_generator: Some(H::generate_hitresults),
        }
    }

    /// Whether the calculated attributes belong to an osu!lazer or osu!stable
    /// score.
    ///
    /// Defaults to `true`.
    ///
    /// This affects internal hitresult generation because lazer (without `CL`
    /// mod) gives two hitresults per hold note whereas stable only gives one.
    /// It also affect accuracy calculation because stable makes no difference
    /// between perfect (n320) and great (n300) hitresults but lazer (without
    /// `CL` mod) rewards slightly more for perfect hitresults.
    pub fn lazer(mut self, lazer: bool) -> Self {
        self.difficulty = self.difficulty.lazer(lazer);

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
    #[expect(clippy::needless_pass_by_value, reason = "more sensible")]
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
    pub fn generate_state(&mut self) -> Result<ManiaScoreState, ConvertError> {
        self.map_or_attrs.insert_attrs(&self.difficulty)?;

        // SAFETY: We just calculated and inserted the attributes.
        let attrs = unsafe { self.map_or_attrs.get_attrs() };

        let inspect = Mania::inspect_performance(self, attrs);

        let total_hits = inspect.total_hits();

        let mut hitresults = match self.hitresult_generator {
            Some(generator) => generator(inspect),
            // TODO: use Statistical(?)
            None => <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect),
        };

        let remain = total_hits.saturating_sub(hitresults.total_hits());

        match self.hitresult_priority {
            HitResultPriority::BestCase => {
                match (self.n320, self.n300, self.n200, self.n100, self.n50) {
                    (None, ..) => hitresults.n320 += remain,
                    (_, None, ..) => hitresults.n300 += remain,
                    (_, _, None, ..) => hitresults.n200 += remain,
                    (.., None, _) => hitresults.n100 += remain,
                    _ => hitresults.n50 += remain,
                }
            }
            HitResultPriority::WorstCase => {
                match (self.n50, self.n100, self.n200, self.n300, self.n320) {
                    (None, ..) => hitresults.n50 += remain,
                    (_, None, ..) => hitresults.n100 += remain,
                    (_, _, None, ..) => hitresults.n200 += remain,
                    (.., None, _) => hitresults.n300 += remain,
                    _ => hitresults.n320 += remain,
                }
            }
        }

        let ManiaHitResults {
            n320,
            n300,
            n200,
            n100,
            n50,
            misses,
        } = &hitresults;

        self.n320 = Some(*n320);
        self.n300 = Some(*n300);
        self.n200 = Some(*n200);
        self.n100 = Some(*n100);
        self.n50 = Some(*n50);
        self.misses = Some(*misses);

        Ok(hitresults)
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> Result<ManiaPerformanceAttributes, ConvertError> {
        let state = self.generate_state()?;

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Attrs(attrs) => attrs,
            MapOrAttrs::Map(ref map) => self.difficulty.calculate_for_mode::<Mania>(map)?,
        };

        Ok(ManiaPerformanceCalculator::new(attrs, self.difficulty.get_mods(), state).calculate())
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
            hitresult_generator: None,
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
        let mods = osu.difficulty.get_mods();

        let map = match OsuPerformance::try_convert_map(osu.map_or_attrs, GameMode::Mania, mods) {
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
            combo: _,
            large_tick_hits: _,
            small_tick_hits: _,
            slider_end_hits: _,
            n300,
            n100,
            n50,
            misses,
            hitresult_priority,
            hitresult_generator: _,
            legacy_total_score: _,
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
            hitresult_generator: None,
        })
    }
}

impl<'map, T: IntoModePerformance<'map, Mania>> From<T> for ManiaPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use rosu_map::section::general::GameMode;
    use rosu_mods::{GameMod, generated_mods::ClassicMania};

    use crate::{
        Beatmap,
        any::{DifficultyAttributes, PerformanceAttributes},
        mania::ManiaDifficultyAttributes,
        osu::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    };

    use super::*;

    static ATTRS: OnceLock<ManiaDifficultyAttributes> = OnceLock::new();

    const N_OBJECTS: u32 = 594;
    const N_HOLD_NOTES: u32 = 121;

    fn beatmap() -> Beatmap {
        Beatmap::from_path("./resources/1638954.osu").unwrap()
    }

    fn attrs() -> ManiaDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let map = beatmap();
                let attrs = Difficulty::new().calculate_for_mode::<Mania>(&map).unwrap();

                assert_eq!(N_OBJECTS, map.hit_objects.len() as u32);
                assert_eq!(
                    N_HOLD_NOTES,
                    map.hit_objects.iter().filter(|h| !h.is_circle()).count() as u32
                );

                attrs
            })
            .to_owned()
    }

    /// Creates a [`rosu_mods::GameMods`] instance and inserts `CL` if `classic`
    /// is true.
    fn mods(classic: bool) -> rosu_mods::GameMods {
        if classic {
            let mut mods = rosu_mods::GameMods::new();
            mods.insert(GameMod::ClassicMania(ClassicMania::default()));

            mods
        } else {
            rosu_mods::GameMods::new()
        }
    }

    #[test]
    fn hitresults_n320_misses_best() {
        let classic = true;

        let state = ManiaPerformance::from(attrs())
            .lazer(!classic)
            .mods(mods(classic))
            .n320(500)
            .misses(2)
            .hitresult_priority(HitResultPriority::BestCase)
            .generate_state()
            .unwrap();

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
        let classic = true;

        let state = ManiaPerformance::from(attrs())
            .lazer(!classic)
            .mods(mods(classic))
            .n100(200)
            .n50(50)
            .misses(2)
            .hitresult_priority(HitResultPriority::WorstCase)
            .generate_state()
            .unwrap();

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

        let _ = ManiaPerformance::new(ManiaDifficultyAttributes::default());
        let _ = ManiaPerformance::new(ManiaPerformanceAttributes::default());
        let _ = ManiaPerformance::new(&map);
        let _ = ManiaPerformance::new(map.clone());

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
        let _ = ManiaPerformance::try_new(&map).unwrap();
        let _ = ManiaPerformance::try_new(map.clone()).unwrap();

        let _ = ManiaPerformance::from(ManiaDifficultyAttributes::default());
        let _ = ManiaPerformance::from(ManiaPerformanceAttributes::default());
        let _ = ManiaPerformance::from(&map);
        let _ = ManiaPerformance::from(map.clone());

        let _ = ManiaDifficultyAttributes::default().performance();
        let _ = ManiaPerformanceAttributes::default().performance();

        assert!(
            map.convert_mut(GameMode::Osu, &GameMods::default())
                .is_err()
        );

        assert!(ManiaPerformance::try_new(OsuDifficultyAttributes::default()).is_none());
        assert!(ManiaPerformance::try_new(OsuPerformanceAttributes::default()).is_none());
        assert!(
            ManiaPerformance::try_new(
                DifficultyAttributes::Osu(OsuDifficultyAttributes::default())
            )
            .is_none()
        );
        assert!(
            ManiaPerformance::try_new(PerformanceAttributes::Osu(
                OsuPerformanceAttributes::default()
            ))
            .is_none()
        );
    }
}
