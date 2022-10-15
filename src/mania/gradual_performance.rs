use crate::{Beatmap, ManiaPP};

use super::{ManiaGradualDifficultyAttributes, ManiaPerformanceAttributes};

/// TODO: docs
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
    /// Return the total amount of hits by adding everything up.
    pub fn total_hits(&self) -> usize {
        self.n320 + self.n300 + self.n200 + self.n100 + self.n50 + self.n_misses
    }
}

impl ManiaScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
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
/// use rosu_pp::{Beatmap, mania::ManiaGradualPerformanceAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = ManiaGradualPerformanceAttributes::new(&map, mods);
/// let mut score = 0;
///
/// // The first 10 objects each increase the score by 123.
/// for _ in 0..10 {
///     score += 123;
///
///     # /*
///     let performance = gradual_perf.process_next_object(score).unwrap();
///     println!("PP: {}", performance.pp);
///     # */
///     # let _ = gradual_perf.process_next_object(score);
/// }
///
/// // Then comes a miss so no additional score is added.
/// # /*
/// let performance = gradual_perf.process_next_object(score).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_object(score);
///
/// // The next 10 objects give a total of 987 score and will be processed in one go.
/// score += 987;
/// # /*
/// let performance = gradual_perf.process_next_n_objects(score, 10).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(score, 10);
///
/// // Skip to the end
/// # /*
/// score = ...
/// let final_performance = gradual_perf.process_next_n_objects(score, usize::MAX).unwrap();
/// println!("PP: {}", performance.pp);
/// # */
/// # let _ = gradual_perf.process_next_n_objects(score, usize::MAX);
///
/// // Once the final performance was calculated,
/// // attempting to process further objects will return `None`.
/// assert!(gradual_perf.process_next_object(score).is_none());
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
        let n = n.min(self.difficulty.len()).saturating_sub(1);
        let difficulty = self.difficulty.nth(n)?;

        self.performance.n320 = Some(state.n320);
        self.performance.n300 = Some(state.n300);
        self.performance.n200 = Some(state.n200);
        self.performance.n100 = Some(state.n100);
        self.performance.n50 = Some(state.n50);
        self.performance.n_misses = Some(state.n_misses);

        let performance = self
            .performance
            .clone()
            .attributes(difficulty)
            .passed_objects(self.difficulty.idx)
            .calculate();

        Some(performance)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn correct_empty() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;

        let mut gradual = ManiaGradualPerformanceAttributes::new(&map, mods);

        let state = ManiaScoreState {
            n320: 0,
            n300: 0,
            n200: 0,
            n100: 0,
            n50: 0,
            n_misses: 0,
        };

        assert!(gradual
            .process_next_n_objects(state.clone(), usize::MAX)
            .is_some());
        assert!(gradual.process_next_object(state).is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn next_and_next_n() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;

        let mut state = ManiaScoreState {
            n320: 0,
            n300: 0,
            n200: 0,
            n100: 0,
            n50: 0,
            n_misses: 0,
        };

        let mut gradual1 = ManiaGradualPerformanceAttributes::new(&map, mods);
        let mut gradual2 = ManiaGradualPerformanceAttributes::new(&map, mods);

        for _ in 0..20 {
            let _ = gradual1.process_next_object(state.clone());
            let _ = gradual2.process_next_object(state.clone());
            state.n320 += 1;
        }

        let n = 80;

        for _ in 1..n {
            let _ = gradual1.process_next_object(state.clone());
            state.n320 += 1;
        }

        let next = gradual1.process_next_object(state.clone());
        let next_n = gradual2.process_next_n_objects(state, n);

        assert_eq!(next_n, next);
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn gradual_end_eq_regular() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;
        let regular = ManiaPP::new(&map).mods(mods).calculate();

        let mut gradual = ManiaGradualPerformanceAttributes::new(&map, mods);

        let state = ManiaScoreState {
            n320: 3238,
            n300: 0,
            n200: 0,
            n100: 0,
            n50: 0,
            n_misses: 0,
        };

        let gradual_end = gradual.process_next_n_objects(state, usize::MAX).unwrap();

        assert_eq!(regular, gradual_end);
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn gradual_eq_regular_passed() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;
        let n = 100;

        let state = ManiaScoreState {
            n320: 100,
            n300: 0,
            n200: 0,
            n100: 0,
            n50: 0,
            n_misses: 0,
        };

        let regular = ManiaPP::new(&map)
            .mods(mods)
            .passed_objects(n)
            .state(state.clone())
            .calculate();

        let mut gradual = ManiaGradualPerformanceAttributes::new(&map, mods);
        let gradual = gradual.process_next_n_objects(state, n).unwrap();

        assert_eq!(regular, gradual);
    }
}
