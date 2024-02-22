use std::borrow::Cow;

use rosu_map::section::general::GameMode;

use crate::{
    catch::{Catch, CatchBeatmap},
    mania::{Mania, ManiaBeatmap},
    model::beatmap::{Beatmap, Converted},
    osu::{Osu, OsuBeatmap},
    taiko::{Taiko, TaikoBeatmap},
};

pub use self::mode::ModeDifficulty;

use super::attributes::DifficultyAttributes;

mod mode;
pub mod object;
pub mod skills;

/// Difficulty calculator on maps of any mode.
#[derive(Clone, Debug, PartialEq)]
pub struct Difficulty<'map> {
    map: Cow<'map, Beatmap>,
    is_convert: bool,
    inner: ModeDifficulty,
}

impl<'map> Difficulty<'map> {
    /// Create a new difficulty calculator for the given beatmap.
    pub fn new(map: &'map Beatmap) -> Self {
        Self::new_with_is_convert(Cow::Borrowed(map), false)
    }

    fn new_with_is_convert(map: Cow<'map, Beatmap>, is_convert: bool) -> Self {
        Self {
            map,
            is_convert,
            inner: ModeDifficulty::new(),
        }
    }
}

macro_rules! impl_from_mode {
    ( $mode:ident ) => {
        impl<'a> From<Converted<'a, $mode>> for Difficulty<'a> {
            fn from(converted: Converted<'a, $mode>) -> Self {
                Self::new_with_is_convert(converted.map, converted.is_convert)
            }
        }

        impl<'a, 'b: 'a> From<&'b Converted<'a, $mode>> for Difficulty<'a> {
            fn from(converted: &'b Converted<'a, $mode>) -> Self {
                Self::new_with_is_convert(
                    Cow::Borrowed(converted.map.as_ref()),
                    converted.is_convert,
                )
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
        let (map, is_convert) = match mode {
            GameMode::Osu => {
                let converted = OsuBeatmap::try_from_ref(self.map.as_ref())?;

                (converted.map, converted.is_convert)
            }
            GameMode::Taiko => {
                let converted = TaikoBeatmap::try_from_ref(self.map.as_ref())?;

                (converted.map, converted.is_convert)
            }
            GameMode::Catch => {
                let converted = CatchBeatmap::try_from_ref(self.map.as_ref())?;

                (converted.map, converted.is_convert)
            }
            GameMode::Mania => {
                let converted = ManiaBeatmap::try_from_ref(self.map.as_ref())?;

                (converted.map, converted.is_convert)
            }
        };

        if matches!(map, Cow::Owned(_)) {
            self.is_convert |= is_convert;
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
        let (map, is_convert) = match mode {
            GameMode::Osu => {
                let Some(converted) = OsuBeatmap::try_from_ref(self.map.as_ref()) else {
                    return self;
                };

                (converted.map, converted.is_convert)
            }
            GameMode::Taiko => {
                let Some(converted) = TaikoBeatmap::try_from_ref(self.map.as_ref()) else {
                    return self;
                };

                (converted.map, converted.is_convert)
            }
            GameMode::Catch => {
                let Some(converted) = CatchBeatmap::try_from_ref(self.map.as_ref()) else {
                    return self;
                };

                (converted.map, converted.is_convert)
            }
            GameMode::Mania => {
                let Some(converted) = ManiaBeatmap::try_from_ref(self.map.as_ref()) else {
                    return self;
                };

                (converted.map, converted.is_convert)
            }
        };

        if matches!(map, Cow::Owned(_)) {
            self.is_convert |= is_convert;
            let map = map.into_owned();
            self.map = Cow::Owned(map);
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
            GameMode::Osu => DifficultyAttributes::Osu(
                self.inner
                    .calculate(&Converted::<Osu>::new(map, self.is_convert)),
            ),
            GameMode::Taiko => DifficultyAttributes::Taiko(
                self.inner
                    .calculate(&Converted::<Taiko>::new(map, self.is_convert)),
            ),
            GameMode::Catch => DifficultyAttributes::Catch(
                self.inner
                    .calculate(&Converted::<Catch>::new(map, self.is_convert)),
            ),
            GameMode::Mania => DifficultyAttributes::Mania(
                self.inner
                    .calculate(&Converted::<Mania>::new(map, self.is_convert)),
            ),
        }
    }
}
