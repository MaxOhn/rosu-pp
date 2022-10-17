//! A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.
//!
//! Async is supported through features, see below.
//!
//! ## Usage
//!
//! ```no_run
//! use rosu_pp::{Beatmap, BeatmapExt};
//!
//! # /*
//! // Parse the map yourself
//! let map = match Beatmap::from_path("/path/to/file.osu") {
//!     Ok(map) => map,
//!     Err(why) => panic!("Error while parsing map: {}", why),
//! };
//! # */ let map = Beatmap::default();
//!
//! // If `BeatmapExt` is included, you can make use of
//! // some methods on `Beatmap` to make your life simpler.
//! let result = map.pp()
//!     .mods(24) // HDHR
//!     .combo(1234)
//!     .misses(2)
//!     .accuracy(99.2) // should be called last
//!     .calculate();
//!
//! println!("PP: {}", result.pp());
//!
//! // If you intend to reuse the current map-mod combination,
//! // make use of the previous result!
//! // If attributes are given, then stars & co don't have to be recalculated.
//! let next_result = map.pp()
//!     .mods(24) // HDHR
//!     .attributes(result) // recycle
//!     .combo(543)
//!     .misses(5)
//!     .n50(3)
//!     .accuracy(96.5)
//!     .calculate();
//!
//! println!("Next PP: {}", next_result.pp());
//!
//! let stars = map.stars()
//!     .mods(16)  // HR
//!     .calculate()
//!     .stars();
//!
//! let max_pp = map.max_pp(16).pp();
//!
//! println!("Stars: {} | Max PP: {}", stars, max_pp);
//! ```
//!
//! ## With async
//! If either the `async_tokio` or `async_std` feature is enabled, beatmap parsing will be async.
//!
//! ```no_run
//! use rosu_pp::{Beatmap, BeatmapExt};
//!
//! # /*
//! // Parse the map asynchronously
//! let map = match Beatmap::from_path("/path/to/file.osu").await {
//!     Ok(map) => map,
//!     Err(why) => panic!("Error while parsing map: {}", why),
//! };
//! # */ let map = Beatmap::default();
//!
//! // The rest stays the same
//! let result = map.pp()
//!     .mods(24) // HDHR
//!     .combo(1234)
//!     .misses(2)
//!     .accuracy(99.2)
//!     .calculate();
//!
//! println!("PP: {}", result.pp());
//! ```
//!
//! ## Gradual calculation
//! Sometimes you might want to calculate the difficulty of a map or performance of a score after each hit object.
//! This could be done by using `passed_objects` as the amount of objects that were passed so far.
//! However, this requires to recalculate the beginning again and again, we can be more efficient than that.
//!
//! Instead, you should use [`GradualDifficultyAttributes`] and [`GradualPerformanceAttributes`]:
//!
//! ```no_run
//! use rosu_pp::{
//!     Beatmap, BeatmapExt, GradualPerformanceAttributes, ScoreState,
//!     taiko::TaikoScoreState,
//! };
//!
//! let map = match Beatmap::from_path("/path/to/file.osu") {
//!     Ok(map) => map,
//!     Err(why) => panic!("Error while parsing map: {}", why),
//! };
//!
//! let mods = 8 + 64; // HDDT
//!
//! // If you're only interested in the star rating or other difficulty value,
//! // use `GradualDifficultyAttributes`, either through its function `new`
//! // or through the method `BeatmapExt::gradual_difficulty`.
//! let gradual_difficulty = map.gradual_difficulty(mods);
//!
//! // Since `GradualDifficultyAttributes` implements `Iterator`, you can use
//! // any iterate function on it, use it in loops, collect them into a `Vec`, ...
//! for (i, difficulty) in gradual_difficulty.enumerate() {
//!     println!("Stars after object {}: {}", i, difficulty.stars());
//! }
//!
//! // Gradually calculating performance values does the same as calculating
//! // difficulty attributes but it goes the extra step and also evaluates
//! // the state of a score for these difficulty attributes.
//! let mut gradual_performance = map.gradual_performance(mods);
//!
//! // The default score state is kinda chunky because it considers all modes.
//! let state = ScoreState {
//!     max_combo: 1,
//!     n_katu: 0, // only relevant for ctb
//!     n300: 1,
//!     n100: 0,
//!     n50: 0,
//!     misses: 0,
//!     score: 300, // only relevant for mania
//! };
//!
//! // Process the score state after the first object
//! let curr_performance = match gradual_performance.process_next_object(state) {
//!     Some(perf) => perf,
//!     None => panic!("the map has no hit objects"),
//! };
//!
//! println!("PP after the first object: {}", curr_performance.pp());
//!
//! // If you're only interested in maps of a specific mode, consider
//! // using the mode's gradual calculator instead of the general one.
//! // Let's assume it's a taiko map.
//! // Instead of starting off with `BeatmapExt::gradual_performance` one could have
//! // created the struct via `TaikoGradualPerformanceAttributes::new`.
//! let mut gradual_performance = match gradual_performance {
//!     GradualPerformanceAttributes::Taiko(gradual) => gradual,
//!     _ => panic!("the map was not taiko but {:?}", map.mode),
//! };
//!
//! // A little simpler than the general score state.
//! let state = TaikoScoreState {
//!     max_combo: 11,
//!     n300: 9,
//!     n100: 1,
//!     misses: 1,
//! };
//!
//! // Process the next 10 objects in one go
//! let curr_performance = match gradual_performance.process_next_n_objects(state, 10) {
//!     Some(perf) => perf,
//!     None => panic!("the last `process_next_object` already processed the last object"),
//! };
//!
//! println!("PP after the first 11 objects: {}", curr_performance.pp());
//! ```
//!
//! ## Features
//!
//! | Flag | Description |
//! |-----|-----|
//! | `default` | Beatmap parsing will be non-async |
//! | `async_tokio` | Beatmap parsing will be async through [tokio](https://github.com/tokio-rs/tokio) |
//! | `async_std` | Beatmap parsing will be async through [async-std](https://github.com/async-rs/async-std) |
//!

