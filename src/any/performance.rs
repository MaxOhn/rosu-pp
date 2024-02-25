use std::borrow::Cow;

use rosu_map::section::general::GameMode;

use crate::{
    any::attributes::DifficultyAttributes,
    catch::{Catch, CatchPerformance},
    mania::{Mania, ManiaPerformance},
    model::beatmap::{Beatmap, Converted},
    osu::{Osu, OsuPerformance},
    taiko::{Taiko, TaikoPerformance},
};

use super::{
    attributes::{AttributeProvider, PerformanceAttributes},
    score_state::ScoreState,
};

/// Performance calculator on maps of any mode.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum Performance<'map> {
    Osu(OsuPerformance<'map>),
    Taiko(TaikoPerformance<'map>),
    Catch(CatchPerformance<'map>),
    Mania(ManiaPerformance<'map>),
}

impl<'map> Performance<'map> {
    /// Create a new performance calculator for maps of any mode.
    pub fn new(map: &'map Beatmap) -> Self {
        let map = Cow::Borrowed(map);

        match map.mode {
            GameMode::Osu => Self::Osu(OsuPerformance::new(Converted::new(map, false))),
            GameMode::Taiko => Self::Taiko(TaikoPerformance::new(Converted::new(map, false))),
            GameMode::Catch => Self::Catch(CatchPerformance::new(Converted::new(map, false))),
            GameMode::Mania => Self::Mania(ManiaPerformance::new(Converted::new(map, false))),
        }
    }

    /// Create a new performance calculator through previously calculated
    /// attributes.
    ///
    /// Note that the map, mods, and passed object count should be the same
    /// as when the attributes were calculated.
    pub fn from_attributes(attributes: impl AttributeProvider) -> Self {
        Self::from(attributes)
    }

    /// Consume the performance calculator and calculate
    /// performance attributes for the given parameters.
    pub fn calculate(self) -> PerformanceAttributes {
        match self {
            Self::Osu(o) => PerformanceAttributes::Osu(o.calculate()),
            Self::Taiko(t) => PerformanceAttributes::Taiko(t.calculate()),
            Self::Catch(f) => PerformanceAttributes::Catch(f.calculate()),
            Self::Mania(m) => PerformanceAttributes::Mania(m.calculate()),
        }
    }

    /// Provide the result of a previous difficulty or performance calculation.
    /// If you already calculated the attributes for the current map-mod combination,
    /// be sure to put them in here so that they don't have to be recalculated.
    pub fn attributes(self, attributes: impl AttributeProvider) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.attributes(attributes.attributes())),
            Self::Taiko(t) => Self::Taiko(t.attributes(attributes.attributes())),
            Self::Catch(f) => Self::Catch(f.attributes(attributes.attributes())),
            Self::Mania(m) => Self::Mania(m.attributes(attributes.attributes())),
        }
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// Returns `None` if the conversion is incompatible or the internal
    /// beatmap was already replaced with difficulty attributes, i.e. if
    /// [`Performance::attributes`] or [`Performance::generate_state`] was
    /// called.
    ///
    /// If the given mode should be ignored in case it is incompatible or if
    /// the internal beatmap was replaced, use [`mode_or_ignore`] instead.
    ///
    /// [`mode_or_ignore`]: Self::mode_or_ignore
    pub fn try_mode(self, mode: GameMode) -> Option<Self> {
        match (self, mode) {
            (Self::Osu(o), _) => o.try_mode(mode),
            (this @ Self::Taiko(_), GameMode::Taiko)
            | (this @ Self::Catch(_), GameMode::Catch)
            | (this @ Self::Mania(_), GameMode::Mania) => Some(this),
            _ => None,
        }
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// If the conversion is incompatible or if the internal beatmap was
    /// already replaced with difficulty attributes, the map won't be modified.
    ///
    /// To see whether the given mode is incompatible or the internal beatmap
    /// was replaced, use [`try_mode`] instead.
    ///
    /// [`try_mode`]: Self::try_mode
    pub fn mode_or_ignore(self, mode: GameMode) -> Self {
        if let Self::Osu(osu) = self {
            osu.mode_or_ignore(mode)
        } else {
            self
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    pub fn mods(self, mods: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.mods(mods)),
            Self::Taiko(t) => Self::Taiko(t.mods(mods)),
            Self::Catch(f) => Self::Catch(f.mods(mods)),
            Self::Mania(m) => Self::Mania(m.mods(mods)),
        }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects,
    /// instead of using [`Performance`] multiple times with different
    /// `passed_objects`, you should use [`GradualPerformance`].
    ///
    /// [`GradualPerformance`]: crate::GradualPerformance
    pub fn passed_objects(self, passed_objects: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.passed_objects(passed_objects)),
            Self::Taiko(t) => Self::Taiko(t.passed_objects(passed_objects)),
            Self::Catch(f) => Self::Catch(f.passed_objects(passed_objects)),
            Self::Mania(m) => Self::Mania(m.passed_objects(passed_objects)),
        }
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.clock_rate(clock_rate)),
            Self::Taiko(t) => Self::Taiko(t.clock_rate(clock_rate)),
            Self::Catch(f) => Self::Catch(f.clock_rate(clock_rate)),
            Self::Mania(m) => Self::Mania(m.clock_rate(clock_rate)),
        }
    }

    /// Provide parameters through a [`ScoreState`].
    pub fn state(self, state: ScoreState) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.state(state.into())),
            Self::Taiko(t) => Self::Taiko(t.state(state.into())),
            Self::Catch(f) => Self::Catch(f.state(state.into())),
            Self::Mania(m) => Self::Mania(m.state(state.into())),
        }
    }

    /// Set the accuracy between `0.0` and `100.0`.
    pub fn accuracy(self, acc: f64) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.accuracy(acc)),
            Self::Taiko(t) => Self::Taiko(t.accuracy(acc)),
            Self::Catch(f) => Self::Catch(f.accuracy(acc)),
            Self::Mania(m) => Self::Mania(m.accuracy(acc)),
        }
    }

    /// Specify the amount of misses of a play.
    pub fn n_misses(self, n_misses: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.n_misses(n_misses)),
            Self::Taiko(t) => Self::Taiko(t.n_misses(n_misses)),
            Self::Catch(f) => Self::Catch(f.misses(n_misses)),
            Self::Mania(m) => Self::Mania(m.n_misses(n_misses)),
        }
    }

    /// Specify the max combo of the play.
    ///
    /// Irrelevant for osu!mania.
    pub fn combo(self, combo: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.combo(combo)),
            Self::Taiko(t) => Self::Taiko(t.combo(combo)),
            Self::Catch(f) => Self::Catch(f.combo(combo)),
            Self::Mania(_) => self,
        }
    }

    /// Specify how hitresults should be generated.
    ///
    /// Defauls to [`HitResultPriority::BestCase`].
    pub fn hitresult_priority(self, priority: HitResultPriority) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.hitresult_priority(priority)),
            Self::Taiko(t) => Self::Taiko(t.hitresult_priority(priority)),
            Self::Catch(_) => self,
            Self::Mania(m) => Self::Mania(m.hitresult_priority(priority)),
        }
    }

    /// Specify the amount of 300s of a play.
    pub fn n300(self, n300: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.n300(n300)),
            Self::Taiko(t) => Self::Taiko(t.n300(n300)),
            Self::Catch(f) => Self::Catch(f.fruits(n300)),
            Self::Mania(m) => Self::Mania(m.n300(n300)),
        }
    }

    /// Specify the amount of 100s of a play.
    pub fn n100(self, n100: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.n100(n100)),
            Self::Taiko(t) => Self::Taiko(t.n100(n100)),
            Self::Catch(f) => Self::Catch(f.droplets(n100)),
            Self::Mania(m) => Self::Mania(m.n100(n100)),
        }
    }

    /// Specify the amount of 50s of a play.
    ///
    /// Irrelevant for osu!taiko.
    pub fn n50(self, n50: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.n50(n50)),
            Self::Taiko(_) => self,
            Self::Catch(f) => Self::Catch(f.tiny_droplets(n50)),
            Self::Mania(m) => Self::Mania(m.n50(n50)),
        }
    }

    /// Specify the amount of katus of a play.
    ///
    /// This value is only relevant for osu!catch for which it represents
    /// the amount of tiny droplet misses and osu!mania for which it.
    /// repesents the amount of n200.
    pub fn n_katu(self, n_katu: u32) -> Self {
        match self {
            Self::Osu(_) | Self::Taiko(_) => self,
            Self::Catch(f) => Self::Catch(f.tiny_droplet_misses(n_katu)),
            Self::Mania(m) => Self::Mania(m.n200(n_katu)),
        }
    }

    /// Specify the amount of gekis of a play.
    ///
    /// This value is only relevant for osu!mania for which it.
    /// repesents the amount of n320.
    pub fn n_geki(self, n_geki: u32) -> Self {
        match self {
            Self::Osu(_) | Self::Taiko(_) | Self::Catch(_) => self,
            Self::Mania(m) => Self::Mania(m.n320(n_geki)),
        }
    }

    /// Create the [`ScoreState`] that will be used for performance calculation.
    pub fn generate_state(&mut self) -> ScoreState {
        match self {
            Self::Osu(o) => o.generate_state().into(),
            Self::Taiko(t) => t.generate_state().into(),
            Self::Catch(f) => f.generate_state().into(),
            Self::Mania(m) => m.generate_state().into(),
        }
    }
}

