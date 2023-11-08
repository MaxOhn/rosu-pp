#![cfg(feature = "gradual")]

use crate::{
    catch::{CatchGradualDifficultyAttributes, CatchGradualPerformanceAttributes},
    mania::{ManiaGradualDifficultyAttributes, ManiaGradualPerformanceAttributes},
    osu::{OsuGradualDifficultyAttributes, OsuGradualPerformanceAttributes},
    taiko::{TaikoGradualDifficultyAttributes, TaikoGradualPerformanceAttributes},
    Beatmap, DifficultyAttributes, GameMode, PerformanceAttributes, ScoreState,
};

/// Gradually calculate the difficulty attributes on maps of any mode.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`](Iterator::next), the map's next hit object will
/// be processed and the [`DifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use [`GradualPerformanceAttributes`] instead.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{Beatmap, GradualDifficultyAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = GradualDifficultyAttributes::new(&map, mods);
///
/// let attrs1 = iter.next(); // the difficulty of the map after the first hit object
/// let attrs2 = iter.next(); //                           after the second hit object
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
#[derive(Debug)]
pub enum GradualDifficultyAttributes<'map> {
    /// Gradual osu!standard difficulty attributes.
    Osu(OsuGradualDifficultyAttributes),
    /// Gradual osu!taiko difficulty attributes.
    Taiko(TaikoGradualDifficultyAttributes),
    /// Gradual osu!catch difficulty attributes.
    Catch(CatchGradualDifficultyAttributes<'map>),
    /// Gradual osu!mania difficulty attributes.
    Mania(ManiaGradualDifficultyAttributes<'map>),
}

impl<'map> GradualDifficultyAttributes<'map> {
    // FIXME: converted catch maps will always count as osu!std since their mode is not modified
    /// Create a new gradual difficulty calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualDifficultyAttributes::new(map, mods)),
            GameMode::Taiko => Self::Taiko(TaikoGradualDifficultyAttributes::new(map, mods)),
            GameMode::Catch => Self::Catch(CatchGradualDifficultyAttributes::new(map, mods)),
            GameMode::Mania => Self::Mania(ManiaGradualDifficultyAttributes::new(map, mods)),
        }
    }
}

impl Iterator for GradualDifficultyAttributes<'_> {
    type Item = DifficultyAttributes;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GradualDifficultyAttributes::Osu(o) => o.next().map(DifficultyAttributes::Osu),
            GradualDifficultyAttributes::Taiko(t) => t.next().map(DifficultyAttributes::Taiko),
            GradualDifficultyAttributes::Catch(f) => f.next().map(DifficultyAttributes::Catch),
            GradualDifficultyAttributes::Mania(m) => m.next().map(DifficultyAttributes::Mania),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GradualDifficultyAttributes::Osu(o) => o.size_hint(),
            GradualDifficultyAttributes::Taiko(t) => t.size_hint(),
            GradualDifficultyAttributes::Catch(f) => f.size_hint(),
            GradualDifficultyAttributes::Mania(m) => m.size_hint(),
        }
    }
}

/// Gradually calculate the performance attributes on maps of any mode.
///
/// After each hit object you can call [`next`](`GradualPerformanceAttributes::next`)
/// and it will return the resulting current [`PerformanceAttributes`].
/// To process multiple objects at once, use
/// [`nth`](`GradualPerformanceAttributes::nth`) instead.
///
/// Both methods require a [`ScoreState`] that contains the current hitresults
/// as well as the maximum combo so far or just the current score for osu!mania.
/// Since the map could have any mode, all fields of `ScoreState` could be of use
/// and should be updated properly.
///
/// Alternatively, you can match on the map's mode yourself and use the gradual
/// performance attribute struct for the corresponding mode, i.e.
/// [`OsuGradualPerformanceAttributes`], [`TaikoGradualPerformanceAttributes`],
/// [`CatchGradualPerformanceAttributes`], or [`ManiaGradualPerformanceAttributes`].
///
/// If you only want to calculate difficulty attributes use [`GradualDifficultyAttributes`] instead.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{Beatmap, GradualPerformanceAttributes, ScoreState};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = GradualPerformanceAttributes::new(&map, mods);
/// let mut state = ScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     let performance = gradual_perf.next(state.clone()).unwrap();
///     println!("PP: {}", performance.pp());
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.n_misses += 1;
///
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
///
/// // The `nth` method takes a zero-based value.
/// let performance = gradual_perf.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
///
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// ...
/// # */
/// let final_performance = gradual_perf.last(state.clone()).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // Once the final performance was calculated,
/// // attempting to process further objects will return `None`.
/// assert!(gradual_perf.next(state).is_none());
/// ```
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum GradualPerformanceAttributes<'map> {
    /// Gradual osu!standard performance attributes.
    Osu(OsuGradualPerformanceAttributes<'map>),
    /// Gradual osu!taiko performance attributes.
    Taiko(TaikoGradualPerformanceAttributes<'map>),
    /// Gradual osu!catch performance attributes.
    Catch(CatchGradualPerformanceAttributes<'map>),
    /// Gradual osu!mania performance attributes.
    Mania(ManiaGradualPerformanceAttributes<'map>),
}

impl<'map> GradualPerformanceAttributes<'map> {
    // FIXME: converted catch maps will always count as osu!std since their mode is not modified
    /// Create a new gradual performance calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualPerformanceAttributes::new(map, mods)),
            GameMode::Taiko => Self::Taiko(TaikoGradualPerformanceAttributes::new(map, mods)),
            GameMode::Catch => Self::Catch(CatchGradualPerformanceAttributes::new(map, mods)),
            GameMode::Mania => Self::Mania(ManiaGradualPerformanceAttributes::new(map, mods)),
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score.
    #[inline]
    pub fn next(&mut self, state: ScoreState) -> Option<PerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance attributes.
    pub fn last(&mut self, state: ScoreState) -> Option<PerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the performance
    /// attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object, `n=1` will process 2,
    /// and so on.
    #[inline]
    pub fn nth(&mut self, state: ScoreState, n: usize) -> Option<PerformanceAttributes> {
        match self {
            GradualPerformanceAttributes::Osu(o) => o.nth(state.into(), n).map(From::from),
            GradualPerformanceAttributes::Taiko(t) => t.nth(state.into(), n).map(From::from),
            GradualPerformanceAttributes::Catch(f) => f.nth(state.into(), n).map(From::from),
            GradualPerformanceAttributes::Mania(m) => m.nth(state.into(), n).map(From::from),
        }
    }
}
