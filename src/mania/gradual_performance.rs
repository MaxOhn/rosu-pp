use crate::{Beatmap, ManiaPP};

use super::{ManiaGradualDifficultyAttributes, ManiaPerformanceAttributes};

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
    pub fn process_next_object(&mut self, score: u32) -> Option<ManiaPerformanceAttributes> {
        self.process_next_n_objects(score, 1)
    }

    /// Same as [`process_next_object`](`ManiaGradualPerformanceAttributes::process_next_object`)
    /// but instead of processing only one object it process `n` many.
    ///
    /// If `n` is 0 it will be considered as 1.
    /// If there are still objects to be processed but `n` is larger than the amount
    /// of remaining objects, `n` will be considered as the amount of remaining objects.
    pub fn process_next_n_objects(
        &mut self,
        score: u32,
        n: usize,
    ) -> Option<ManiaPerformanceAttributes> {
        let n = n.min(self.difficulty.len()).saturating_sub(1);
        let difficulty = self.difficulty.nth(n)?;

        let _ = self.performance.score.insert(score as f64);

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
        let score = 0;

        assert!(gradual.process_next_n_objects(score, usize::MAX).is_some());
        assert!(gradual.process_next_object(score).is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn next_and_next_n() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;
        let score = 0;

        let mut gradual1 = ManiaGradualPerformanceAttributes::new(&map, mods);
        let mut gradual2 = ManiaGradualPerformanceAttributes::new(&map, mods);

        for _ in 0..20 {
            let _ = gradual1.process_next_object(score);
            let _ = gradual2.process_next_object(score);
        }

        let n = 80;

        for _ in 1..n {
            let _ = gradual1.process_next_object(score);
        }

        let score = 100_000;
        let next = gradual1.process_next_object(score);
        let next_n = gradual2.process_next_n_objects(score, n);

        assert_eq!(next_n, next);
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn gradual_end_eq_regular() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;
        let regular = ManiaPP::new(&map).mods(mods).calculate();

        let mut gradual = ManiaGradualPerformanceAttributes::new(&map, mods);

        let score = 1_000_000;
        let gradual_end = gradual.process_next_n_objects(score, usize::MAX).unwrap();

        assert_eq!(regular, gradual_end);
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn gradual_eq_regular_passed() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;
        let n = 100;
        let score = 100_000;

        let regular = ManiaPP::new(&map)
            .mods(mods)
            .passed_objects(n)
            .score(score)
            .calculate();

        let mut gradual = ManiaGradualPerformanceAttributes::new(&map, mods);
        let gradual = gradual.process_next_n_objects(score, n).unwrap();

        assert_eq!(regular, gradual);
    }
}
