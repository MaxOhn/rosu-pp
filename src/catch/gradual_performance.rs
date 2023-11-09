use crate::{Beatmap, CatchPP};

use super::{CatchGradualDifficulty, CatchPerformanceAttributes, CatchScoreState};

/// Gradually calculate the performance attributes of an osu!catch map.
///
/// After each hit object you can call [`next`](`CatchGradualPerformance::next`)
/// and it will return the resulting current [`CatchPerformanceAttributes`].
/// To process multiple objects at once, use [`nth`](`CatchGradualPerformance::nth`) instead.
///
/// Both methods require a [`CatchScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// Note that neither hits nor misses of tiny droplets require
/// to be processed. Only fruits and droplets do.
///
/// If you only want to calculate difficulty attributes use [`CatchGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, catch::{CatchGradualPerformance, CatchScoreState}};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = CatchGradualPerformance::new(&map, mods);
/// let mut state = CatchScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are only fruits
/// for _ in 0..10 {
///     state.n_fruits += 1;
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
/// // The next 10 objects will be a mixture of fruits and droplets.
/// // Notice how tiny droplets from sliders do not count as hit objects
/// // that require processing. Only fruits and droplets do.
/// // Also notice how all 10 objects will be processed in one go.
/// state.n_fruits += 4;
/// state.n_droplets += 6;
/// state.n_tiny_droplets += 12;
/// // The `nth` method takes a zero-based value.
/// # /*
/// let performance = gradual_perf.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.nth(state.clone(), 9);
///
/// // Now comes another fruit. Note that the max combo gets incremented again.
/// state.n_fruits += 1;
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
/// state.n_fruits = ...
/// state.n_droplets = ...
/// state.n_tiny_droplets = ...
/// state.n_tiny_droplet_misses = ...
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
pub struct CatchGradualPerformance<'map> {
    difficulty: CatchGradualDifficulty<'map>,
    performance: CatchPP<'map>,
}

impl<'map> CatchGradualPerformance<'map> {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = CatchGradualDifficulty::new(map, mods);
        let performance = CatchPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that neither hits nor misses of tiny droplets require
    /// to be processed. Only fruits and droplets do.
    pub fn next(&mut self, state: CatchScoreState) -> Option<CatchPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance attributes.
    pub fn last(&mut self, state: CatchScoreState) -> Option<CatchPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the performance
    /// attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object, `n=1` will process 2,
    /// and so on.
    pub fn nth(&mut self, state: CatchScoreState, n: usize) -> Option<CatchPerformanceAttributes> {
        let difficulty = self.difficulty.by_ref().take(n.saturating_add(1)).last()?;

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
