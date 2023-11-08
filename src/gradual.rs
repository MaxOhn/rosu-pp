#![cfg(feature = "gradual")]

use crate::{
    catch::{CatchGradualDifficulty, CatchGradualPerformance},
    mania::{ManiaGradualDifficulty, ManiaGradualPerformance},
    osu::{OsuGradualDifficulty, OsuGradualPerformance},
    taiko::{TaikoGradualDifficulty, TaikoGradualPerformance},
    Beatmap, DifficultyAttributes, GameMode, PerformanceAttributes, ScoreState,
};

/// Gradually calculate the difficulty attributes on maps of any mode.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`](Iterator::next), the map's next hit object will
/// be processed and the [`DifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use [`GradualPerformance`] instead.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{Beatmap, GradualDifficulty};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = GradualDifficulty::new(&map, mods);
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
pub enum GradualDifficulty<'map> {
    /// Gradual osu!standard difficulty attributes.
    Osu(OsuGradualDifficulty),
    /// Gradual osu!taiko difficulty attributes.
    Taiko(TaikoGradualDifficulty),
    /// Gradual osu!catch difficulty attributes.
    Catch(CatchGradualDifficulty<'map>),
    /// Gradual osu!mania difficulty attributes.
    Mania(ManiaGradualDifficulty<'map>),
}

impl<'map> GradualDifficulty<'map> {
    // FIXME: converted catch maps will always count as osu!std since their mode is not modified
    /// Create a new gradual difficulty calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualDifficulty::new(map, mods)),
            GameMode::Taiko => Self::Taiko(TaikoGradualDifficulty::new(map, mods)),
            GameMode::Catch => Self::Catch(CatchGradualDifficulty::new(map, mods)),
            GameMode::Mania => Self::Mania(ManiaGradualDifficulty::new(map, mods)),
        }
    }
}

impl Iterator for GradualDifficulty<'_> {
    type Item = DifficultyAttributes;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GradualDifficulty::Osu(o) => o.next().map(DifficultyAttributes::Osu),
            GradualDifficulty::Taiko(t) => t.next().map(DifficultyAttributes::Taiko),
            GradualDifficulty::Catch(f) => f.next().map(DifficultyAttributes::Catch),
            GradualDifficulty::Mania(m) => m.next().map(DifficultyAttributes::Mania),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GradualDifficulty::Osu(o) => o.size_hint(),
            GradualDifficulty::Taiko(t) => t.size_hint(),
            GradualDifficulty::Catch(f) => f.size_hint(),
            GradualDifficulty::Mania(m) => m.size_hint(),
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
/// [`OsuGradualPerformance`], [`TaikoGradualPerformance`],
/// [`CatchGradualPerformance`], or [`ManiaGradualPerformance`].
///
/// If you only want to calculate difficulty attributes use [`GradualDifficulty`] instead.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{Beatmap, GradualPerformance, ScoreState};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = GradualPerformance::new(&map, mods);
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
pub enum GradualPerformance<'map> {
    /// Gradual osu!standard performance attributes.
    Osu(OsuGradualPerformance<'map>),
    /// Gradual osu!taiko performance attributes.
    Taiko(TaikoGradualPerformance<'map>),
    /// Gradual osu!catch performance attributes.
    Catch(CatchGradualPerformance<'map>),
    /// Gradual osu!mania performance attributes.
    Mania(ManiaGradualPerformance<'map>),
}

impl<'map> GradualPerformance<'map> {
    // FIXME: converted catch maps will always count as osu!std since their mode is not modified
    /// Create a new gradual performance calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualPerformance::new(map, mods)),
            GameMode::Taiko => Self::Taiko(TaikoGradualPerformance::new(map, mods)),
            GameMode::Catch => Self::Catch(CatchGradualPerformance::new(map, mods)),
            GameMode::Mania => Self::Mania(ManiaGradualPerformance::new(map, mods)),
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
            GradualPerformance::Osu(o) => o.nth(state.into(), n).map(From::from),
            GradualPerformance::Taiko(t) => t.nth(state.into(), n).map(From::from),
            GradualPerformance::Catch(f) => f.nth(state.into(), n).map(From::from),
            GradualPerformance::Mania(m) => m.nth(state.into(), n).map(From::from),
        }
    }
}
