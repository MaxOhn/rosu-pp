use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

use crate::{
    model::{beatmap::Converted, mode::IGameMode},
    util::generic_fmt::GenericFormatter,
    Difficulty,
};

/// Difficulty calculator on maps of a given mode.
///
/// # Example
///
/// ```
/// use akatsuki_pp::{Beatmap, Difficulty};
/// use akatsuki_pp::catch::{Catch, CatchDifficultyAttributes};
///
/// let converted = Beatmap::from_path("./resources/2118524.osu")
///     .unwrap()
///     .unchecked_into_converted();
///
/// let attrs: CatchDifficultyAttributes = Difficulty::new()
///     .mods(8 + 1024) // HDFL
///     .with_mode::<Catch>() // -> `ConvertedDifficulty`
///     .calculate(&converted);
/// ```
#[must_use]
pub struct ConvertedDifficulty<'a, M> {
    inner: &'a Difficulty,
    _mode: PhantomData<M>,
}

impl<'a, M> ConvertedDifficulty<'a, M> {
    pub(crate) const fn new(difficulty: &'a Difficulty) -> Self {
        Self {
            inner: difficulty,
            _mode: PhantomData,
        }
    }

    /// Return the internal [`Difficulty`].
    pub const fn inner(self) -> &'a Difficulty {
        self.inner
    }

    /// Cast from generic mode `M` to `N`.
    pub const fn cast<N: IGameMode>(self) -> ConvertedDifficulty<'a, N> {
        ConvertedDifficulty {
            inner: self.inner,
            _mode: PhantomData,
        }
    }
}

impl<M: IGameMode> ConvertedDifficulty<'_, M> {
    /// Perform the difficulty calculation for a [`Converted`] beatmap and
    /// process the final skill values.
    pub fn calculate(self, map: &Converted<'_, M>) -> M::DifficultyAttributes {
        M::difficulty(self.inner, map)
    }

    /// Perform a difficulty calculation for a [`Converted`] beatmap without
    /// processing the final skill values.
    pub fn strains(self, map: &Converted<'_, M>) -> M::Strains {
        M::strains(self.inner, map)
    }

    /// Create a gradual difficulty calculator for a [`Converted`] beatmap.
    pub fn gradual_difficulty(self, map: &Converted<'_, M>) -> M::GradualDifficulty {
        M::gradual_difficulty(self.inner.to_owned(), map)
    }

    /// Create a gradual performance calculator for a [`Converted`] beatmap.
    pub fn gradual_performance(self, map: &Converted<'_, M>) -> M::GradualPerformance {
        M::gradual_performance(self.inner.to_owned(), map)
    }
}

impl<'a, M: IGameMode> From<&'a Difficulty> for ConvertedDifficulty<'a, M> {
    fn from(difficulty: &'a Difficulty) -> Self {
        Self::new(difficulty)
    }
}

impl<M> AsRef<Difficulty> for ConvertedDifficulty<'_, M> {
    fn as_ref(&self) -> &Difficulty {
        self.inner
    }
}

impl<M> Copy for ConvertedDifficulty<'_, M> {}

impl<M> Clone for ConvertedDifficulty<'_, M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M> Debug for ConvertedDifficulty<'_, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ConvertedDifficulty")
            .field("inner", self.inner)
            .field("mode", &GenericFormatter::<M>::new())
            .finish()
    }
}

impl<M> PartialEq for ConvertedDifficulty<'_, M> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
