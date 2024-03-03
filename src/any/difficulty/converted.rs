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
#[must_use]
pub struct ConvertedDifficulty<M> {
    inner: Difficulty,
    _mode: PhantomData<M>,
}

impl<M> ConvertedDifficulty<M> {
    /// Create a new difficulty calculator for a generic mode.
    pub const fn new() -> Self {
        Self::from_difficulty(Difficulty::new())
    }

    pub(crate) const fn from_difficulty(difficulty: Difficulty) -> Self {
        Self {
            inner: difficulty,
            _mode: PhantomData,
        }
    }

    /// Return the internal [`Difficulty`].
    pub const fn into_inner(self) -> Difficulty {
        self.inner
    }

    /// Cast from generic mode `M` to `N`.
    pub fn cast<N: IGameMode>(self) -> ConvertedDifficulty<N> {
        ConvertedDifficulty {
            inner: self.inner,
            _mode: PhantomData,
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    pub const fn mods(self, mods: u32) -> Self {
        Self {
            inner: self.inner.mods(mods),
            ..self
        }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    pub const fn passed_objects(self, passed_objects: u32) -> Self {
        Self {
            inner: self.inner.passed_objects(passed_objects),
            ..self
        }
    }

    /// Adjust the clock rate used in the calculation between 0.01 and 100.0.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    pub fn clock_rate(self, clock_rate: f64) -> Self {
        Self {
            inner: self.inner.clock_rate(clock_rate),
            ..self
        }
    }
}

impl<M: IGameMode> ConvertedDifficulty<M> {
    /// Perform the difficulty calculation for a [`Converted`] beatmap and
    /// process the final skill values.
    pub fn calculate(&self, map: &Converted<'_, M>) -> M::DifficultyAttributes {
        M::difficulty(&self.inner, map)
    }

    /// Perform a difficulty calculation for a [`Converted`] beatmap without
    /// processing the final skill values.
    pub fn strains(&self, map: &Converted<'_, M>) -> M::Strains {
        M::strains(&self.inner, map)
    }

    /// Create a gradual difficulty calculator for a [`Converted`] beatmap.
    pub fn gradual_difficulty(&self, map: &Converted<'_, M>) -> M::GradualDifficulty {
        M::gradual_difficulty(&self.inner, map)
    }

    /// Create a gradual performance calculator for a [`Converted`] beatmap.
    pub fn gradual_performance(&self, map: &Converted<'_, M>) -> M::GradualPerformance {
        M::gradual_performance(&self.inner, map)
    }
}

impl<M: IGameMode> From<Difficulty> for ConvertedDifficulty<M> {
    fn from(difficulty: Difficulty) -> Self {
        Self::from_difficulty(difficulty)
    }
}

impl<M> AsRef<Difficulty> for ConvertedDifficulty<M> {
    fn as_ref(&self) -> &Difficulty {
        &self.inner
    }
}

impl<M> Clone for ConvertedDifficulty<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _mode: PhantomData,
        }
    }
}

impl<M> Debug for ConvertedDifficulty<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ConvertedDifficulty")
            .field("inner", &self.inner)
            .field("mode", &GenericFormatter::<M>::new())
            .finish()
    }
}

impl<M> PartialEq for ConvertedDifficulty<M> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
