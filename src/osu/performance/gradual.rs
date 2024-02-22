use crate::{
    osu::{OsuBeatmap, OsuGradualDifficulty},
    ModeDifficulty,
};

use super::{OsuPerformanceAttributes, OsuScoreState};

/// Gradually calculate the performance attributes of an osu!standard map.
///
/// After each hit object you can call [`next`]
/// and it will return the resulting current [`OsuPerformanceAttributes`].
/// To process multiple objects at once, use [`nth`] instead.
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
/// use rosu_pp::{Beatmap, ModeDifficulty};
/// use rosu_pp::osu::{Osu, OsuGradualPerformance, OsuScoreState};
///
/// let converted = Beatmap::from_path("./resources/2785319.osu")
///     .unwrap()
///     .unchecked_into_converted::<Osu>();
///
/// let difficulty = ModeDifficulty::new().mods(64); // DT
/// let mut gradual_perf = OsuGradualPerformance::new(&difficulty, converted);
/// let mut state = OsuScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hits are 300s and there are no sliders for additional combo
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     let performance = gradual_perf.next(state.clone()).unwrap();
///     println!("PP: {}", performance.pp);
/// }
///
/// // Then comes a miss. Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.n_misses += 1;
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp);
///
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
/// // The `nth` method takes a zero-based value.
/// let performance = gradual_perf.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", performance.pp);
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp);
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
/// state.n50 = ...
/// state.n_misses = ...
/// # */
/// let final_performance = gradual_perf.nth(state.clone(), usize::MAX).unwrap();
/// println!("PP: {}", performance.pp);
///
/// // Once the final performance has been calculated, attempting to process
/// // further objects will return `None`.
/// assert!(gradual_perf.next(state).is_none());
/// ```
///
/// [`next`]: OsuGradualPerformance::next
/// [`nth`]: OsuGradualPerformance::nth
#[derive(Debug)]
pub struct OsuGradualPerformance {
    difficulty: OsuGradualDifficulty,
}

impl OsuGradualPerformance {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(difficulty: &ModeDifficulty, converted: &OsuBeatmap<'_>) -> Self {
        let difficulty = OsuGradualDifficulty::new(difficulty, converted);

        Self { difficulty }
    }

    /// Process the next hit object and calculate the performance attributes
    /// for the resulting score state.
    pub fn next(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance
    /// attributes.
    pub fn last(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up to the next `n`th hitobject and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object,
    /// `n=1` will process 2, and so on.
    pub fn nth(&mut self, state: OsuScoreState, n: usize) -> Option<OsuPerformanceAttributes> {
        let performance = self
            .difficulty
            .nth(n)?
            .performance()
            .state(state)
            .mods(self.difficulty.mods)
            .clock_rate(self.difficulty.clock_rate)
            .passed_objects(self.difficulty.idx as u32)
            .calculate();

        Some(performance)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        osu::{Osu, OsuPerformance},
        Beatmap,
    };

    use super::*;

    #[test]
    fn next_and_nth() {
        let converted = Beatmap::from_path("./resources/2785319.osu")
            .unwrap()
            .unchecked_into_converted::<Osu>();

        let mods = 88; // HDHRDT
        let difficulty = ModeDifficulty::new().mods(88);

        let mut gradual = OsuGradualPerformance::new(&difficulty, &converted);
        let mut gradual_2nd = OsuGradualPerformance::new(&difficulty, &converted);
        let mut gradual_3rd = OsuGradualPerformance::new(&difficulty, &converted);

        let mut state = OsuScoreState::default();

        let hit_objects_len = converted.map.hit_objects.len();

        for i in 1.. {
            state.n_misses += 1;

            let Some(next_gradual) = gradual.next(state.clone()) else {
                assert_eq!(i, hit_objects_len + 1);
                assert!(gradual_2nd.last(state.clone()).is_some() || hit_objects_len % 2 == 0);
                assert!(gradual_3rd.last(state.clone()).is_some() || hit_objects_len % 3 == 0);
                break;
            };

            if i % 2 == 0 {
                let next_gradual_2nd = gradual_2nd.nth(state.clone(), 1).unwrap();
                assert_eq!(next_gradual, next_gradual_2nd);
            }

            if i % 3 == 0 {
                let next_gradual_3rd = gradual_3rd.nth(state.clone(), 2).unwrap();
                assert_eq!(next_gradual, next_gradual_3rd);
            }

            let mut regular_calc = OsuPerformance::new(converted.as_owned())
                .mods(mods)
                .passed_objects(i as u32)
                .state(state.clone());

            let regular_state = regular_calc.generate_state();
            assert_eq!(state, regular_state);

            let expected = regular_calc.calculate();

            assert_eq!(next_gradual, expected);
        }
    }
}
