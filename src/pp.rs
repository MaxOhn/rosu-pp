use crate::{Beatmap, DifficultyAttributes, GameMode, PerformanceAttributes, ScoreState};

#[cfg(feature = "fruits")]
use crate::fruits::{FruitsDifficultyAttributes, FruitsPP};

#[cfg(feature = "mania")]
use crate::mania::{ManiaDifficultyAttributes, ManiaPP};

#[cfg(feature = "osu")]
use crate::osu::{OsuDifficultyAttributes, OsuPP};

#[cfg(feature = "taiko")]
use crate::taiko::{TaikoDifficultyAttributes, TaikoPP};

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
    #[cfg(feature = "fruits")]
    /// osu!ctb performance calculator
    Fruits(FruitsPP<'map>),
    #[cfg(feature = "mania")]
    /// osu!mania performance calculator
    Mania(ManiaPP<'map>),
    #[cfg(feature = "osu")]
    /// osu!standard performance calculator
    Osu(OsuPP<'map>),
    #[cfg(feature = "taiko")]
    /// osu!taiko performance calculator
    Taiko(TaikoPP<'map>),
}

impl<'map> AnyPP<'map> {
    /// Create a new performance calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        match map.mode {
            #[cfg(feature = "fruits")]
            GameMode::CTB => Self::Fruits(FruitsPP::new(map)),
            #[cfg(feature = "mania")]
            GameMode::MNA => Self::Mania(ManiaPP::new(map)),
            #[cfg(feature = "osu")]
            GameMode::STD => Self::Osu(OsuPP::new(map)),
            #[cfg(feature = "taiko")]
            GameMode::TKO => Self::Taiko(TaikoPP::new(map)),
            #[allow(unreachable_patterns)]
            _ => panic!("feature for mode {:?} is not enabled", map.mode),
        }
    }

    /// Consume the performance calculator and calculate
    /// performance attributes for the given parameters.
    #[inline]
    pub fn calculate(self) -> PerformanceAttributes {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => PerformanceAttributes::Fruits(f.calculate()),
            #[cfg(feature = "mania")]
            Self::Mania(m) => PerformanceAttributes::Mania(m.calculate()),
            #[cfg(feature = "osu")]
            Self::Osu(o) => PerformanceAttributes::Osu(o.calculate()),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => PerformanceAttributes::Taiko(t.calculate()),
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    #[inline]
    pub fn attributes(self, attributes: impl AttributeProvider) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.attributes(attributes.attributes())),
            #[cfg(feature = "mania")]
            Self::Mania(m) => Self::Mania(m.attributes(attributes.attributes())),
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.attributes(attributes.attributes())),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.attributes(attributes.attributes())),
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(self, mods: u32) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.mods(mods)),
            #[cfg(feature = "mania")]
            Self::Mania(m) => Self::Mania(m.mods(mods)),
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.mods(mods)),
            #[cfg(feature = "taiko")]
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
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.passed_objects(passed_objects)),
            #[cfg(feature = "mania")]
            Self::Mania(m) => Self::Mania(m.passed_objects(passed_objects)),
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.passed_objects(passed_objects)),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.passed_objects(passed_objects)),
        }
    }

    /// Provide parameters through a [`ScoreState`].
    #[inline]
    pub fn state(self, state: ScoreState) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.state(state.into())),
            #[cfg(feature = "mania")]
            Self::Mania(m) => Self::Mania(m.score(state.score)),
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.state(state.into())),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.state(state.into())),
        }
    }

    /// Set the accuracy between 0.0 and 100.0.
    ///
    /// For some modes this method depends on previously set values.
    /// Be sure to call this last before calling `calculate`.
    ///
    /// Irrelevant for osu!mania.
    #[allow(unused_variables)]
    #[inline]
    pub fn accuracy(self, acc: f64) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.accuracy(acc)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.accuracy(acc)),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.accuracy(acc)),
        }
    }

    /// Specify the amount of misses of a play.
    ///
    /// Irrelevant for osu!mania.
    #[allow(unused_variables)]
    #[inline]
    pub fn misses(self, misses: usize) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.misses(misses)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.misses(misses)),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.misses(misses)),
        }
    }

    /// Specify the max combo of the play.
    ///
    /// Irrelevant for osu!mania.
    #[allow(unused_variables)]
    #[inline]
    pub fn combo(self, combo: usize) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.combo(combo)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.combo(combo)),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.combo(combo)),
        }
    }

    /// Specify the amount of 300s of a play.
    ///
    /// Irrelevant for osu!mania.
    #[allow(unused_variables)]
    #[inline]
    pub fn n300(self, n300: usize) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.fruits(n300)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.n300(n300)),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.n300(n300)),
        }
    }

    /// Specify the amount of 100s of a play.
    ///
    /// Irrelevant for osu!mania.
    #[allow(unused_variables)]
    #[inline]
    pub fn n100(self, n100: usize) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.droplets(n100)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.n100(n100)),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Self::Taiko(t.n100(n100)),
        }
    }

    /// Specify the amount of 50s of a play.
    ///
    /// Irrelevant for osu!mania and osu!taiko.
    #[allow(unused_variables)]
    #[inline]
    pub fn n50(self, n50: usize) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.tiny_droplets(n50)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Self::Osu(o.n50(n50)),
            #[cfg(feature = "taiko")]
            Self::Taiko(_) => self,
        }
    }

    /// Specify the amount of katus of a play.
    ///
    /// This value is only relevant for osu!ctb for which it represents
    /// the amount of tiny droplet misses.
    #[allow(unused_variables)]
    #[inline]
    pub fn n_katu(self, n_katu: usize) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Self::Fruits(f.tiny_droplet_misses(n_katu)),
            #[cfg(feature = "mania")]
            Self::Mania(_) => self,
            #[cfg(feature = "osu")]
            Self::Osu(_) => self,
            #[cfg(feature = "taiko")]
            Self::Taiko(_) => self,
        }
    }

    /// Specify the score of a play.
    ///
    /// This value is only relevant for osu!mania.
    ///
    /// On `NoMod` its between 0 and 1,000,000, on `Easy` between 0 and 500,000, etc.
    #[allow(unused_variables)]
    #[inline]
    pub fn score(self, score: u32) -> Self {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(_) => self,
            #[cfg(feature = "mania")]
            Self::Mania(m) => Self::Mania(m.score(score)),
            #[cfg(feature = "osu")]
            Self::Osu(_) => self,
            #[cfg(feature = "taiko")]
            Self::Taiko(_) => self,
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
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => DifficultyAttributes::Fruits(f.difficulty),
            #[cfg(feature = "mania")]
            Self::Mania(m) => DifficultyAttributes::Mania(m.difficulty),
            #[cfg(feature = "osu")]
            Self::Osu(o) => DifficultyAttributes::Osu(o.difficulty),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => DifficultyAttributes::Taiko(t.difficulty),
        }
    }
}

#[cfg(feature = "fruits")]
impl AttributeProvider for FruitsDifficultyAttributes {
    fn attributes(self) -> DifficultyAttributes {
        DifficultyAttributes::Fruits(self)
    }
}

#[cfg(feature = "mania")]
impl AttributeProvider for ManiaDifficultyAttributes {
    fn attributes(self) -> DifficultyAttributes {
        DifficultyAttributes::Mania(self)
    }
}

#[cfg(feature = "osu")]
impl AttributeProvider for OsuDifficultyAttributes {
    fn attributes(self) -> DifficultyAttributes {
        DifficultyAttributes::Osu(self)
    }
}

#[cfg(feature = "taiko")]
impl AttributeProvider for TaikoDifficultyAttributes {
    fn attributes(self) -> DifficultyAttributes {
        DifficultyAttributes::Taiko(self)
    }
}
