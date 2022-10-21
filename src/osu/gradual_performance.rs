use crate::{Beatmap, OsuPP};

use super::{OsuGradualDifficultyAttributes, OsuPerformanceAttributes};

/// Aggregation for a score's current state i.e. what was the
/// maximum combo so far and what are the current hitresults.
///
/// This struct is used for [`OsuGradualPerformanceAttributes`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OsuScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    pub max_combo: usize,
    /// Amount of current 300s.
    pub n300: usize,
    /// Amount of current 100s.
    pub n100: usize,
    /// Amount of current 50s.
    pub n50: usize,
    /// Amount of current misses.
    pub n_misses: usize,
}

impl OsuScoreState {
    /// Create a new empty score state.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    #[inline]
    pub fn total_hits(&self) -> usize {
        self.n300 + self.n100 + self.n50 + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    #[inline]
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 6 * self.n300 + 2 * self.n100 + self.n50;
        let denominator = 6 * total_hits;

        numerator as f64 / denominator as f64
    }
}

/// Gradually calculate the performance attributes of an osu!standard map.
///
/// After each hit object you can call
/// [`process_next_object`](`OsuGradualPerformanceAttributes::process_next_object`)
/// and it will return the resulting current [`OsuPerformanceAttributes`].
/// To process multiple objects at once, use
/// [`process_next_n_objects`](`OsuGradualPerformanceAttributes::process_next_n_objects`) instead.
///
/// Both methods require an [`OsuScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`OsuGradualDifficultyAttributes`](crate::osu::OsuGradualDifficultyAttributes) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, osu::{OsuGradualPerformanceAttributes, OsuScoreState}};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = OsuGradualPerformanceAttributes::new(&map, mods);
/// let mut state = OsuScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s and there are no sliders for additional combo
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
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
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
/// state.n50 = ...
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
#[derive(Debug)]
pub struct OsuGradualPerformanceAttributes<'map> {
    difficulty: OsuGradualDifficultyAttributes,
    performance: OsuPP<'map>,
}

impl<'map> OsuGradualPerformanceAttributes<'map> {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = OsuGradualDifficultyAttributes::new(map, mods);
        let performance = OsuPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score state.
    pub fn process_next_object(
        &mut self,
        state: OsuScoreState,
    ) -> Option<OsuPerformanceAttributes> {
        self.process_next_n_objects(state, 1)
    }

    /// Same as [`process_next_object`](`OsuGradualPerformanceAttributes::process_next_object`)
    /// but instead of processing only one object it process `n` many.
    ///
    /// If `n` is 0 it will be considered as 1.
    /// If there are still objects to be processed but `n` is larger than the amount
    /// of remaining objects, `n` will be considered as the amount of remaining objects.
    pub fn process_next_n_objects(
        &mut self,
        state: OsuScoreState,
        n: usize,
    ) -> Option<OsuPerformanceAttributes> {
        let sub = (self.difficulty.idx == 0) as usize;
        let difficulty = self.difficulty.nth(n.saturating_sub(sub))?;

        let performance = self
            .performance
            .clone()
            .attributes(difficulty)
            .state(state)
            .passed_objects(self.difficulty.idx + 1)
            .calculate();

        Some(performance)
    }
}
