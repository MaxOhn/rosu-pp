#![cfg(feature = "gradual")]

use crate::{taiko::TaikoScoreState, Beatmap, TaikoPP};

use super::{TaikoGradualDifficultyAttributes, TaikoPerformanceAttributes};

/// Gradually calculate the performance attributes of an osu!taiko map.
///
/// After each hit object you can call [`next`](`TaikoGradualPerformanceAttributes::next`)
/// and it will return the resulting current [`TaikoPerformanceAttributes`].
/// To process multiple objects at once, use
/// [`nth`](`TaikoGradualPerformanceAttributes::nth`) instead.
///
/// Both methods require a [`TaikoScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`TaikoGradualDifficultyAttributes`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, taiko::{TaikoGradualPerformanceAttributes, TaikoScoreState}};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = TaikoGradualPerformanceAttributes::new(&map, mods);
/// let mut state = TaikoScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s
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
/// // The next 10 objects will be a mixture of 300s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 3;
/// state.n100 += 7;
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
#[derive(Debug)]
pub struct TaikoGradualPerformanceAttributes<'map> {
    difficulty: TaikoGradualDifficultyAttributes,
    performance: TaikoPP<'map>,
}

impl<'map> TaikoGradualPerformanceAttributes<'map> {
    /// Create a new gradual performance calculator for osu!taiko maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = TaikoGradualDifficultyAttributes::new(map, mods);
        let performance = TaikoPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score.
    pub fn next(&mut self, state: TaikoScoreState) -> Option<TaikoPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process everything up the the next `n`th hit object and calculate the performance
    /// attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object, `n=1` will process 2,
    /// and so on.
    pub fn nth(&mut self, state: TaikoScoreState, n: usize) -> Option<TaikoPerformanceAttributes> {
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
