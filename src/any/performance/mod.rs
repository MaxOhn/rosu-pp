use rosu_map::section::general::GameMode;

use crate::{
    Difficulty, GameMods, catch::CatchPerformance, mania::ManiaPerformance, osu::OsuPerformance,
    taiko::TaikoPerformance,
};

use self::into::IntoPerformance;

use super::{attributes::PerformanceAttributes, score_state::ScoreState};

pub mod gradual;
pub mod into;

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
    /// Create a new performance calculator for any mode.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`DifficultyAttributes`],
    ///   [`PerformanceAttributes`], or mode-specific attributes like
    ///   [`TaikoDifficultyAttributes`], [`ManiaPerformanceAttributes`], ...)
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
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`TaikoDifficultyAttributes`]: crate::taiko::TaikoDifficultyAttributes
    /// [`ManiaPerformanceAttributes`]: crate::mania::ManiaPerformanceAttributes
    pub fn new(map_or_attrs: impl IntoPerformance<'map>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Consume the performance calculator and calculate
    /// performance attributes for the given parameters.
    #[allow(clippy::missing_panics_doc)]
    pub fn calculate(self) -> PerformanceAttributes {
        match self {
            Self::Osu(o) => {
                PerformanceAttributes::Osu(o.calculate().expect("no conversion required"))
            }
            Self::Taiko(t) => {
                PerformanceAttributes::Taiko(t.calculate().expect("no conversion required"))
            }
            Self::Catch(f) => {
                PerformanceAttributes::Catch(f.calculate().expect("no conversion required"))
            }
            Self::Mania(m) => {
                PerformanceAttributes::Mania(m.calculate().expect("no conversion required"))
            }
        }
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// Returns `Err(self)` if the conversion is incompatible or no beatmap is
    /// contained, i.e. if this [`Performance`] was created through attributes
    /// or [`Performance::generate_state`] was called.
    ///
    /// If the given mode should be ignored in case of an error, use
    /// [`mode_or_ignore`] instead.
    ///
    /// [`mode_or_ignore`]: Self::mode_or_ignore
    // Both variants have the same size
    #[allow(clippy::result_large_err)]
    pub fn try_mode(self, mode: GameMode) -> Result<Self, Self> {
        match (self, mode) {
            (Self::Osu(o), _) => o.try_mode(mode).map_err(Self::Osu),
            (this @ Self::Taiko(_), GameMode::Taiko)
            | (this @ Self::Catch(_), GameMode::Catch)
            | (this @ Self::Mania(_), GameMode::Mania) => Ok(this),
            (this, _) => Err(this),
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
    pub fn mods(self, mods: impl Into<GameMods>) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.mods(mods)),
            Self::Taiko(t) => Self::Taiko(t.mods(mods)),
            Self::Catch(f) => Self::Catch(f.mods(mods)),
            Self::Mania(m) => Self::Mania(m.mods(mods)),
        }
    }

    /// Use the specified settings of the given [`Difficulty`].
    pub fn difficulty(self, difficulty: Difficulty) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.difficulty(difficulty)),
            Self::Taiko(t) => Self::Taiko(t.difficulty(difficulty)),
            Self::Catch(f) => Self::Catch(f.difficulty(difficulty)),
            Self::Mania(m) => Self::Mania(m.difficulty(difficulty)),
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
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | 0.01    | 100     |
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.clock_rate(clock_rate)),
            Self::Taiko(t) => Self::Taiko(t.clock_rate(clock_rate)),
            Self::Catch(f) => Self::Catch(f.clock_rate(clock_rate)),
            Self::Mania(m) => Self::Mania(m.clock_rate(clock_rate)),
        }
    }

    /// Override a beatmap's set AR.
    ///
    /// Only relevant for osu! and osu!catch.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn ar(self, ar: f32, with_mods: bool) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.ar(ar, with_mods)),
            Self::Catch(c) => Self::Catch(c.ar(ar, with_mods)),
            Self::Taiko(_) | Self::Mania(_) => self,
        }
    }

    /// Override a beatmap's set CS.
    ///
    /// Only relevant for osu! and osu!catch.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn cs(self, cs: f32, with_mods: bool) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.cs(cs, with_mods)),
            Self::Catch(c) => Self::Catch(c.cs(cs, with_mods)),
            Self::Taiko(_) | Self::Mania(_) => self,
        }
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
    pub fn hp(self, hp: f32, with_mods: bool) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.hp(hp, with_mods)),
            Self::Taiko(t) => Self::Taiko(t.hp(hp, with_mods)),
            Self::Catch(c) => Self::Catch(c.hp(hp, with_mods)),
            Self::Mania(m) => Self::Mania(m.hp(hp, with_mods)),
        }
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
    pub fn od(self, od: f32, with_mods: bool) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.od(od, with_mods)),
            Self::Taiko(t) => Self::Taiko(t.od(od, with_mods)),
            Self::Catch(c) => Self::Catch(c.od(od, with_mods)),
            Self::Mania(m) => Self::Mania(m.od(od, with_mods)),
        }
    }

    /// Adjust patterns as if the HR mod is enabled.
    ///
    /// Only relevant for osu!catch.
    pub fn hardrock_offsets(self, hardrock_offsets: bool) -> Self {
        if let Self::Catch(catch) = self {
            Self::Catch(catch.hardrock_offsets(hardrock_offsets))
        } else {
            self
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
    pub fn misses(self, n_misses: u32) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.misses(n_misses)),
            Self::Taiko(t) => Self::Taiko(t.misses(n_misses)),
            Self::Catch(f) => Self::Catch(f.misses(n_misses)),
            Self::Mania(m) => Self::Mania(m.misses(n_misses)),
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

    /// Whether the calculated attributes belong to an osu!lazer or osu!stable
    /// score.
    ///
    /// Defaults to `true`.
    ///
    /// This affects internal accuracy calculation because lazer considers
    /// slider heads for accuracy whereas stable does not.
    ///
    /// Only relevant for osu!standard and osu!mania.
    pub fn lazer(self, lazer: bool) -> Self {
        match self {
            Self::Osu(o) => Self::Osu(o.lazer(lazer)),
            Self::Taiko(_) | Self::Catch(_) => self,
            Self::Mania(m) => Self::Mania(m.lazer(lazer)),
        }
    }

    /// Specify the amount of "large tick" hits.
    ///
    /// Only relevant for osu!standard.
    ///
    /// The meaning depends on the kind of score:
    /// - if set on osu!stable, this value is irrelevant and can be `0`
    /// - if set on osu!lazer *with* slider accuracy, this value is the amount
    ///   of hit slider ticks and repeats
    /// - if set on osu!lazer *without* slider accuracy, this value is the
    ///   amount of hit slider heads, ticks, and repeats
    pub fn large_tick_hits(self, large_tick_hits: u32) -> Self {
        if let Self::Osu(osu) = self {
            Self::Osu(osu.large_tick_hits(large_tick_hits))
        } else {
            self
        }
    }

    /// Specify the amount of "small tick" hits.
    ///
    /// Only relevant for osu!standard lazer scores without slider accuracy. In
    /// that case, this value is the amount of slider tail hits.
    pub fn small_tick_hits(self, small_tick_hits: u32) -> Self {
        if let Self::Osu(osu) = self {
            Self::Osu(osu.small_tick_hits(small_tick_hits))
        } else {
            self
        }
    }

    /// Specify the amount of hit slider ends.
    ///
    /// Only relevant for osu!standard lazer scores with slider accuracy.
    pub fn slider_end_hits(self, slider_end_hits: u32) -> Self {
        if let Self::Osu(osu) = self {
            Self::Osu(osu.slider_end_hits(slider_end_hits))
        } else {
            self
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
    /// Only relevant for osu!catch for which it represents the amount of tiny
    /// droplet misses and osu!mania for which it repesents the amount of n200.
    pub fn n_katu(self, n_katu: u32) -> Self {
        match self {
            Self::Osu(_) | Self::Taiko(_) => self,
            Self::Catch(f) => Self::Catch(f.tiny_droplet_misses(n_katu)),
            Self::Mania(m) => Self::Mania(m.n200(n_katu)),
        }
    }

    /// Specify the amount of gekis of a play.
    ///
    /// Only relevant for osu!mania for which it repesents the
    /// amount of n320.
    pub fn n_geki(self, n_geki: u32) -> Self {
        match self {
            Self::Osu(_) | Self::Taiko(_) | Self::Catch(_) => self,
            Self::Mania(m) => Self::Mania(m.n320(n_geki)),
        }
    }

    /// Create the [`ScoreState`] that will be used for performance calculation.
    #[allow(clippy::missing_panics_doc)]
    pub fn generate_state(&mut self) -> ScoreState {
        match self {
            Self::Osu(o) => o.generate_state().expect("no conversion required").into(),
            Self::Taiko(t) => t.generate_state().expect("no conversion required").into(),
            Self::Catch(f) => f.generate_state().expect("no conversion required").into(),
            Self::Mania(m) => m.generate_state().expect("no conversion required").into(),
        }
    }
}

/// While generating remaining hitresults, decide how they should be distributed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HitResultPriority {
    /// Prioritize good hitresults over bad ones
    BestCase,
    /// Prioritize bad hitresults over good ones
    WorstCase,
    /// Prioritize fast hitresults generation
    Fastest,
}

impl HitResultPriority {
    pub(crate) const DEFAULT: Self = Self::BestCase;
}

impl Default for HitResultPriority {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'a, T: IntoPerformance<'a>> From<T> for Performance<'a> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Beatmap,
        any::DifficultyAttributes,
        catch::{CatchDifficultyAttributes, CatchPerformanceAttributes},
        mania::{ManiaDifficultyAttributes, ManiaPerformanceAttributes},
        osu::{OsuDifficultyAttributes, OsuPerformanceAttributes},
        taiko::{TaikoDifficultyAttributes, TaikoPerformanceAttributes},
    };

    use super::*;

    #[test]
    fn create() {
        let map = Beatmap::from_path("./resources/1028484.osu").unwrap();

        let _ = Performance::new(&map);
        let _ = Performance::new(map.clone());

        let _ = Performance::new(OsuDifficultyAttributes::default());
        let _ = Performance::new(TaikoDifficultyAttributes::default());
        let _ = Performance::new(CatchDifficultyAttributes::default());
        let _ = Performance::new(ManiaDifficultyAttributes::default());

        let _ = Performance::new(OsuPerformanceAttributes::default());
        let _ = Performance::new(TaikoPerformanceAttributes::default());
        let _ = Performance::new(CatchPerformanceAttributes::default());
        let _ = Performance::new(ManiaPerformanceAttributes::default());

        let _ = Performance::new(DifficultyAttributes::Osu(OsuDifficultyAttributes::default()));
        let _ = Performance::new(PerformanceAttributes::Taiko(
            TaikoPerformanceAttributes::default(),
        ));

        let _ = Performance::from(&map);
        let _ = Performance::from(map);

        let _ = Performance::from(OsuDifficultyAttributes::default());
        let _ = Performance::from(TaikoDifficultyAttributes::default());
        let _ = Performance::from(CatchDifficultyAttributes::default());
        let _ = Performance::from(ManiaDifficultyAttributes::default());

        let _ = Performance::from(OsuPerformanceAttributes::default());
        let _ = Performance::from(TaikoPerformanceAttributes::default());
        let _ = Performance::from(CatchPerformanceAttributes::default());
        let _ = Performance::from(ManiaPerformanceAttributes::default());

        let _ = Performance::from(DifficultyAttributes::Osu(OsuDifficultyAttributes::default()));
        let _ = Performance::from(PerformanceAttributes::Taiko(
            TaikoPerformanceAttributes::default(),
        ));

        let _ = DifficultyAttributes::Osu(OsuDifficultyAttributes::default()).performance();
        let _ = PerformanceAttributes::Taiko(TaikoPerformanceAttributes::default()).performance();
    }
}
