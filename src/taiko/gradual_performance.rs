use crate::{Beatmap, TaikoPP};

use super::{TaikoGradualDifficultyAttributes, TaikoPerformanceAttributes};

/// Aggregation for a score's current state i.e. what was the
/// maximum combo so far and what are the current hitresults.
///
/// This struct is used for [`TaikoGradualPerformanceAttributes`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaikoScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    pub max_combo: usize,
    /// Amount of current 300s.
    pub n300: usize,
    /// Amount of current 100s.
    pub n100: usize,
    /// Amount of current misses.
    pub n_misses: usize,
}

impl TaikoScoreState {
    /// Create a new empty score state.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    #[inline]
    pub fn total_hits(&self) -> usize {
        self.n300 + self.n100 + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    #[inline]
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 2 * self.n300 + self.n100;
        let denominator = 2 * total_hits;

        numerator as f64 / denominator as f64
    }
}

/// Gradually calculate the performance attributes of an osu!taiko map.
///
/// After each hit object you can call
/// [`process_next_object`](`TaikoGradualPerformanceAttributes::process_next_object`)
/// and it will return the resulting current [`TaikoPerformanceAttributes`].
/// To process multiple objects at once, use
/// [`process_next_n_objects`](`TaikoGradualPerformanceAttributes::process_next_n_objects`) instead.
///
/// Both methods require a [`TaikoScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`TaikoGradualDifficultyAttributes`](crate::taiko::TaikoGradualDifficultyAttributes) instead.
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
///     let performance = gradual_perf.process_next_object(state.clone()).unwrap();
///     println!("PP: {}", performance.pp);
///     # */
///     # let _ = gradual_perf.process_next_object(state.clone());
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.n_misses += 1;
/// # /*
/// let performance = gradual_perf.process_next_object(state.clone()).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_object(state.clone());
///
/// // The next 10 objects will be a mixture of 300s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 3;
/// state.n100 += 7;
/// # /*
/// let performance = gradual_perf.process_next_n_objects(state.clone(), 10).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(state.clone(), 10);
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
/// # /*
/// let performance = gradual_perf.process_next_object(state.clone()).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_object(state.clone());
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
/// state.n_misses = ...
/// let final_performance = gradual_perf.process_next_n_objects(state.clone(), usize::MAX).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(state.clone(), usize::MAX);
///
/// // Once the final performance was calculated,
/// // attempting to process further objects will return `None`.
/// assert!(gradual_perf.process_next_object(state).is_none());
/// ```
#[derive(Clone, Debug)]
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
    pub fn process_next_object(
        &mut self,
        state: TaikoScoreState,
    ) -> Option<TaikoPerformanceAttributes> {
        self.process_next_n_objects(state, 1)
    }

    /// Same as [`process_next_object`](`TaikoGradualPerformanceAttributes::process_next_object`)
    /// but instead of processing only one object it process `n` many.
    ///
    /// If `n` is 0 it will be considered as 1.
    /// If there are still objects to be processed but `n` is larger than the amount
    /// of remaining objects, `n` will be considered as the amount of remaining objects.
    pub fn process_next_n_objects(
        &mut self,
        state: TaikoScoreState,
        n: usize,
    ) -> Option<TaikoPerformanceAttributes> {
        let sub = 2 * !self.difficulty.started as usize;
        let difficulty = self.difficulty.nth(n.saturating_sub(sub))?;
        let passed_objects = difficulty.max_combo;

        let performance = self
            .performance
            .clone()
            .attributes(difficulty)
            .state(state)
            .passed_objects(passed_objects)
            .calculate();

        Some(performance)
    }
}
