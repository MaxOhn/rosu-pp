use crate::{
    catch::{CatchGradualDifficultyAttributes, CatchGradualPerformanceAttributes, CatchScoreState},
    mania::{ManiaGradualDifficultyAttributes, ManiaGradualPerformanceAttributes, ManiaScoreState},
    osu::{OsuGradualDifficultyAttributes, OsuGradualPerformanceAttributes, OsuScoreState},
    taiko::{TaikoGradualDifficultyAttributes, TaikoGradualPerformanceAttributes, TaikoScoreState},
    Beatmap, DifficultyAttributes, GameMode, PerformanceAttributes,
};

/// Gradually calculate the difficulty attributes on maps of any mode.
///
/// Note that this struct implements [`Iterator`](std::iter::Iterator).
/// On every call of [`Iterator::next`](std::iter::Iterator::next), the map's next hit object will
/// be processed and the [`DifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`GradualPerformanceAttributes`](crate::GradualPerformanceAttributes) instead.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{Beatmap, GradualDifficultyAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = GradualDifficultyAttributes::new(&map, mods);
///
/// let attrs1 = iter.next(); // the difficulty of the map after the first hit object
/// let attrs2 = iter.next(); //                           after the second hit object
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum GradualDifficultyAttributes<'map> {
    /// Gradual osu!standard difficulty attributes.
    Osu(OsuGradualDifficultyAttributes),
    /// Gradual osu!taiko difficulty attributes.
    Taiko(TaikoGradualDifficultyAttributes),
    /// Gradual osu!catch difficulty attributes.
    Catch(CatchGradualDifficultyAttributes<'map>),
    /// Gradual osu!mania difficulty attributes.
    Mania(ManiaGradualDifficultyAttributes<'map>),
}

impl<'map> GradualDifficultyAttributes<'map> {
    /// Create a new gradual difficulty calculator for maps of any mode.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualDifficultyAttributes::new(map, mods)),
            GameMode::Taiko => Self::Taiko(TaikoGradualDifficultyAttributes::new(map, mods)),
            GameMode::Catch => Self::Catch(CatchGradualDifficultyAttributes::new(map, mods)),
            GameMode::Mania => Self::Mania(ManiaGradualDifficultyAttributes::new(map, mods)),
        }
    }
}

impl Iterator for GradualDifficultyAttributes<'_> {
    type Item = DifficultyAttributes;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GradualDifficultyAttributes::Osu(o) => o.next().map(DifficultyAttributes::Osu),
            GradualDifficultyAttributes::Taiko(t) => t.next().map(DifficultyAttributes::Taiko),
            GradualDifficultyAttributes::Catch(f) => f.next().map(DifficultyAttributes::Catch),
            GradualDifficultyAttributes::Mania(m) => m.next().map(DifficultyAttributes::Mania),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GradualDifficultyAttributes::Osu(o) => o.size_hint(),
            GradualDifficultyAttributes::Taiko(t) => t.size_hint(),
            GradualDifficultyAttributes::Catch(f) => f.size_hint(),
            GradualDifficultyAttributes::Mania(m) => m.size_hint(),
        }
    }
}

/// Aggregation for a score's current state i.e. what is
/// the maximum combo so far, what are the current
/// hitresults and what is the current score.
///
/// This struct is used for [`GradualPerformanceAttributes`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    ///
    /// Note that for osu!catch only fruits and droplets are considered for combo.
    ///
    /// Irrelevant for osu!mania.
    pub max_combo: usize,
    /// Amount of current gekis (n320 for osu!mania).
    pub n_geki: usize,
    /// Amount of current katus (tiny droplet misses for osu!catch / n200 for osu!mania).
    pub n_katu: usize,
    /// Amount of current 300s (fruits for osu!catch).
    pub n300: usize,
    /// Amount of current 100s (droplets for osu!catch).
    pub n100: usize,
    /// Amount of current 50s (tiny droplets for osu!catch).
    pub n50: usize,
    /// Amount of current misses (fruits + droplets for osu!catch).
    pub n_misses: usize,
}

impl ScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<ScoreState> for OsuScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            n_misses: state.n_misses,
        }
    }
}

