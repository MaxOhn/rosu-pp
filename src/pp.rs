use crate::{
    catch::{CatchDifficultyAttributes, CatchPP, CatchPerformanceAttributes},
    mania::{ManiaDifficultyAttributes, ManiaPP, ManiaPerformanceAttributes},
    osu::{OsuDifficultyAttributes, OsuPP, OsuPerformanceAttributes},
    taiko::{TaikoDifficultyAttributes, TaikoPP, TaikoPerformanceAttributes},
    Beatmap, DifficultyAttributes, GameMode, PerformanceAttributes, ScoreState,
};

/// Performance calculator on maps of any mode.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{AnyPP, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
///
/// # let map = Beatmap::default();
/// let pp_result = AnyPP::new(&map)
///     .mods(8 + 64) // HDDT
///     .combo(1234)
///     .misses(1)
///     .accuracy(98.5) // should be set last
///     .calculate();
///
/// println!("PP: {} | Stars: {}", pp_result.pp(), pp_result.stars());
///
/// let next_result = AnyPP::new(&map)
///     .attributes(pp_result)  // reusing previous results for performance
///     .mods(8 + 64)           // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp(), next_result.stars());
/// ```
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum AnyPP<'map> {
    /// osu!catch performance calculator
    Catch(CatchPP<'map>),
    /// osu!mania performance calculator
    Mania(ManiaPP<'map>),
    /// osu!standard performance calculator
    Osu(OsuPP<'map>),
    /// osu!taiko performance calculator
    Taiko(TaikoPP<'map>),
}

impl<'map> AnyPP<'map> {
    /// Create a new performance calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        match map.mode {
            GameMode::Catch => Self::Catch(CatchPP::new(map)),
            GameMode::Mania => Self::Mania(ManiaPP::new(map)),
            GameMode::Osu => Self::Osu(OsuPP::new(map)),
            GameMode::Taiko => Self::Taiko(TaikoPP::new(map)),
        }
    }

    /// Consume the performance calculator and calculate
    /// performance attributes for the given parameters.
    #[inline]
    pub fn calculate(self) -> PerformanceAttributes {
        match self {
            Self::Catch(f) => PerformanceAttributes::Catch(f.calculate()),
            Self::Mania(m) => PerformanceAttributes::Mania(m.calculate()),
            Self::Osu(o) => PerformanceAttributes::Osu(o.calculate()),
            Self::Taiko(t) => PerformanceAttributes::Taiko(t.calculate()),
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    #[inline]
    pub fn attributes(self, attributes: impl AttributeProvider) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.attributes(attributes.attributes())),
            Self::Mania(m) => Self::Mania(m.attributes(attributes.attributes())),
            Self::Osu(o) => Self::Osu(o.attributes(attributes.attributes())),
            Self::Taiko(t) => Self::Taiko(t.attributes(attributes.attributes())),
        }
    }

    /// If the map is an osu!standard map, convert it to another mode.
    #[inline]
    pub fn mode(self, mode: GameMode) -> Self {
        match self {
            AnyPP::Osu(o) => match mode {
                GameMode::Osu => AnyPP::Osu(o),
                GameMode::Taiko => AnyPP::Taiko(o.into()),
                GameMode::Catch => AnyPP::Catch(o.into()),
                GameMode::Mania => AnyPP::Mania(o.into()),
            },
            other => other,
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(self, mods: u32) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.mods(mods)),
            Self::Mania(m) => Self::Mania(m.mods(mods)),
            Self::Osu(o) => Self::Osu(o.mods(mods)),
            Self::Taiko(t) => Self::Taiko(t.mods(mods)),
        }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`AnyPP`] multiple times with different `passed_objects`, you should use
    /// [`GradualPerformanceAttributes`](crate::GradualPerformanceAttributes).
    #[inline]
    pub fn passed_objects(self, passed_objects: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.passed_objects(passed_objects)),
            Self::Mania(m) => Self::Mania(m.passed_objects(passed_objects)),
            Self::Osu(o) => Self::Osu(o.passed_objects(passed_objects)),
            Self::Taiko(t) => Self::Taiko(t.passed_objects(passed_objects)),
        }
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    #[inline]
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.clock_rate(clock_rate)),
            Self::Mania(m) => Self::Mania(m.clock_rate(clock_rate)),
            Self::Osu(o) => Self::Osu(o.clock_rate(clock_rate)),
            Self::Taiko(t) => Self::Taiko(t.clock_rate(clock_rate)),
        }
    }

    /// Provide parameters through a [`ScoreState`].
    #[inline]
    pub fn state(self, state: ScoreState) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.state(state.into())),
            Self::Mania(m) => Self::Mania(m.state(state.into())),
            Self::Osu(o) => Self::Osu(o.state(state.into())),
            Self::Taiko(t) => Self::Taiko(t.state(state.into())),
        }
    }

    /// Set the accuracy between 0.0 and 100.0.
    ///
    /// For some modes this method depends on previously set values.
    /// Be sure to call this last before calling `calculate`.
    #[inline]
    pub fn accuracy(self, acc: f64) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.accuracy(acc)),
            Self::Mania(m) => Self::Mania(m.accuracy(acc)),
            Self::Osu(o) => Self::Osu(o.accuracy(acc)),
            Self::Taiko(t) => Self::Taiko(t.accuracy(acc)),
        }
    }

    /// Specify the amount of misses of a play.
    #[inline]
    pub fn n_misses(self, n_misses: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.misses(n_misses)),
            Self::Mania(m) => Self::Mania(m.n_misses(n_misses)),
            Self::Osu(o) => Self::Osu(o.n_misses(n_misses)),
            Self::Taiko(t) => Self::Taiko(t.n_misses(n_misses)),
        }
    }

    /// Specify the max combo of the play.
    ///
    /// Irrelevant for osu!mania.
    #[inline]
    pub fn combo(self, combo: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.combo(combo)),
            Self::Mania(_) => self,
            Self::Osu(o) => Self::Osu(o.combo(combo)),
            Self::Taiko(t) => Self::Taiko(t.combo(combo)),
        }
    }

    /// Specify the amount of 300s of a play.
    #[inline]
    pub fn n300(self, n300: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.fruits(n300)),
            Self::Mania(m) => Self::Mania(m.n300(n300)),
            Self::Osu(o) => Self::Osu(o.n300(n300)),
            Self::Taiko(t) => Self::Taiko(t.n300(n300)),
        }
    }

    /// Specify the amount of 100s of a play.
    #[inline]
    pub fn n100(self, n100: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.droplets(n100)),
            Self::Mania(m) => Self::Mania(m.n100(n100)),
            Self::Osu(o) => Self::Osu(o.n100(n100)),
            Self::Taiko(t) => Self::Taiko(t.n100(n100)),
        }
    }

    /// Specify the amount of 50s of a play.
    ///
    /// Irrelevant for osu!taiko.
    #[inline]
    pub fn n50(self, n50: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.tiny_droplets(n50)),
            Self::Mania(m) => Self::Mania(m.n50(n50)),
            Self::Osu(o) => Self::Osu(o.n50(n50)),
            Self::Taiko(_) => self,
        }
    }

    /// Specify the amount of katus of a play.
    ///
    /// This value is only relevant for osu!catch for which it represents
    /// the amount of tiny droplet misses and osu!mania for which it.
    /// repesents the amount of n200.
    #[inline]
    pub fn n_katu(self, n_katu: usize) -> Self {
        match self {
            Self::Catch(f) => Self::Catch(f.tiny_droplet_misses(n_katu)),
            Self::Mania(m) => Self::Mania(m.n200(n_katu)),
            Self::Osu(_) => self,
            Self::Taiko(_) => self,
        }
    }

    /// Specify the amount of gekis of a play.
    ///
    /// This value is only relevant for osu!mania for which it.
    /// repesents the amount of n320.
    #[inline]
    pub fn n_geki(self, n_geki: usize) -> Self {
        match self {
            Self::Mania(m) => Self::Mania(m.n320(n_geki)),
            Self::Osu(_) => self,
            Self::Taiko(_) => self,
            Self::Catch(_) => self,
        }
    }
}

