use rosu_map::section::general::GameMode;

use self::calculator::CatchPerformanceCalculator;

use crate::{
    Performance,
    any::{
        Difficulty, HitResultGenerator, InspectablePerformance, IntoModePerformance,
        IntoPerformance, hitresult_generator::Fast,
    },
    catch::{CatchHitResults, performance::inspect::InspectCatchPerformance},
    model::{mode::ConvertError, mods::GameMods},
    osu::OsuPerformance,
    util::map_or_attrs::MapOrAttrs,
};

use super::{Catch, attributes::CatchPerformanceAttributes, score_state::CatchScoreState};

mod calculator;
pub mod gradual;
mod hitresult_generator;
mod inspect;

/// Performance calculator on osu!catch maps.
#[derive(Clone, Debug)]
#[must_use]
pub struct CatchPerformance<'map> {
    map_or_attrs: MapOrAttrs<'map, Catch>,
    difficulty: Difficulty,
    acc: Option<f64>,
    combo: Option<u32>,
    fruits: Option<u32>,
    droplets: Option<u32>,
    tiny_droplets: Option<u32>,
    tiny_droplet_misses: Option<u32>,
    misses: Option<u32>,
    hitresult_generator: Option<fn(InspectCatchPerformance<'_>) -> CatchHitResults>,
}

// Manual implementation because of the `hitresult_generator` function pointer
impl PartialEq for CatchPerformance<'_> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            map_or_attrs,
            difficulty,
            acc,
            combo,
            fruits,
            droplets,
            tiny_droplets,
            tiny_droplet_misses,
            misses,
            hitresult_generator: _,
        } = self;

        map_or_attrs == &other.map_or_attrs
            && difficulty == &other.difficulty
            && acc == &other.acc
            && combo == &other.combo
            && fruits == &other.fruits
            && droplets == &other.droplets
            && tiny_droplets == &other.tiny_droplets
            && tiny_droplet_misses == &other.tiny_droplet_misses
            && misses == &other.misses
    }
}

impl<'map> CatchPerformance<'map> {
    /// Create a new performance calculator for osu!catch maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`CatchDifficultyAttributes`]
    ///   or [`CatchPerformanceAttributes`])
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
    /// [`CatchDifficultyAttributes`]: crate::catch::CatchDifficultyAttributes
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, Catch>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu!catch maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu!catch i.e.
    /// a [`DifficultyAttributes`] or [`PerformanceAttributes`] of a different
    /// mode.
    ///
    /// See [`CatchPerformance::new`] for more information.
    ///
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::Catch(calc) = map_or_attrs.into_performance() {
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

    /// Specify the amount of fruits of a play i.e. n300.
    pub const fn fruits(mut self, n_fruits: u32) -> Self {
        self.fruits = Some(n_fruits);

        self
    }

    /// Specify the amount of droplets of a play i.e. n100.
    pub const fn droplets(mut self, n_droplets: u32) -> Self {
        self.droplets = Some(n_droplets);

        self
    }

    /// Specify the amount of tiny droplets of a play i.e. n50.
    pub const fn tiny_droplets(mut self, n_tiny_droplets: u32) -> Self {
        self.tiny_droplets = Some(n_tiny_droplets);

        self
    }

    /// Specify the amount of tiny droplet misses of a play i.e. `n_katu`.
    pub const fn tiny_droplet_misses(mut self, n_tiny_droplet_misses: u32) -> Self {
        self.tiny_droplet_misses = Some(n_tiny_droplet_misses);

        self
    }

    /// Specify the amount of fruit / droplet misses of the play.
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
    /// instead of using [`CatchPerformance`] multiple times with different
    /// `passed_objects`, you should use [`CatchGradualPerformance`].
    ///
    /// [`CatchGradualPerformance`]: crate::catch::CatchGradualPerformance
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

    /// Adjust patterns as if the HR mod is enabled.
    pub fn hardrock_offsets(mut self, hardrock_offsets: bool) -> Self {
        self.difficulty = self.difficulty.hardrock_offsets(hardrock_offsets);

        self
    }

    /// Specify how hitresults should be generated.
    pub fn hitresult_generator<H: HitResultGenerator<Catch>>(self) -> CatchPerformance<'map> {
        CatchPerformance {
            map_or_attrs: self.map_or_attrs,
            difficulty: self.difficulty,
            acc: self.acc,
            combo: self.combo,
            fruits: self.fruits,
            droplets: self.droplets,
            tiny_droplets: self.tiny_droplets,
            tiny_droplet_misses: self.tiny_droplet_misses,
            misses: self.misses,
            hitresult_generator: Some(H::generate_hitresults),
        }
    }

