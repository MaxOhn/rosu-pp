use crate::{Beatmap, DifficultyAttributes, GameMode, Mods, PerformanceAttributes};

#[cfg(feature = "fruits")]
use crate::fruits::{
    FruitsGradualDifficultyAttributes, FruitsGradualPerformanceAttributes, FruitsScoreState,
};

#[cfg(feature = "mania")]
use crate::mania::{ManiaGradualDifficultyAttributes, ManiaGradualPerformanceAttributes};

#[cfg(feature = "osu")]
use crate::osu::{OsuGradualDifficultyAttributes, OsuGradualPerformanceAttributes, OsuScoreState};

#[cfg(feature = "taiko")]
use crate::taiko::{
    TaikoGradualDifficultyAttributes, TaikoGradualPerformanceAttributes, TaikoScoreState,
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
#[derive(Clone, Debug)]
pub enum GradualDifficultyAttributes<'map> {
    #[cfg(feature = "fruits")]
    /// Gradual osu!fruits difficulty attributes.
    Fruits(FruitsGradualDifficultyAttributes<'map>),
    #[cfg(feature = "mania")]
    /// Gradual osu!mania difficulty attributes.
    Mania(ManiaGradualDifficultyAttributes<'map>),
    #[cfg(feature = "osu")]
    /// Gradual osu!standard difficulty attributes.
    Osu(OsuGradualDifficultyAttributes),
    #[cfg(feature = "taiko")]
    /// Gradual osu!taiko difficulty attributes.
    Taiko(TaikoGradualDifficultyAttributes<'map>),
}

impl<'map> GradualDifficultyAttributes<'map> {
    /// Create a new gradual difficulty calculator for maps of any mode.
    pub fn new(map: &'map Beatmap, mods: impl Mods) -> Self {
        match map.mode {
            #[cfg(feature = "osu")]
            GameMode::STD => Self::Osu(OsuGradualDifficultyAttributes::new(map, mods)),
            #[cfg(feature = "taiko")]
            GameMode::TKO => Self::Taiko(TaikoGradualDifficultyAttributes::new(map, mods)),
            #[cfg(feature = "fruits")]
            GameMode::CTB => Self::Fruits(FruitsGradualDifficultyAttributes::new(map, mods)),
            #[cfg(feature = "mania")]
            GameMode::MNA => Self::Mania(ManiaGradualDifficultyAttributes::new(map, mods)),
            #[allow(unreachable_patterns)]
            _ => panic!("feature for mode {:?} is not enabled", map.mode),
        }
    }
}

impl Iterator for GradualDifficultyAttributes<'_> {
    type Item = DifficultyAttributes;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            #[cfg(feature = "fruits")]
            GradualDifficultyAttributes::Fruits(f) => f.next().map(DifficultyAttributes::Fruits),
            #[cfg(feature = "mania")]
            GradualDifficultyAttributes::Mania(m) => m.next().map(DifficultyAttributes::Mania),
            #[cfg(feature = "osu")]
            GradualDifficultyAttributes::Osu(o) => o.next().map(DifficultyAttributes::Osu),
            #[cfg(feature = "taiko")]
            GradualDifficultyAttributes::Taiko(t) => t.next().map(DifficultyAttributes::Taiko),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            #[cfg(feature = "fruits")]
            GradualDifficultyAttributes::Fruits(f) => f.size_hint(),
            #[cfg(feature = "mania")]
            GradualDifficultyAttributes::Mania(m) => m.size_hint(),
            #[cfg(feature = "osu")]
            GradualDifficultyAttributes::Osu(o) => o.size_hint(),
            #[cfg(feature = "taiko")]
            GradualDifficultyAttributes::Taiko(t) => t.size_hint(),
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
    /// Note that for osu!ctb only fruits and droplets are considered for combo.
    ///
    /// Irrelevant for osu!mania.
    pub max_combo: usize,
    /// Amount of current katus (tiny droplet misses for osu!ctb).
    ///
    /// Only relevant for osu!ctb.
    pub n_katu: usize,
    /// Amount of current 300s (fruits for osu!ctb).
    ///
    /// Irrelevant for osu!mania.
    pub n300: usize,
    /// Amount of current 100s (droplets for osu!ctb).
    ///
    /// Irrelevant for osu!mania.
    pub n100: usize,
    /// Amount of current 50s (tiny droplets for osu!ctb).
    ///
    /// Irrelevant for osu!taiko and osu!mania.
    pub n50: usize,
    /// Amount of current misses (fruits + droplets for osu!ctb).
    ///
    /// Irrelevant for osu!mania.
    pub misses: usize,
    /// The current score.
    ///
    /// Only relevant for osu!mania.
    pub score: u32,
}

impl ScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "fruits")]
impl From<ScoreState> for FruitsScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n_fruits: state.n300,
            n_droplets: state.n100,
            n_tiny_droplets: state.n50,
            n_tiny_droplet_misses: state.n_katu,
            misses: state.misses,
        }
    }
}