/// Abstract type to provide flexibility when passing difficulty attributes to a performance calculation.
pub trait AttributeProvider {
    /// Provide the actual difficulty attributes.
    fn attributes(self) -> DifficultyAttributes;
}

impl AttributeProvider for DifficultyAttributes {
    #[inline]
    fn attributes(self) -> DifficultyAttributes {
        self
    }
}

impl AttributeProvider for PerformanceAttributes {
    #[inline]
    fn attributes(self) -> DifficultyAttributes {
        match self {
            Self::Catch(f) => DifficultyAttributes::Catch(f.difficulty),
            Self::Mania(m) => DifficultyAttributes::Mania(m.difficulty),
            Self::Osu(o) => DifficultyAttributes::Osu(o.difficulty),
            Self::Taiko(t) => DifficultyAttributes::Taiko(t.difficulty),
        }
    }
}

macro_rules! impl_attr_provider {
    ($mode:ident: $difficulty:ident, $performance:ident) => {
        impl AttributeProvider for $difficulty {
            #[inline]
            fn attributes(self) -> DifficultyAttributes {
                DifficultyAttributes::$mode(self)
            }
        }

        impl AttributeProvider for $performance {
            #[inline]
            fn attributes(self) -> DifficultyAttributes {
                DifficultyAttributes::$mode(self.difficulty)
            }
        }
    };
}

impl_attr_provider!(Catch: CatchDifficultyAttributes, CatchPerformanceAttributes);
impl_attr_provider!(Mania: ManiaDifficultyAttributes, ManiaPerformanceAttributes);
impl_attr_provider!(Osu: OsuDifficultyAttributes, OsuPerformanceAttributes);
impl_attr_provider!(Taiko: TaikoDifficultyAttributes, TaikoPerformanceAttributes);