    /// Provide parameters through an [`CatchScoreState`].
    #[expect(clippy::needless_pass_by_value, reason = "more sensible")]
    pub const fn state(mut self, state: CatchScoreState) -> Self {
        let CatchScoreState {
            max_combo,
            hitresults,
        } = state;

        self.combo = Some(max_combo);

        self.hitresults(hitresults)
    }

    /// Provide parameters through [`CatchHitResults`].
    pub const fn hitresults(mut self, hitresults: CatchHitResults) -> Self {
        let CatchHitResults {
            fruits: n_fruits,
            droplets: n_droplets,
            tiny_droplets: n_tiny_droplets,
            tiny_droplet_misses: n_tiny_droplet_misses,
            misses,
        } = hitresults;

        self.fruits = Some(n_fruits);
        self.droplets = Some(n_droplets);
        self.tiny_droplets = Some(n_tiny_droplets);
        self.tiny_droplet_misses = Some(n_tiny_droplet_misses);
        self.misses = Some(misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Create the [`CatchScoreState`] that will be used for performance calculation.
    pub fn generate_state(&mut self) -> Result<CatchScoreState, ConvertError> {
        self.map_or_attrs.insert_attrs(&self.difficulty)?;

        // SAFETY: We just calculated and inserted the attributes.
        let attrs = unsafe { self.map_or_attrs.get_attrs() };

        let inspect = Catch::inspect_performance(self, attrs);

        let misses = inspect.misses();
        let max_combo = self.combo.unwrap_or_else(|| attrs.max_combo() - misses);

        let hitresults = match self.hitresult_generator {
            Some(generator) => generator(inspect),
            None => <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect),
        };

        let CatchHitResults {
            fruits,
            droplets,
            tiny_droplets,
            tiny_droplet_misses,
            misses,
        } = hitresults;

        self.combo = Some(max_combo);
        self.fruits = Some(fruits);
        self.droplets = Some(droplets);
        self.tiny_droplets = Some(tiny_droplets);
        self.tiny_droplet_misses = Some(tiny_droplet_misses);
        self.misses = Some(misses);

        Ok(CatchScoreState {
            max_combo,
            hitresults,
        })
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> Result<CatchPerformanceAttributes, ConvertError> {
        let state = self.generate_state()?;

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Attrs(attrs) => attrs,
            MapOrAttrs::Map(ref map) => self.difficulty.calculate_for_mode::<Catch>(map)?,
        };

        Ok(CatchPerformanceCalculator::new(attrs, self.difficulty.get_mods(), state).calculate())
    }

    pub(crate) const fn from_map_or_attrs(map_or_attrs: MapOrAttrs<'map, Catch>) -> Self {
        Self {
            map_or_attrs,
            difficulty: Difficulty::new(),
            acc: None,
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: None,
            hitresult_generator: None,
        }
    }
}

impl<'map> TryFrom<OsuPerformance<'map>> for CatchPerformance<'map> {
    type Error = OsuPerformance<'map>;

    /// Try to create [`CatchPerformance`] through [`OsuPerformance`].
    ///
    /// Returns `None` if [`OsuPerformance`] does not contain a beatmap, i.e.
    /// if it was constructed through attributes or
    /// [`OsuPerformance::generate_state`] was called.
    fn try_from(mut osu: OsuPerformance<'map>) -> Result<Self, Self::Error> {
        let mods = osu.difficulty.get_mods();

        let map = match OsuPerformance::try_convert_map(osu.map_or_attrs, GameMode::Catch, mods) {
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
            n50,
            misses,
            hitresult_priority: _,
            hitresult_generator: _,
            legacy_total_score: _,
        } = osu;

        Ok(Self {
            map_or_attrs: MapOrAttrs::Map(map),
            difficulty,
            acc,
            combo,
            fruits: n300,
            droplets: n100,
            tiny_droplets: n50,
            tiny_droplet_misses: None,
            misses,
            hitresult_generator: None,
        })
    }
}

