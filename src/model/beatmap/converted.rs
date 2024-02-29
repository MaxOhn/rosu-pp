use std::{
    any,
    borrow::Cow,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

use crate::{
    model::mode::{ConvertStatus, IGameMode},
    ModeDifficulty,
};

use super::{attributes::BeatmapAttributesBuilder, Beatmap};

const INCOMPATIBLE_MODES: &str = "the gamemodes were incompatible";

/// A [`Beatmap`] that is attached to a mode.
///
/// # Incompatibility
///
/// The following conversions are compatible:
/// - `Osu` → `Osu`
/// - `Taiko` → `Taiko`
/// - `Catch` → `Catch`
/// - `Mania` → `Mania`
/// - `Osu` → `Taiko`
/// - `Osu` → `Catch`
/// - `Osu` → `Mania`
///
/// All other conversions are incompatible.
pub struct Converted<'a, M> {
    pub(crate) map: Cow<'a, Beatmap>,
    pub(crate) is_convert: bool,
    mode: PhantomData<M>,
}

impl<'a, M> Converted<'a, M> {
    /// Initialize a [`Converted`] beatmap by promising the given map's mode
    /// matches the generic type `M`.
    pub(crate) const fn new(map: Cow<'a, Beatmap>, is_convert: bool) -> Self {
        Self {
            map,
            is_convert,
            mode: PhantomData,
        }
    }
}

impl<M> Converted<'_, M> {
    /// Sum up the duration of all breaks (in milliseconds).
    pub fn total_break_time(&self) -> f64 {
        self.map.total_break_time()
    }

    /// Returns a [`BeatmapAttributesBuilder`] to calculate modified beatmap
    /// attributes.
    pub fn attributes(&self) -> BeatmapAttributesBuilder {
        self.into()
    }

    /// The beats per minute of the map.
    pub fn bpm(&self) -> f64 {
        self.map.bpm()
    }
}

impl<M: IGameMode> Converted<'_, M> {
    /// Attempt to convert a [`Beatmap`] to the specified mode.
    ///
    /// If the conversion is incompatible the [`Beatmap`] will be returned
    /// unchanged as `Err`.
    #[allow(clippy::result_large_err)]
    pub fn try_from_owned(mut map: Beatmap) -> Result<Self, Beatmap> {
        match M::try_convert(&mut map) {
            ConvertStatus::Noop => Ok(Self::new(Cow::Owned(map), false)),
            ConvertStatus::Conversion => Ok(Self::new(Cow::Owned(map), true)),
            ConvertStatus::Incompatible => Err(map),
        }
    }

    /// Convert a [`Beatmap`] to the specified mode.
    ///
    /// # Panics
    ///
    /// Panics if the conversion is incompatible.
    pub fn unchecked_from_owned(map: Beatmap) -> Self {
        Self::try_from_owned(map).unwrap_or_else(|_| panic!("{}", INCOMPATIBLE_MODES))
    }

    /// Create a gradual difficulty calculator for the map.
    pub fn gradual_difficulty(&self, difficulty: &ModeDifficulty) -> M::GradualDifficulty {
        difficulty.gradual_difficulty(self)
    }

    /// Create a gradual performance calculator for the map.
    pub fn gradual_performance(&self, difficulty: &ModeDifficulty) -> M::GradualPerformance {
        difficulty.gradual_performance(self)
    }
}

impl<'a, M: IGameMode> Converted<'a, M> {
    /// Borrow the contained [`Beatmap`] to cheaply create a new owned
    /// [`Converted`].
    ///
    /// This is the same as `.clone()` except cheap - but its lifetime might be
    /// shorter.
    #[must_use]
    pub fn as_owned(&'a self) -> Self {
        Self::new(Cow::Borrowed(self.map.as_ref()), self.is_convert)
    }

    /// Create a performance calculator for the map.
    pub fn performance(self) -> M::Performance<'a> {
        M::performance(self)
    }

    /// Attempt to convert a [`&Beatmap`] to the specified mode.
    ///
    /// If the conversion is incompatible, `None` is returned.
    ///
    /// [`&Beatmap`]: Beatmap
    pub fn try_from_ref(map: &'a Beatmap) -> Option<Self> {
        let mut map = match M::check_convert(map) {
            ConvertStatus::Noop => return Some(Self::new(Cow::Borrowed(map), false)),
            ConvertStatus::Conversion => map.to_owned(),
            ConvertStatus::Incompatible => return None,
        };

        match M::try_convert(&mut map) {
            ConvertStatus::Conversion => Some(Self::new(Cow::Owned(map), true)),
            ConvertStatus::Noop => Some(Self::new(Cow::Owned(map), false)),
            ConvertStatus::Incompatible => None,
        }
    }

    /// Convert a [`&Beatmap`] to the specified mode.
    ///
    /// # Panics
    ///
    /// Panics if the conversion is incompatible.
    ///
    /// [`&Beatmap`]: Beatmap
    pub fn unchecked_from_ref(map: &'a Beatmap) -> Self {
        Self::try_from_ref(map).expect(INCOMPATIBLE_MODES)
    }

    /// Attempt to convert a [`Converted`] from mode `M` to mode `N`.
    ///
    /// If the conversion is incompatible the [`Converted`] will be returned
    /// unchanged as `Err`.
    #[allow(clippy::result_large_err)]
    pub fn try_convert<N: IGameMode>(self) -> Result<Converted<'a, N>, Self> {
        match self.map {
            Cow::Borrowed(map) => Converted::<N>::try_from_ref(map).ok_or(self),
            Cow::Owned(map) => Converted::<N>::try_from_owned(map)
                .map_err(|map| Self::new(Cow::Owned(map), self.is_convert)),
        }
    }

    /// Convert a [`Converted`] from mode `M` to mode `N`.
    ///
    /// # Panics
    ///
    /// Panics if the conversion is incompatible.
    pub fn unchecked_convert<N: IGameMode>(self) -> Converted<'a, N> {
        match self.map {
            Cow::Borrowed(map) => Converted::<N>::unchecked_from_ref(map),
            Cow::Owned(map) => Converted::<N>::unchecked_from_owned(map),
        }
    }
}

impl<M> Clone for Converted<'_, M> {
    fn clone(&self) -> Self {
        Self::new(self.map.clone(), self.is_convert)
    }
}

impl<M> Debug for Converted<'_, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        struct GenericFormatter<T>(PhantomData<T>);

        impl<T> Default for GenericFormatter<T> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl<T> Debug for GenericFormatter<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                let full_type_name = any::type_name::<T>();

                // Strip fully qualified syntax
                if let Some(position) = full_type_name.rfind("::") {
                    if let Some(type_name) = full_type_name.get(position + 2..) {
                        f.write_str(type_name)?;
                    }
                }

                Ok(())
            }
        }

        f.debug_struct("Converted")
            .field("map", &self.map)
            .field("is_convert", &self.is_convert)
            .field("mode", &GenericFormatter::<M>::default())
            .finish()
    }
}

impl<M> PartialEq for Converted<'_, M> {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map && self.is_convert == other.is_convert
    }
}