#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![deny(
    clippy::all,
    nonstandard_style,
    rust_2018_idioms,
    unused,
    warnings,
    missing_debug_implementations,
    missing_docs
)]

/// Everything about osu!catch.
pub mod catch;

/// Everything about osu!mania.
pub mod mania;

/// Everything about osu!standard.
pub mod osu;

/// Everything about osu!taiko.
pub mod taiko;

/// Beatmap parsing
pub mod parse;

/// Beatmap and contained types
pub mod beatmap;
pub use beatmap::{Beatmap, GameMode};

mod gradual;
pub use gradual::{GradualDifficultyAttributes, GradualPerformanceAttributes, ScoreState};

mod pp;
pub use pp::{AnyPP, AttributeProvider, HitResultPriority};

mod stars;
pub use stars::AnyStars;

mod curve;
mod limited_queue;
mod mods;

pub use catch::{CatchPP, CatchStars};
pub use mania::{ManiaPP, ManiaStars};
pub use osu::{OsuPP, OsuStars};
pub use taiko::{TaikoPP, TaikoStars};

pub use mods::Mods;
pub use parse::{ParseError, ParseResult};

/// Provides some additional methods on [`Beatmap`](crate::Beatmap).
pub trait BeatmapExt {
    /// Calculate the stars and other attributes of a beatmap which are required for pp calculation.
    fn stars(&self) -> AnyStars<'_>;

    /// Calculate the max pp of a beatmap.
    ///
    /// If you seek more fine-tuning you can use the [`pp`](BeatmapExt::pp) method.
    fn max_pp(&self, mods: u32) -> PerformanceAttributes;

    /// Returns a builder for performance calculation.
    ///
    /// Convenient method that matches on the map's mode to choose the appropriate calculator.
    fn pp(&self) -> AnyPP<'_>;

    /// Calculate the strains of a map.
    /// This essentially performs the same calculation as [`BeatmapExt::stars`] but
    /// instead of evaluating the final strains, they are just returned as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    fn strains(&self, mods: u32) -> Strains;

    /// Return an iterator that gives you the [`DifficultyAttributes`] after each hit object.
    ///
    /// Suitable to efficiently get the map's star rating after multiple different locations.
    fn gradual_difficulty(&self, mods: u32) -> GradualDifficultyAttributes<'_>;

    /// Return a struct that gives you the [`PerformanceAttributes`] after every (few) hit object(s).
    ///
    /// Suitable to efficiently get a score's performance after multiple different locations,
    /// i.e. live update a score's pp.
    fn gradual_performance(&self, mods: u32) -> GradualPerformanceAttributes<'_>;
}

