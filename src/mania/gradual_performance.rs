#![cfg(feature = "gradual")]

use crate::{Beatmap, ManiaPP};

use super::{ManiaGradualDifficulty, ManiaPerformanceAttributes, ManiaScoreState};

/// Gradually calculate the performance attributes of an osu!mania map.
///
/// After each hit object you can call [`next`](`ManiaGradualPerformance::next`)
/// and it will return the resulting current [`ManiaPerformanceAttributes`].
/// To process multiple objects at once, use [`nth`](`ManiaGradualPerformance::nth`) instead.
///
/// Both methods require a play's current score so far.
/// Be sure the given score is adjusted with respect to mods.
///
/// If you only want to calculate difficulty attributes use
/// [`ManiaGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, mania::{ManiaGradualPerformance, ManiaScoreState}};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = ManiaGradualPerformance::new(&map, mods);
/// let mut state = ManiaScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 320s
/// for _ in 0..10 {
///     state.n320 += 1;
///
///     # /*
///     let performance = gradual_perf.next(score).unwrap();
///     println!("PP: {}", performance.pp);
///     # */
///     # let _ = gradual_perf.next(state.clone());
/// }
///
/// // Then comes a miss.
/// state.n_misses += 1;
/// # /*
/// let performance = gradual_perf.next(score).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.next(state.clone());
///
/// // The next 10 objects will be a mixture of 320s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n320 += 3;
/// state.n100 += 7;
/// // The `nth` method takes a zero-based value.
/// # /*
/// let performance = gradual_perf.nth(score, 9).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.nth(state.clone(), 9);
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
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
#[derive(Clone, Debug)]
pub struct ManiaGradualPerformance<'map> {
    difficulty: ManiaGradualDifficulty<'map>,
    performance: ManiaPP<'map>,
}

impl<'map> ManiaGradualPerformance<'map> {
    /// Create a new gradual performance calculator for osu!mania maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = ManiaGradualDifficulty::new(map, mods);
        let performance = ManiaPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score.
    pub fn next(&mut self, state: ManiaScoreState) -> Option<ManiaPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance attributes.
    pub fn last(&mut self, state: ManiaScoreState) -> Option<ManiaPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the performance
    /// attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object, `n=1` will process 2,
    /// and so on.
    pub fn nth(&mut self, state: ManiaScoreState, n: usize) -> Option<ManiaPerformanceAttributes> {
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