impl<'map, T: IntoModePerformance<'map, Catch>> From<T> for CatchPerformance<'map> {
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
        catch::CatchDifficultyAttributes,
        osu::{OsuDifficultyAttributes, OsuPerformanceAttributes},
    };

    use super::*;

    static ATTRS: OnceLock<CatchDifficultyAttributes> = OnceLock::new();

    const N_FRUITS: u32 = 728;
    const N_DROPLETS: u32 = 2;
    const N_TINY_DROPLETS: u32 = 263;

    fn beatmap() -> Beatmap {
        Beatmap::from_path("./resources/2118524.osu").unwrap()
    }

    fn attrs() -> CatchDifficultyAttributes {
        ATTRS
            .get_or_init(|| {
                let map = beatmap();
                let attrs = Difficulty::new().calculate_for_mode::<Catch>(&map).unwrap();

                assert_eq!(N_FRUITS, attrs.n_fruits);
                assert_eq!(N_DROPLETS, attrs.n_droplets);
                assert_eq!(N_TINY_DROPLETS, attrs.n_tiny_droplets);

                attrs
            })
            .to_owned()
    }

    #[test]
    fn fruits_missing_objects() {
        let state = CatchPerformance::from(attrs())
            .fruits(N_FRUITS - 10)
            .droplets(N_DROPLETS - 1)
            .tiny_droplets(N_TINY_DROPLETS - 50)
            .tiny_droplet_misses(20)
            .misses(2)
            .generate_state()
            .unwrap();

        let expected = CatchScoreState {
            max_combo: N_FRUITS + N_DROPLETS - 2,
            hitresults: {
                CatchHitResults {
                    fruits: N_FRUITS - 2,
                    droplets: N_DROPLETS,
                    tiny_droplets: N_TINY_DROPLETS - 20,
                    tiny_droplet_misses: 20,
                    misses: 2,
                }
            },
        };

        assert_eq!(state, expected);
    }

    #[test]
    fn create() {
        let mut map = beatmap();

        let _ = CatchPerformance::new(CatchDifficultyAttributes::default());
        let _ = CatchPerformance::new(CatchPerformanceAttributes::default());
        let _ = CatchPerformance::new(&map);
        let _ = CatchPerformance::new(map.clone());

        let _ = CatchPerformance::try_new(CatchDifficultyAttributes::default()).unwrap();
        let _ = CatchPerformance::try_new(CatchPerformanceAttributes::default()).unwrap();
        let _ = CatchPerformance::try_new(DifficultyAttributes::Catch(
            CatchDifficultyAttributes::default(),
        ))
        .unwrap();
        let _ = CatchPerformance::try_new(PerformanceAttributes::Catch(
            CatchPerformanceAttributes::default(),
        ))
        .unwrap();
        let _ = CatchPerformance::try_new(&map).unwrap();
        let _ = CatchPerformance::try_new(map.clone()).unwrap();

        let _ = CatchPerformance::from(CatchDifficultyAttributes::default());
        let _ = CatchPerformance::from(CatchPerformanceAttributes::default());
        let _ = CatchPerformance::from(&map);
        let _ = CatchPerformance::from(map.clone());

        let _ = CatchDifficultyAttributes::default().performance();
        let _ = CatchPerformanceAttributes::default().performance();

        assert!(
            map.convert_mut(GameMode::Osu, &GameMods::default())
                .is_err()
        );

        assert!(CatchPerformance::try_new(OsuDifficultyAttributes::default()).is_none());
        assert!(CatchPerformance::try_new(OsuPerformanceAttributes::default()).is_none());
        assert!(
            CatchPerformance::try_new(
                DifficultyAttributes::Osu(OsuDifficultyAttributes::default())
            )
            .is_none()
        );
        assert!(
            CatchPerformance::try_new(PerformanceAttributes::Osu(
                OsuPerformanceAttributes::default()
            ))
            .is_none()
        );
    }
}