impl BeatmapExt for Beatmap {
    #[inline]
    fn stars(&self) -> AnyStars<'_> {
        match self.mode {
            GameMode::Osu => AnyStars::Osu(OsuStars::new(self)),
            GameMode::Mania => AnyStars::Mania(ManiaStars::new(self)),
            GameMode::Taiko => AnyStars::Taiko(TaikoStars::new(self)),
            GameMode::Catch => AnyStars::Catch(CatchStars::new(self)),
        }
    }

    #[inline]
    fn max_pp(&self, mods: u32) -> PerformanceAttributes {
        match self.mode {
            GameMode::Osu => PerformanceAttributes::Osu(OsuPP::new(self).mods(mods).calculate()),
            GameMode::Mania => {
                PerformanceAttributes::Mania(ManiaPP::new(self).mods(mods).calculate())
            }
            GameMode::Taiko => {
                PerformanceAttributes::Taiko(TaikoPP::new(self).mods(mods).calculate())
            }
            GameMode::Catch => {
                PerformanceAttributes::Catch(CatchPP::new(self).mods(mods).calculate())
            }
        }
    }

    #[inline]
    fn pp(&self) -> AnyPP<'_> {
        AnyPP::new(self)
    }

    #[inline]
    fn strains(&self, mods: u32) -> Strains {
        match self.mode {
            GameMode::Osu => Strains::Osu(OsuStars::new(self).mods(mods).strains()),
            GameMode::Mania => Strains::Mania(ManiaStars::new(self).mods(mods).strains()),
            GameMode::Taiko => Strains::Taiko(TaikoStars::new(self).mods(mods).strains()),
            GameMode::Catch => Strains::Catch(CatchStars::new(self).mods(mods).strains()),
        }
    }

    #[inline]
    fn gradual_difficulty(&self, mods: u32) -> GradualDifficultyAttributes<'_> {
        GradualDifficultyAttributes::new(self, mods)
    }

    #[inline]
    fn gradual_performance(&self, mods: u32) -> GradualPerformanceAttributes<'_> {
        GradualPerformanceAttributes::new(self, mods)
    }
}

/// The result of calculating the strains on a map.
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug)]
pub enum Strains {
    /// osu!catch strain values.
    Catch(catch::CatchStrains),
    /// osu!mania strain values.
    Mania(mania::ManiaStrains),
    /// osu!standard strain values.
    Osu(osu::OsuStrains),
    /// osu!taiko strain values.
    Taiko(taiko::TaikoStrains),
}

impl Strains {
    /// Time in ms inbetween two strains.
    #[inline]
    pub fn section_len(&self) -> f64 {
        match self {
            Strains::Catch(strains) => strains.section_len,
            Strains::Mania(strains) => strains.section_len,
            Strains::Osu(strains) => strains.section_len,
            Strains::Taiko(strains) => strains.section_len,
        }
    }

    /// Returns the number of strain peaks per skill.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Strains::Catch(strains) => strains.len(),
            Strains::Mania(strains) => strains.len(),
            Strains::Osu(strains) => strains.len(),
            Strains::Taiko(strains) => strains.len(),
        }
    }
}

/// The result of a difficulty calculation based on the mode.
#[derive(Clone, Debug)]
pub enum DifficultyAttributes {
    /// osu!catch difficulty calculation result.
    Catch(catch::CatchDifficultyAttributes),
    /// osu!mania difficulty calculation result.
    Mania(mania::ManiaDifficultyAttributes),
    /// osu!standard difficulty calculation result.
    Osu(osu::OsuDifficultyAttributes),
    /// osu!taiko difficulty calculation result.
    Taiko(taiko::TaikoDifficultyAttributes),
}

impl DifficultyAttributes {
    /// The star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        match self {
            Self::Catch(attributes) => attributes.stars,
            Self::Mania(attributes) => attributes.stars,
            Self::Osu(attributes) => attributes.stars,
            Self::Taiko(attributes) => attributes.stars,
        }
    }

    /// The maximum combo of the map.
    ///
    /// This will only be `None` for attributes of osu!mania maps.
    #[inline]
    pub fn max_combo(&self) -> Option<usize> {
        match self {
            Self::Catch(attributes) => Some(attributes.max_combo()),
            Self::Mania(_) => None,
            Self::Osu(attributes) => Some(attributes.max_combo),
            Self::Taiko(attributes) => Some(attributes.max_combo),
        }
    }
}

impl From<catch::CatchDifficultyAttributes> for DifficultyAttributes {
    #[inline]
    fn from(attributes: catch::CatchDifficultyAttributes) -> Self {
        Self::Catch(attributes)
    }
}

impl From<mania::ManiaDifficultyAttributes> for DifficultyAttributes {
    #[inline]
    fn from(attributes: mania::ManiaDifficultyAttributes) -> Self {
        Self::Mania(attributes)
    }
}

impl From<osu::OsuDifficultyAttributes> for DifficultyAttributes {
    #[inline]
    fn from(attributes: osu::OsuDifficultyAttributes) -> Self {
        Self::Osu(attributes)
    }
}

impl From<taiko::TaikoDifficultyAttributes> for DifficultyAttributes {
    #[inline]
    fn from(attributes: taiko::TaikoDifficultyAttributes) -> Self {
        Self::Taiko(attributes)
    }
}

