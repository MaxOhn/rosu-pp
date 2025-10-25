use crate::{Beatmap, Difficulty, mania::ManiaGradualDifficulty, model::mode::ConvertError};

use super::{ManiaPerformanceAttributes, ManiaScoreState};

/// Gradually calculate the performance attributes of an osu!mania map.
///
/// After each hit object you can call [`next`] and it will return the
/// resulting current [`ManiaPerformanceAttributes`]. To process multiple
/// objects at once, use [`nth`] instead.
///
/// Both methods require a play's current score so far. Be sure the given score
/// is adjusted with respect to mods.
///
/// If you only want to calculate difficulty attributes use
/// [`ManiaGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::mania::{Mania, ManiaGradualPerformance, ManiaScoreState};
///
/// let map = Beatmap::from_path("./resources/1638954.osu").unwrap();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut gradual = ManiaGradualPerformance::new(difficulty, &map).unwrap();
/// let mut state = ManiaScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 320s
/// for _ in 0..10 {
///     state.n320 += 1;
///
///     let attrs = gradual.next(state.clone()).unwrap();
///     println!("PP: {}", attrs.pp);
/// }
///
/// // Then comes a miss.
/// state.misses += 1;
/// let attrs = gradual.next(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // The next 10 objects will be a mixture of 320s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n320 += 3;
/// state.n100 += 7;
/// // The `nth` method takes a zero-based value.
/// let attrs = gradual.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
/// state.misses = ...
/// # */
/// let attrs = gradual.last(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // Once the final performance was calculated,
/// // attempting to process further objects will return `None`.
/// assert!(gradual.next(state).is_none());
/// ```
///
/// [`next`]: ManiaGradualPerformance::next
/// [`nth`]: ManiaGradualPerformance::nth
pub struct ManiaGradualPerformance {
    difficulty: ManiaGradualDifficulty,
}

impl ManiaGradualPerformance {
    /// Create a new gradual performance calculator for osu!mania maps.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Result<Self, ConvertError> {
        let difficulty = ManiaGradualDifficulty::new(difficulty, map)?;

        Ok(Self { difficulty })
    }

    /// Process the next hit object and calculate the performance attributes
    /// for the resulting score.
    pub fn next(&mut self, state: ManiaScoreState) -> Option<ManiaPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance
    /// attributes.
    pub fn last(&mut self, state: ManiaScoreState) -> Option<ManiaPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object,
    /// `n=1` will process 2, and so on.
    #[allow(clippy::missing_panics_doc)]
    pub fn nth(&mut self, state: ManiaScoreState, n: usize) -> Option<ManiaPerformanceAttributes> {
        let performance = self
            .difficulty
            .nth(n)?
            .performance()
            .state(state)
            .difficulty(self.difficulty.difficulty.clone())
            .passed_objects(self.difficulty.idx as u32)
            .calculate()
            .expect("no conversion required");

        Some(performance)
    }

    /// Returns the amount of remaining objects.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.difficulty.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Beatmap, mania::ManiaPerformance};

    use super::*;

    #[test]
    fn next_and_nth() {
        let map = Beatmap::from_path("./resources/1638954.osu").unwrap();

        let difficulty = Difficulty::new().mods(88); // HDHRDT

        let mut gradual = ManiaGradualPerformance::new(difficulty.clone(), &map).unwrap();
        let mut gradual_2nd = ManiaGradualPerformance::new(difficulty.clone(), &map).unwrap();
        let mut gradual_3rd = ManiaGradualPerformance::new(difficulty.clone(), &map).unwrap();

        let mut state = ManiaScoreState::default();

        let hit_objects_len = map.hit_objects.len();

        for i in 1.. {
            state.misses += 1;

            // Hold notes award two hitresults in lazer
            if let Some(h) = map.hit_objects.get(i - 1) {
                if !h.is_circle() {
                    state.n320 += 1;
                }
            }

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

            let mut regular_calc = ManiaPerformance::new(&map)
                .difficulty(difficulty.clone())
                .passed_objects(i as u32)
                .state(state.clone());

            let regular_state = regular_calc.generate_state().unwrap();
            assert_eq!(state, regular_state);

            let expected = regular_calc.calculate().unwrap();

            assert_eq!(next_gradual, expected);
        }
    }
}
