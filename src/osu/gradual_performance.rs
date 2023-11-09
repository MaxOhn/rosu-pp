#![cfg(feature = "gradual")]

use crate::{Beatmap, OsuPP};

use super::{OsuGradualDifficulty, OsuPerformanceAttributes, OsuScoreState};

/// Gradually calculate the performance attributes of an osu!standard map.
///
/// After each hit object you can call [`next`](`OsuGradualPerformance::next`)
/// and it will return the resulting current [`OsuPerformanceAttributes`].
/// To process multiple objects at once, use [`nth`](`OsuGradualPerformance::nth`) instead.
///
/// Both methods require an [`OsuScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`OsuGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, osu::{OsuGradualPerformance, OsuScoreState}};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = OsuGradualPerformance::new(&map, mods);
/// let mut state = OsuScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s and there are no sliders for additional combo
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     # /*
///     let performance = gradual_perf.next(state.clone()).unwrap();
///     println!("PP: {}", performance.pp);
///     # */
///     # let _ = gradual_perf.next(state.clone());
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.n_misses += 1;
/// # /*
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.next(state.clone());
///
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
/// // The `nth` method takes a zero-based value.
/// # /*
/// let performance = gradual_perf.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.nth(state.clone(), 9);
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
/// # /*
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.next(state.clone());
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
/// state.n50 = ...
/// state.n_misses = ...
/// let final_performance = gradual_perf.nth(state.clone(), usize::MAX).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.nth(state.clone(), usize::MAX);
///
/// // Once the final performance was calculated,
/// // attempting to process further objects will return `None`.
/// assert!(gradual_perf.next(state).is_none());
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "gradual")))]
#[derive(Debug)]
pub struct OsuGradualPerformance<'map> {
    difficulty: OsuGradualDifficulty,
    performance: OsuPP<'map>,
}

impl<'map> OsuGradualPerformance<'map> {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = OsuGradualDifficulty::new(map, mods);
        let performance = OsuPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score state.
    pub fn next(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance attributes.
    pub fn last(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the performance
    /// attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object, `n=1` will process 2,
    /// and so on.
    pub fn nth(&mut self, state: OsuScoreState, n: usize) -> Option<OsuPerformanceAttributes> {
        let difficulty = self.difficulty.nth(n)?;

        let performance = self
            .performance
            .clone()
            .attributes(difficulty)
            .state(state)
            .passed_objects(self.difficulty.idx)
            .calculate();

        Some(performance)
    }
}

/// Gradually calculate the performance attributes of an osu!standard map.
///
/// Check [`OsuGradualPerformance`] for more information. This struct does the same
/// but takes ownership of [`Beatmap`] to avoid being bound to a lifetime.
#[cfg_attr(docsrs, doc(cfg(feature = "gradual")))]
#[derive(Debug)]
pub struct OsuOwnedGradualPerformance {
    difficulty: OsuGradualDifficulty,
    map: Beatmap,
    mods: u32,
}

impl OsuOwnedGradualPerformance {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(map: Beatmap, mods: u32) -> Self {
        let difficulty = OsuGradualDifficulty::new(&map, mods);

        Self {
            difficulty,
            map,
            mods,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score state.
    pub fn next(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance attributes.
    pub fn last(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the performance
    /// attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object, `n=1` will process 2,
    /// and so on.
    pub fn nth(&mut self, state: OsuScoreState, n: usize) -> Option<OsuPerformanceAttributes> {
        let difficulty = self.difficulty.nth(n)?;

        let performance = OsuPP::new(&self.map)
            .mods(self.mods)
            .attributes(difficulty)
            .state(state)
            .passed_objects(self.difficulty.idx)
            .calculate();

        Some(performance)
    }
}