impl<A: AttributeProvider> From<A> for Performance<'_> {
    fn from(attrs: A) -> Self {
        fn inner(attrs: DifficultyAttributes) -> Performance<'static> {
            match attrs {
                DifficultyAttributes::Osu(attrs) => Performance::Osu(attrs.performance()),
                DifficultyAttributes::Taiko(attrs) => Performance::Taiko(attrs.performance()),
                DifficultyAttributes::Catch(attrs) => Performance::Catch(attrs.performance()),
                DifficultyAttributes::Mania(attrs) => Performance::Mania(attrs.performance()),
            }
        }

        inner(attrs.attributes())
    }
}

macro_rules! impl_from_mode {
    ( $mode:ident: $performance:ident ) => {
        impl<'a> From<Converted<'a, $mode>> for Performance<'a> {
            fn from(converted: Converted<'a, $mode>) -> Self {
                Self::$mode($performance::new(converted))
            }
        }

        impl<'a, 'b: 'a> From<&'b Converted<'a, $mode>> for Performance<'a> {
            fn from(converted: &'b Converted<'a, $mode>) -> Self {
                Self::$mode($performance::new(converted.as_owned()))
            }
        }
    };
}

impl_from_mode!(Osu: OsuPerformance);
impl_from_mode!(Taiko: TaikoPerformance);
impl_from_mode!(Catch: CatchPerformance);
impl_from_mode!(Mania: ManiaPerformance);

/// While generating remaining hitresults, decide how they should be distributed.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum HitResultPriority {
    /// Prioritize good hitresults over bad ones
    #[default]
    BestCase,
    /// Prioritize bad hitresults over good ones
    WorstCase,
}
