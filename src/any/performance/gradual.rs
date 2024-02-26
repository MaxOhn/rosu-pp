use std::borrow::Cow;

use rosu_map::section::general::GameMode;

use crate::{
    any::{PerformanceAttributes, ScoreState},
    catch::{CatchBeatmap, CatchGradualPerformance},
    mania::{ManiaBeatmap, ManiaGradualPerformance},
    osu::{OsuBeatmap, OsuGradualPerformance},
    taiko::{TaikoBeatmap, TaikoGradualPerformance},
    Beatmap, Converted, ModeDifficulty,
};

/// Gradually calculate the performance attributes on maps of any mode.
///
/// After each hit object you can call [`next`] and it will return the
/// resulting current [`PerformanceAttributes`]. To process multiple objects at
/// the once, use [`nth`] instead.
///
/// Both methods require a [`ScoreState`] that contains the current hitresults
/// as well as the maximum combo so far. Since the map could have any mode, all
/// fields of `ScoreState` could be of use and should be updated properly.
///
/// Alternatively, you can match on the map's mode yourself and use the gradual
/// performance attribute struct for the corresponding mode, i.e.
/// [`OsuGradualPerformance`], [`TaikoGradualPerformance`],
/// [`CatchGradualPerformance`], or [`ManiaGradualPerformance`].
///
/// If you only want to calculate difficulty attributes use [`GradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, GradualPerformance, ModeDifficulty, ScoreState};
///
/// let map = Beatmap::from_path("./resources/2785319.osu").unwrap();
/// let difficulty = ModeDifficulty::new().mods(64); // DT
/// let mut gradual_perf = GradualPerformance::new(&difficulty, &map);
/// let mut state = ScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     let performance = gradual_perf.next(state.clone()).unwrap();
///     println!("PP: {}", performance.pp());
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.n_misses += 1;
///
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
///
/// // The `nth` method takes a zero-based value.
/// let performance = gradual_perf.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
///
/// let performance = gradual_perf.next(state.clone()).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// ...
/// # */
/// let final_performance = gradual_perf.last(state.clone()).unwrap();
/// println!("PP: {}", performance.pp());
///
/// // Once the final performance has been calculated, attempting to process
/// // further objects will return `None`.
/// assert!(gradual_perf.next(state).is_none());
/// ```
///
/// [`next`]: GradualPerformance::next
/// [`nth`]: GradualPerformance::nth
/// [`GradualDifficulty`]: crate::GradualDifficulty
pub enum GradualPerformance {
    Osu(OsuGradualPerformance),
    Taiko(TaikoGradualPerformance),
    Catch(CatchGradualPerformance),
    Mania(ManiaGradualPerformance),
}

macro_rules! from_converted {
    ( $fn:ident, $mode:ident, $converted:ident, $gradual:ident ) => {
        #[doc = concat!("Create a [`GradualPerformance`] for a [`", stringify!($converted), "`]")]
        pub fn $fn(difficulty: &ModeDifficulty, converted: &$converted<'_>) -> Self {
            Self::$mode($gradual::new(difficulty, converted))
        }
    };
}

impl GradualPerformance {
    /// Create a [`GradualPerformance`] for a map of any mode.
    pub fn new(difficulty: &ModeDifficulty, map: &Beatmap) -> Self {
        let map = Cow::Borrowed(map);

        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualPerformance::new(
                difficulty,
                &Converted::new(map, false),
            )),
            GameMode::Taiko => Self::Taiko(TaikoGradualPerformance::new(
                difficulty,
                &Converted::new(map, false),
            )),
            GameMode::Catch => Self::Catch(CatchGradualPerformance::new(
                difficulty,
                &Converted::new(map, false),
            )),
            GameMode::Mania => Self::Mania(ManiaGradualPerformance::new(
                difficulty,
                &Converted::new(map, false),
            )),
        }
    }

    from_converted!(from_osu_map, Osu, OsuBeatmap, OsuGradualPerformance);
    from_converted!(from_taiko_map, Taiko, TaikoBeatmap, TaikoGradualPerformance);
    from_converted!(from_catch_map, Catch, CatchBeatmap, CatchGradualPerformance);
    from_converted!(from_mania_map, Mania, ManiaBeatmap, ManiaGradualPerformance);

    /// Process the next hit object and calculate the performance attributes
    /// for the resulting score state.
    pub fn next(&mut self, state: ScoreState) -> Option<PerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance
    /// attributes.
    pub fn last(&mut self, state: ScoreState) -> Option<PerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up to the next `n`th hitobject and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object,
    /// `n=1` will process 2, and so on.
    pub fn nth(&mut self, state: ScoreState, n: usize) -> Option<PerformanceAttributes> {
        match self {
            GradualPerformance::Osu(gradual) => {
                gradual.nth(state.into(), n).map(PerformanceAttributes::Osu)
            }
            GradualPerformance::Taiko(gradual) => gradual
                .nth(state.into(), n)
                .map(PerformanceAttributes::Taiko),
            GradualPerformance::Catch(gradual) => gradual
                .nth(state.into(), n)
                .map(PerformanceAttributes::Catch),
            GradualPerformance::Mania(gradual) => gradual
                .nth(state.into(), n)
                .map(PerformanceAttributes::Mania),
        }
    }
}