/// The result of a performance calculation based on the mode.
#[derive(Clone, Debug)]
pub enum PerformanceAttributes {
    /// osu!catch performance calculation result.
    Catch(catch::CatchPerformanceAttributes),
    /// osu!mania performance calculation result.
    Mania(mania::ManiaPerformanceAttributes),
    /// osu!standard performance calculation result.
    Osu(osu::OsuPerformanceAttributes),
    /// osu!taiko performance calculation result.
    Taiko(taiko::TaikoPerformanceAttributes),
}

impl PerformanceAttributes {
    /// The pp value.
    #[inline]
    pub fn pp(&self) -> f64 {
        match self {
            Self::Catch(attributes) => attributes.pp,
            Self::Mania(attributes) => attributes.pp,
            Self::Osu(attributes) => attributes.pp,
            Self::Taiko(attributes) => attributes.pp,
        }
    }

    /// The star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        match self {
            Self::Catch(attributes) => attributes.stars(),
            Self::Mania(attributes) => attributes.stars(),
            Self::Osu(attributes) => attributes.stars(),
            Self::Taiko(attributes) => attributes.stars(),
        }
    }

    /// Difficulty attributes that were used for the performance calculation.
    #[inline]
    pub fn difficulty_attributes(&self) -> DifficultyAttributes {
        match self {
            Self::Catch(attributes) => DifficultyAttributes::Catch(attributes.difficulty.clone()),
            Self::Mania(attributes) => DifficultyAttributes::Mania(attributes.difficulty),
            Self::Osu(attributes) => DifficultyAttributes::Osu(attributes.difficulty.clone()),
            Self::Taiko(attributes) => DifficultyAttributes::Taiko(attributes.difficulty.clone()),
        }
    }

    #[inline]
    /// The maximum combo of the map.
    ///
    /// This will only be `None` for attributes of osu!mania maps.
    pub fn max_combo(&self) -> Option<usize> {
        match self {
            Self::Catch(f) => Some(f.difficulty.max_combo()),
            Self::Mania(_) => None,
            Self::Osu(o) => Some(o.difficulty.max_combo),
            Self::Taiko(t) => Some(t.difficulty.max_combo),
        }
    }
}

impl From<PerformanceAttributes> for DifficultyAttributes {
    #[inline]
    fn from(attributes: PerformanceAttributes) -> Self {
        match attributes {
            PerformanceAttributes::Catch(attributes) => Self::Catch(attributes.difficulty),
            PerformanceAttributes::Mania(attributes) => Self::Mania(attributes.difficulty),
            PerformanceAttributes::Osu(attributes) => Self::Osu(attributes.difficulty),
            PerformanceAttributes::Taiko(attributes) => Self::Taiko(attributes.difficulty),
        }
    }
}

impl From<catch::CatchPerformanceAttributes> for PerformanceAttributes {
    #[inline]
    fn from(attributes: catch::CatchPerformanceAttributes) -> Self {
        Self::Catch(attributes)
    }
}

impl From<mania::ManiaPerformanceAttributes> for PerformanceAttributes {
    #[inline]
    fn from(attributes: mania::ManiaPerformanceAttributes) -> Self {
        Self::Mania(attributes)
    }
}

impl From<osu::OsuPerformanceAttributes> for PerformanceAttributes {
    #[inline]
    fn from(attributes: osu::OsuPerformanceAttributes) -> Self {
        Self::Osu(attributes)
    }
}

impl From<taiko::TaikoPerformanceAttributes> for PerformanceAttributes {
    #[inline]
    fn from(attributes: taiko::TaikoPerformanceAttributes) -> Self {
        Self::Taiko(attributes)
    }
}

#[cfg(all(feature = "async_tokio", feature = "async_std"))]
compile_error!("Only one of the features `async_tokio` and `async_std` should be enabled");

#[cfg(test)]
mod tests {
    use crate::{Beatmap, GameMode, OsuPP, PerformanceAttributes};

    #[test]
    fn custom() {
        let path = "F:/osu!/beatmaps/1000168.osu";
        let map = Beatmap::from_path(path).unwrap();

        let attrs = match OsuPP::new(&map).mode(GameMode::Mania).mods(0).calculate() {
            PerformanceAttributes::Mania(attrs) => attrs,
            _ => unreachable!(),
        };

        println!(
            "difficulty:\n\
            stars={}\n\
            max_combo={}\n\
            performance:\n\
            difficulty={}\n\
            pp={}\n",
            attrs.difficulty.stars, attrs.difficulty.max_combo, attrs.pp_difficulty, attrs.pp,
        );
    }
}
