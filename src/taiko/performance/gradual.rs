use crate::{
    Beatmap, Difficulty,
    model::mode::ConvertError,
    taiko::{TaikoScoreState, difficulty::gradual::TaikoGradualDifficulty},
};

use super::TaikoPerformanceAttributes;

/// Gradually calculate the performance attributes of an osu!taiko map.
///
/// After each hit object you can call [`next`] and it will return the
/// resulting current [`TaikoPerformanceAttributes`]. To process multiple
/// objects at once, use [`nth`] instead.
///
/// Both methods require a [`TaikoScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`TaikoGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::taiko::{Taiko, TaikoGradualPerformance, TaikoScoreState};
///
/// let map = Beatmap::from_path("./resources/1028484.osu").unwrap();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut gradual = TaikoGradualPerformance::new(difficulty, &map).unwrap();
/// let mut state = TaikoScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     let attrs = gradual.next(state.clone()).unwrap();
///     println!("PP: {}", attrs.pp);
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.misses += 1;
/// let attrs = gradual.next(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // The next 10 objects will be a mixture of 300s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 3;
/// state.n100 += 7;
/// // The `nth` method takes a zero-based value.
/// let attrs = gradual.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
/// let attrs = gradual.next(state.clone()).unwrap();
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
/// // Once the final performance has been calculated, attempting to process
/// // further objects will return `None`.
/// assert!(gradual.next(state).is_none());
/// ```
///
/// [`next`]: TaikoGradualPerformance::next
/// [`nth`]: TaikoGradualPerformance::nth
pub struct TaikoGradualPerformance {
    difficulty: TaikoGradualDifficulty,
}

impl TaikoGradualPerformance {
    /// Create a new gradual performance calculator for osu!taiko maps.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Result<Self, ConvertError> {
        let difficulty = TaikoGradualDifficulty::new(difficulty, map)?;

        Ok(Self { difficulty })
    }

    /// Process the next hit object and calculate the performance attributes
    /// for the resulting score.
    pub fn next(&mut self, state: TaikoScoreState) -> Option<TaikoPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance
    /// attributes.
    pub fn last(&mut self, state: TaikoScoreState) -> Option<TaikoPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object,
    /// `n=1` will process 2, and so on.
    #[allow(clippy::missing_panics_doc, reason = "technically false positive")]
    pub fn nth(&mut self, state: TaikoScoreState, n: usize) -> Option<TaikoPerformanceAttributes> {
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
    use crate::{Beatmap, taiko::TaikoPerformance};

    use super::*;

    #[test]
    fn next_and_nth() {
        let map = Beatmap::from_path("./resources/1028484.osu").unwrap();

        let difficulty = Difficulty::new().mods(88); // HDHRDT

        let mut gradual = TaikoGradualPerformance::new(difficulty.clone(), &map).unwrap();
        let mut gradual_2nd = TaikoGradualPerformance::new(difficulty.clone(), &map).unwrap();
        let mut gradual_3rd = TaikoGradualPerformance::new(difficulty.clone(), &map).unwrap();

        let mut state = TaikoScoreState::default();

        let hit_objects_len = map.hit_objects.len();

        let n_hits = map.hit_objects.iter().filter(|h| h.is_circle()).count();

        for i in 1.. {
            state.misses += 1;

            let Some(next_gradual) = gradual.next(state) else {
                assert_eq!(i, n_hits + 1);
                assert!(gradual_2nd.last(state).is_some() || hit_objects_len % 2 == 0);
                assert!(gradual_3rd.last(state).is_some() || hit_objects_len % 3 == 0);
                break;
            };

            if i % 2 == 0 {
                let next_gradual_2nd = gradual_2nd.nth(state, 1).unwrap();
                assert_eq!(next_gradual, next_gradual_2nd);
            }

            if i % 3 == 0 {
                let next_gradual_3rd = gradual_3rd.nth(state, 2).unwrap();
                assert_eq!(next_gradual, next_gradual_3rd);
            }

            let mut regular_calc = TaikoPerformance::new(&map)
                .difficulty(difficulty.clone())
                .passed_objects(i as u32)
                .state(state);

            let regular_state = regular_calc.generate_state().unwrap();
            assert_eq!(state, regular_state);

            let expected = regular_calc.calculate().unwrap();

            assert_eq!(next_gradual, expected);
        }
    }
}
