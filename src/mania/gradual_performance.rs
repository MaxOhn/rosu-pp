use crate::{Beatmap, ManiaPP};

use super::{ManiaGradualDifficultyAttributes, ManiaPerformanceAttributes};

/// Aggregation for a score's current state
/// i.e. what are the current hitresults.
///
/// This struct is used for [`ManiaGradualPerformanceAttributes`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManiaScoreState {
    /// Amount of current 320s.
    pub n320: usize,
    /// Amount of current 300s.
    pub n300: usize,
    /// Amount of current 200s.
    pub n200: usize,
    /// Amount of current 100s.
    pub n100: usize,
    /// Amount of current 50s.
    pub n50: usize,
    /// Amount of current misses.
    pub n_misses: usize,
}

impl ManiaScoreState {
    /// Create a new empty score state.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    #[inline]
    pub fn total_hits(&self) -> usize {
        self.n320 + self.n300 + self.n200 + self.n100 + self.n50 + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    #[inline]
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 6 * (self.n320 + self.n300) + 4 * self.n200 + 2 * self.n100 + self.n50;
        let denominator = 6 * total_hits;

        numerator as f64 / denominator as f64
    }
}

/// Gradually calculate the performance attributes of an osu!mania map.
///
/// After each hit object you can call
/// [`process_next_object`](`ManiaGradualPerformanceAttributes::process_next_object`)
/// and it will return the resulting current [`ManiaPerformanceAttributes`].
/// To process multiple objects at once, use
/// [`process_next_n_objects`](`ManiaGradualPerformanceAttributes::process_next_n_objects`) instead.
///
/// Both methods require a play's current score so far.
/// Be sure the given score is adjusted with respect to mods.
///
/// If you only want to calculate difficulty attributes use
/// [`ManiaGradualDifficultyAttributes`](crate::mania::ManiaGradualDifficultyAttributes) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, mania::{ManiaGradualPerformanceAttributes, ManiaScoreState}};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = ManiaGradualPerformanceAttributes::new(&map, mods);
/// let mut state = ManiaScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 320s
/// for _ in 0..10 {
///     state.n320 += 1;
///
///     # /*
///     let performance = gradual_perf.process_next_object(score).unwrap();
///     println!("PP: {}", performance.pp);
///     # */
///     # let _ = gradual_perf.process_next_object(state.clone());
/// }
///
/// // Then comes a miss.
/// state.n_misses += 1;
/// # /*
/// let performance = gradual_perf.process_next_object(score).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_object(state.clone());
///
/// // The next 10 objects will be a mixture of 320s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n320 += 3;
/// state.n100 += 7;
/// # /*
/// let performance = gradual_perf.process_next_n_objects(score, 10).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(state.clone(), 10);
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
pub struct ManiaGradualPerformanceAttributes<'map> {
    difficulty: ManiaGradualDifficultyAttributes<'map>,
    performance: ManiaPP<'map>,
}

impl<'map> ManiaGradualPerformanceAttributes<'map> {
    /// Create a new gradual performance calculator for osu!mania maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = ManiaGradualDifficultyAttributes::new(map, mods);
        let performance = ManiaPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score.
    pub fn process_next_object(
        &mut self,
        state: ManiaScoreState,
    ) -> Option<ManiaPerformanceAttributes> {
        self.process_next_n_objects(state, 1)
    }

    /// Same as [`process_next_object`](`ManiaGradualPerformanceAttributes::process_next_object`)
    /// but instead of processing only one object it process `n` many.
    ///
    /// If `n` is 0 it will be considered as 1.
    /// If there are still objects to be processed but `n` is larger than the amount
    /// of remaining objects, `n` will be considered as the amount of remaining objects.
    pub fn process_next_n_objects(
        &mut self,
        state: ManiaScoreState,
        n: usize,
    ) -> Option<ManiaPerformanceAttributes> {
        let sub = (self.difficulty.idx == 0) as usize;
        let difficulty = self.difficulty.nth(n.saturating_sub(sub))?;

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
