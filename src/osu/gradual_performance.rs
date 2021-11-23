use crate::{Beatmap, OsuPP};

use super::{OsuGradualDifficultyAttributes, OsuPerformanceAttributes};

// TODO: Benchmark if Copy is faster than Clone
/// Aggregation for a score's current state i.e. what was the
/// maximum combo so far and what are the current hitresults.
#[derive(Copy, Clone, Debug, Default)]
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
    pub misses: usize,
}

impl OsuScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
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
///     let performance = gradual_perf.process_next_object(state).unwrap();
///     println!("PP: {}", performance.pp);
///     # */
///     # let _ = gradual_perf.process_next_object(state);
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.misses += 1;
/// # /*
/// let performance = gradual_perf.process_next_object(state).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_object(state);
///
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
/// # /*
/// let performance = gradual_perf.process_next_n_objects(state, 10).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(state, 10);
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
/// # /*
/// let performance = gradual_perf.process_next_object(state).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_object(state);
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
/// state.n50 = ...
/// state.misses = ...
/// let final_performance = gradual_perf.process_next_n_objects(state, usize::MAX).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(state, usize::MAX);
///
/// // Once the final performance was calculated,
/// // attempting to process further objects will return `None`.
/// assert!(gradual_perf.process_next_object(state).is_none());
/// ```
#[derive(Clone, Debug)]
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
        let n = n.min(self.difficulty.len()).saturating_sub(1);
        let difficulty = self.difficulty.nth(n)?;

        let _ = self.performance.n300.insert(state.n300);
        let _ = self.performance.n100.insert(state.n100);
        let _ = self.performance.n50.insert(state.n50);
        self.performance.n_misses = state.misses;

        let performance = self
            .performance
            .clone()
            .attributes(difficulty)
            .combo(state.max_combo)
            .passed_objects(self.difficulty.idx)
            .calculate();

        Some(performance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn correct_empty() {
        let map = Beatmap::from_path("./maps/2785319.osu").expect("failed to parse map");
        let mods = 64;

        let mut gradual = OsuGradualPerformanceAttributes::new(&map, mods);
        let state = OsuScoreState::default();

        assert!(gradual.process_next_n_objects(state, usize::MAX).is_some());
        assert!(gradual.process_next_object(state).is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn next_and_next_n() {
        let map = Beatmap::from_path("./maps/2785319.osu").expect("failed to parse map");
        let mods = 64;
        let state = OsuScoreState::default();

        let mut gradual1 = OsuGradualPerformanceAttributes::new(&map, mods);
        let mut gradual2 = OsuGradualPerformanceAttributes::new(&map, mods);

        for _ in 0..20 {
            let _ = gradual1.process_next_object(state);
            let _ = gradual2.process_next_object(state);
        }

        let n = 80;

        for _ in 1..n {
            let _ = gradual1.process_next_object(state);
        }

        let state = OsuScoreState {
            max_combo: 110,
            n300: 90,
            n100: 8,
            n50: 2,
            misses: 2,
        };

        let next = gradual1.process_next_object(state);
        let next_n = gradual2.process_next_n_objects(state, n);

        assert_eq!(next_n, next);
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn gradual_end_eq_regular() {
        let map = Beatmap::from_path("./maps/2785319.osu").expect("failed to parse map");
        let mods = 64;
        let regular = OsuPP::new(&map).mods(mods).calculate();

        let mut gradual = OsuGradualPerformanceAttributes::new(&map, mods);

        let state = OsuScoreState {
            max_combo: 909,
            n300: 601,
            n100: 0,
            n50: 0,
            misses: 0,
        };

        let gradual_end = gradual.process_next_n_objects(state, usize::MAX).unwrap();

        assert_eq!(regular, gradual_end);
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn gradual_eq_regular_passed() {
        let map = Beatmap::from_path("./maps/2785319.osu").expect("failed to parse map");
        let mods = 64;
        let n = 100;
        let regular = OsuPP::new(&map).mods(mods).passed_objects(n).calculate();

        let mut gradual = OsuGradualPerformanceAttributes::new(&map, mods);

        let state = OsuScoreState {
            max_combo: 122,
            n300: 100,
            n100: 0,
            n50: 0,
            misses: 0,
        };

        let gradual = gradual.process_next_n_objects(state, n).unwrap();

        assert_eq!(regular, gradual);
    }
}
