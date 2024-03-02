use std::borrow::Cow;

use rosu_map::section::general::GameMode;

use crate::{
    catch::{Catch, CatchBeatmap},
    mania::{Mania, ManiaBeatmap},
    model::beatmap::{Beatmap, Converted},
    osu::{Osu, OsuBeatmap},
    taiko::{Taiko, TaikoBeatmap},
};

use self::mode::ModeDifficulty;

use super::{attributes::DifficultyAttributes, Strains};

pub mod gradual;
pub mod mode;
pub mod object;
pub mod skills;

/// Difficulty calculator on maps of any mode.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct Difficulty<'map> {
    map: Cow<'map, Beatmap>,
    inner: ModeDifficulty,
}

impl<'map> Difficulty<'map> {
    /// Create a new difficulty calculator for the given beatmap.
    pub const fn new(map: &'map Beatmap) -> Self {
        Self::new_with_cow(Cow::Borrowed(map))
    }

    const fn new_with_cow(map: Cow<'map, Beatmap>) -> Self {
        Self {
            map,
            inner: ModeDifficulty::new(),
        }
    }
}

macro_rules! impl_from_mode {
    ( $mode:ident ) => {
        impl<'a> From<Converted<'a, $mode>> for Difficulty<'a> {
            fn from(converted: Converted<'a, $mode>) -> Self {
                Self::new_with_cow(converted.into_inner())
            }
        }

        impl<'a, 'b: 'a> From<&'b Converted<'a, $mode>> for Difficulty<'a> {
            fn from(converted: &'b Converted<'a, $mode>) -> Self {
                Self::new(&converted)
            }
        }
    };
}

impl_from_mode!(Osu);
impl_from_mode!(Taiko);
impl_from_mode!(Catch);
impl_from_mode!(Mania);

impl Difficulty<'_> {
    /// Attempt to convert the map to the specified mode.
    ///
    /// If the conversion is incompatible, `None` is returned.
    ///
    /// If the given mode should be ignored in case it is incompatible, use
    /// [`mode_or_ignore`] instead.
    ///
    /// [`mode_or_ignore`]: Self::mode_or_ignore
    pub fn try_mode(&mut self, mode: GameMode) -> Option<&mut Self> {
        let map = match mode {
            GameMode::Osu => OsuBeatmap::try_from_ref(self.map.as_ref())?.into_inner(),
            GameMode::Taiko => TaikoBeatmap::try_from_ref(self.map.as_ref())?.into_inner(),
            GameMode::Catch => CatchBeatmap::try_from_ref(self.map.as_ref())?.into_inner(),
            GameMode::Mania => ManiaBeatmap::try_from_ref(self.map.as_ref())?.into_inner(),
        };

        if matches!(map, Cow::Owned(_)) {
            let map = map.into_owned();
            self.map = Cow::Owned(map);
        }

        Some(self)
    }

    /// Attempt to convert the map to the specified mode.
    ///
    /// If the conversion is incompatible, the map won't be modified.
    ///
    /// To see whether the given mode is incompatible, use [`try_mode`]
    /// instead.
    ///
    /// [`try_mode`]: Self::try_mode
    pub fn mode_or_ignore(&mut self, mode: GameMode) -> &mut Self {
        let map = self.map.as_ref();

        let map_opt = match mode {
            GameMode::Osu => OsuBeatmap::try_from_ref(map).map(Converted::into_inner),
            GameMode::Taiko => TaikoBeatmap::try_from_ref(map).map(Converted::into_inner),
            GameMode::Catch => CatchBeatmap::try_from_ref(map).map(Converted::into_inner),
            GameMode::Mania => ManiaBeatmap::try_from_ref(map).map(Converted::into_inner),
        };

        match map_opt {
            Some(cow @ Cow::Owned(_)) => {
                let map = cow.into_owned();
                self.map = Cow::Owned(map);
            }
            Some(Cow::Borrowed(_)) | None => {}
        }

        self
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    pub fn mods(self, mods: u32) -> Self {
        Self {
            inner: self.inner.mods(mods),
            ..self
        }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    pub fn passed_objects(self, passed_objects: u32) -> Self {
        Self {
            inner: self.inner.passed_objects(passed_objects),
            ..self
        }
    }

    /// Adjust the clock rate used in the calculation.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        Self {
            inner: self.inner.clock_rate(clock_rate),
            ..self
        }
    }

    /// Perform the difficulty calculation.
    ///
    /// The returned attributes depend on the map's mode.
    pub fn calculate(&self) -> DifficultyAttributes {
        let map = Cow::Borrowed(self.map.as_ref());

        match self.map.mode {
            GameMode::Osu => DifficultyAttributes::Osu(self.inner.calculate(&OsuBeatmap::new(map))),
            GameMode::Taiko => {
                DifficultyAttributes::Taiko(self.inner.calculate(&TaikoBeatmap::new(map)))
            }
            GameMode::Catch => {
                DifficultyAttributes::Catch(self.inner.calculate(&CatchBeatmap::new(map)))
            }
            GameMode::Mania => {
                DifficultyAttributes::Mania(self.inner.calculate(&ManiaBeatmap::new(map)))
            }
        }
    }

    /// Perform the difficulty calculation but instead of evaluating the skill
    /// strains, return them as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    pub fn strains(&self) -> Strains {
        let map = Cow::Borrowed(self.map.as_ref());

        match self.map.mode {
            GameMode::Osu => Strains::Osu(self.inner.strains(&OsuBeatmap::new(map))),
            GameMode::Taiko => Strains::Taiko(self.inner.strains(&TaikoBeatmap::new(map))),
            GameMode::Catch => Strains::Catch(self.inner.strains(&CatchBeatmap::new(map))),
            GameMode::Mania => Strains::Mania(self.inner.strains(&ManiaBeatmap::new(map))),
        }
    }
}
