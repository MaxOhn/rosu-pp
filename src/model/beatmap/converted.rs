use std::{
    borrow::Cow,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ops::Deref,
};

use crate::{
    model::mode::{ConvertStatus, IGameMode},
    util::generic_fmt::GenericFormatter,
    Difficulty,
};

use super::Beatmap;

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
    map: Cow<'a, Beatmap>,
    mode: PhantomData<M>,
}

impl<'a, M> Converted<'a, M> {
    /// Initialize a [`Converted`] beatmap by promising the given map's mode
    /// matches the generic type `M`.
    pub(crate) const fn new(map: Cow<'a, Beatmap>) -> Self {
        Self {
            map,
            mode: PhantomData,
        }
    }

    /// Returns the internal [`Beatmap`].
    pub fn into_inner(self) -> Cow<'a, Beatmap> {
        self.map
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
            ConvertStatus::Noop => Ok(Self::new(Cow::Owned(map))),
            ConvertStatus::Conversion => Ok(Self::new(Cow::Owned(map))),
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
    pub fn gradual_difficulty(&self, difficulty: &Difficulty) -> M::GradualDifficulty {
        M::gradual_difficulty(difficulty, self)
    }

    /// Create a gradual performance calculator for the map.
    pub fn gradual_performance(&self, difficulty: &Difficulty) -> M::GradualPerformance {
        M::gradual_performance(difficulty, self)
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
        Self::new(Cow::Borrowed(self.map.as_ref()))
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
            ConvertStatus::Noop => return Some(Self::new(Cow::Borrowed(map))),
            ConvertStatus::Conversion => map.to_owned(),
            ConvertStatus::Incompatible => return None,
        };

        match M::try_convert(&mut map) {
            ConvertStatus::Conversion => Some(Self::new(Cow::Owned(map))),
            ConvertStatus::Noop => Some(Self::new(Cow::Owned(map))),
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

    /// Attempt to convert a [`&mut Beatmap`] to the specified mode.
    ///
    /// If the conversion is incompatible, `None` is returned.
    ///
    /// [`&mut Beatmap`]: Beatmap
    pub fn try_from_mut(map: &'a mut Beatmap) -> Option<Self> {
        match M::try_convert(map) {
            ConvertStatus::Conversion => Some(Self::new(Cow::Borrowed(map))),
            ConvertStatus::Noop => Some(Self::new(Cow::Borrowed(map))),
            ConvertStatus::Incompatible => None,
        }
    }

    /// Convert a [`&mut Beatmap`] to the specified mode.
    ///
    /// # Panics
    ///
    /// Panics if the conversion is incompatible.
    ///
    /// [`&mut Beatmap`]: Beatmap
    pub fn unchecked_from_mut(map: &'a mut Beatmap) -> Self {
        Self::try_from_mut(map).expect(INCOMPATIBLE_MODES)
    }

    /// Attempt to convert a [`Converted`] from mode `M` to mode `N`.
    ///
    /// If the conversion is incompatible the [`Converted`] will be returned
    /// unchanged as `Err`.
    #[allow(clippy::result_large_err)]
    pub fn try_convert<N: IGameMode>(self) -> Result<Converted<'a, N>, Self> {
        match self.map {
            Cow::Borrowed(map) => Converted::<N>::try_from_ref(map).ok_or(self),
            Cow::Owned(map) => {
                Converted::<N>::try_from_owned(map).map_err(|map| Self::new(Cow::Owned(map)))
            }
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
        Self::new(self.map.clone())
    }
}

impl<M> Debug for Converted<'_, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Converted")
            .field("map", &self.map)
            .field("mode", &GenericFormatter::<M>::new())
            .finish()
    }
}

impl<M> PartialEq for Converted<'_, M> {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl<M> Deref for Converted<'_, M> {
    type Target = Beatmap;

    fn deref(&self) -> &Self::Target {
        self.map.as_ref()
    }
}