impl From<ScoreState> for TaikoScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            n_misses: state.n_misses,
        }
    }
}

impl From<ScoreState> for CatchScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n_fruits: state.n300,
            n_droplets: state.n100,
            n_tiny_droplets: state.n50,
            n_tiny_droplet_misses: state.n_katu,
            n_misses: state.n_misses,
        }
    }
}

impl From<ScoreState> for ManiaScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            n320: state.n_geki,
            n300: state.n300,
            n200: state.n_katu,
            n100: state.n100,
            n50: state.n50,
            n_misses: state.n_misses,
        }
    }
}

/// Gradually calculate the performance attributes on maps of any mode.
///
/// After each hit object you can call
/// [`process_next_object`](`GradualPerformanceAttributes::process_next_object`)
/// and it will return the resulting current [`PerformanceAttributes`].
/// To process multiple objects at once, use
/// [`process_next_n_objects`](`GradualPerformanceAttributes::process_next_n_objects`) instead.
///
/// Both methods require a [`ScoreState`] that contains the current hitresults
/// as well as the maximum combo so far or just the current score for osu!mania.
/// Since the map could have any mode, all fields of `ScoreState` could be of use
/// and should be updated properly.
///
/// Alternatively, you can match on the map's mode yourself and use the gradual
/// performance attribute struct for the corresponding mode, i.e.
/// [`OsuGradualPerformanceAttributes`],
/// [`TaikoGradualPerformanceAttributes`],
/// [`CatchGradualPerformanceAttributes`], or
/// [`ManiaGradualPerformanceAttributes`].
///
/// If you only want to calculate difficulty attributes use
/// [`GradualDifficultyAttributes`](crate::GradualDifficultyAttributes) instead.
///
/// # Example
///
/// ```no_run
/// use rosu_pp::{Beatmap, GradualPerformanceAttributes, ScoreState};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut gradual_perf = GradualPerformanceAttributes::new(&map, mods);
/// let mut state = ScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s and increase the score by 123 each.
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
/// // Don't forget state.n_katu
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
/// ...
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
#[allow(clippy::large_enum_variant)]
pub enum GradualPerformanceAttributes<'map> {
    /// Gradual osu!standard performance attributes.
    Osu(OsuGradualPerformanceAttributes<'map>),
    /// Gradual osu!taiko performance attributes.
    Taiko(TaikoGradualPerformanceAttributes<'map>),
    /// Gradual osu!catch performance attributes.
    Catch(CatchGradualPerformanceAttributes<'map>),
    /// Gradual osu!mania performance attributes.
    Mania(ManiaGradualPerformanceAttributes<'map>),
}

impl<'map> GradualPerformanceAttributes<'map> {
    /// Create a new gradual performance calculator for maps of any mode.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            GameMode::Osu => Self::Osu(OsuGradualPerformanceAttributes::new(map, mods)),
            GameMode::Taiko => Self::Taiko(TaikoGradualPerformanceAttributes::new(map, mods)),
            GameMode::Catch => Self::Catch(CatchGradualPerformanceAttributes::new(map, mods)),
            GameMode::Mania => Self::Mania(ManiaGradualPerformanceAttributes::new(map, mods)),
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score.
    pub fn process_next_object(&mut self, state: ScoreState) -> Option<PerformanceAttributes> {
        self.process_next_n_objects(state, 1)
    }

    /// Same as [`process_next_object`](`GradualPerformanceAttributes::process_next_object`)
    /// but instead of processing only one object it process `n` many.
    ///
    /// If `n` is 0 it will be considered as 1.
    /// If there are still objects to be processed but `n` is larger than the amount
    /// of remaining objects, `n` will be considered as the amount of remaining objects.
    pub fn process_next_n_objects(
        &mut self,
        state: ScoreState,
        n: usize,
    ) -> Option<PerformanceAttributes> {
        match self {
            GradualPerformanceAttributes::Osu(o) => o
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Osu),
            GradualPerformanceAttributes::Taiko(t) => t
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Taiko),
            GradualPerformanceAttributes::Catch(f) => f
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Catch),
            GradualPerformanceAttributes::Mania(m) => m
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Mania),
        }
    }
}