#[cfg(feature = "osu")]
impl From<ScoreState> for OsuScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            misses: state.misses,
        }
    }
}

#[cfg(feature = "taiko")]
impl From<ScoreState> for TaikoScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            misses: state.misses,
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
/// [`FruitsGradualPerformanceAttributes`],
/// [`ManiaGradualPerformanceAttributes`],
/// [`OsuGradualPerformanceAttributes`], or
/// [`TaikoGradualPerformanceAttributes`].
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
///     state.score += 123;
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
/// state.misses += 1;
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
/// state.score += 987;
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
/// state.score += 123;
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
#[derive(Clone, Debug)]
pub enum GradualPerformanceAttributes<'map> {
    #[cfg(feature = "fruits")]
    /// Gradual osu!ctb performance attributes.
    Fruits(FruitsGradualPerformanceAttributes<'map>),
    #[cfg(feature = "mania")]
    /// Gradual osu!mania performance attributes.
    Mania(ManiaGradualPerformanceAttributes<'map>),
    #[cfg(feature = "osu")]
    /// Gradual osu!standard performance attributes.
    Osu(OsuGradualPerformanceAttributes<'map>),
    #[cfg(feature = "taiko")]
    /// Gradual osu!taiko performance attributes.
    Taiko(TaikoGradualPerformanceAttributes<'map>),
}

impl<'map> GradualPerformanceAttributes<'map> {
    /// Create a new gradual performance calculator for maps of any mode.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        match map.mode {
            #[cfg(feature = "osu")]
            GameMode::STD => Self::Osu(OsuGradualPerformanceAttributes::new(map, mods)),
            #[cfg(feature = "taiko")]
            GameMode::TKO => Self::Taiko(TaikoGradualPerformanceAttributes::new(map, mods)),
            #[cfg(feature = "fruits")]
            GameMode::CTB => Self::Fruits(FruitsGradualPerformanceAttributes::new(map, mods)),
            #[cfg(feature = "mania")]
            GameMode::MNA => Self::Mania(ManiaGradualPerformanceAttributes::new(map, mods)),
            #[allow(unreachable_patterns)]
            _ => panic!("feature for mode {:?} is not enabled", map.mode),
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
            #[cfg(feature = "fruits")]
            GradualPerformanceAttributes::Fruits(f) => f
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Fruits),
            #[cfg(feature = "mania")]
            GradualPerformanceAttributes::Mania(m) => m
                .process_next_n_objects(state.score, n)
                .map(PerformanceAttributes::Mania),
            #[cfg(feature = "osu")]
            GradualPerformanceAttributes::Osu(o) => o
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Osu),
            #[cfg(feature = "taiko")]
            GradualPerformanceAttributes::Taiko(t) => t
                .process_next_n_objects(state.into(), n)
                .map(PerformanceAttributes::Taiko),
        }
    }
}
