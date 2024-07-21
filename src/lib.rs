//! Library to calculate difficulty and performance attributes for all [osu!] gamemodes.
//!
//! A large part of `rosu-pp` is a port of [osu!lazer]'s difficulty and performance calculation
//! with emphasis on a precise translation to Rust for the most [accurate results](#accuracy)
//! while also providing a significant [boost in performance](#speed).
//!
//! Last commits of the ported code:
//!   - [osu!lazer] : `7342fb7f51b34533a42bffda89c3d6c569cc69ce` (2022-10-11)
//!   - [osu!tools] : `146d5916937161ef65906aa97f85d367035f3712` (2022-10-08)
//!
//! News posts of the latest gamemode updates:
//!   - osu: <https://osu.ppy.sh/home/news/2022-09-30-changes-to-osu-sr-and-pp>
//!   - taiko: <https://osu.ppy.sh/home/news/2022-09-28-changes-to-osu-taiko-sr-and-pp>
//!   - catch: <https://osu.ppy.sh/home/news/2020-05-14-osucatch-scoring-updates>
//!   - mania: <https://osu.ppy.sh/home/news/2022-10-09-changes-to-osu-mania-sr-and-pp>
//!
//! ## Usage
//!
//! ```
//! // Decode the map
//! let map = rosu_pp::Beatmap::from_path("./resources/2785319.osu").unwrap();
//!
//! // Calculate difficulty attributes
//! let diff_attrs = rosu_pp::Difficulty::new()
//!     .mods(8 + 16) // HDHR
//!     .calculate(&map);
//!
//! let stars = diff_attrs.stars();
//!
//! // Calculate performance attributes
//! let perf_attrs = rosu_pp::Performance::new(diff_attrs)
//!     // To speed up the calculation, we used the previous attributes.
//!     // **Note** that this should only be done if the map and all difficulty
//!     // settings stay the same, otherwise the final attributes will be incorrect!
//!     .mods(24) // HDHR, must be the same as before
//!     .combo(789)
//!     .accuracy(99.2)
//!     .misses(2)
//!     .calculate();
//!
//! let pp = perf_attrs.pp();
//!
//! // Again, we re-use the previous attributes for maximum efficiency.
//! let max_pp = perf_attrs.performance()
//!     .mods(24) // Still the same
//!     .calculate()
//!     .pp();
//!
//! println!("Stars: {stars} | PP: {pp}/{max_pp}");
//! ```
//!
//! ## Gradual calculation
//!
//! Gradually calculating attributes provides an efficient way to process each hitobject
//! separately and calculate the attributes only up to that point.
//!
//! For difficulty attributes, there is [`GradualDifficulty`] which implements `Iterator`
//! and for performance attributes there is [`GradualPerformance`] which requires the current
//! score state.
//!
//! ```
//! use rosu_pp::{Beatmap, GradualPerformance, Difficulty, any::ScoreState};
//!
//! let map = Beatmap::from_path("./resources/1028484.osu").unwrap();
//!
//! let mut gradual = Difficulty::new()
//!     .mods(16 + 64) // HRDT
//!     .clock_rate(1.2)
//!     .gradual_performance(&map);
//!
//! let mut state = ScoreState::new(); // empty state, everything is on 0.
//!
//! // The first 10 hitresults are 300s
//! for _ in 0..10 {
//!     state.n300 += 1;
//!     state.max_combo += 1;
//!     let attrs = gradual.next(state.clone()).unwrap();
//!     println!("PP: {}", attrs.pp());
//! }
//!
//! // Fast-forward to the end
//! # /*
//! state.max_combo = ...
//! state.n300 = ...
//! state.n_katu = ...
//! ...
//! # */
//! let attrs = gradual.last(state).unwrap();
//! println!("PP: {}", attrs.pp());
//! ```
//!
//! ## Accuracy
//!
//! `rosu-pp` was tested against all current beatmaps on multiple mod combinations and delivered
//! values that matched osu!lazer perfectly down to the last decimal place.
//!
//! However, there is one small caveat: the values are only this precise on debug mode.
//! On release mode, Rust's compiler performs optimizations that produce the tiniest discrepancies
//! due to floating point inaccuracies which can cascade into larger differences in the end.
//! With this in mind, `rosu-pp` is still as accurate as can be without targeting the
//! .NET compiler itself. Realistically, the inaccuracies in release mode are negligibly small.
//!
//! ## Speed
//!
//! An important factor for `rosu-pp` is the calculation speed. Optimizations and an accurate translation
//! unfortunately don't always go hand-in-hand. Nonetheless, performance improvements are still
//! snuck in wherever possible, providing a significantly faster runtime than the native C# code.
//!
//! Results of a rudimentary [benchmark] of osu!lazer and rosu-pp:
//! ```txt
//! osu!lazer:
//! Decoding maps:            Median: 378.10ms | Mean: 381.47ms
//! Calculating difficulties: Median: 588.89ms | Mean: 597.11ms
//! Calculating performances: Median: 315.90µs | Mean: 310.60µs
//!
//! rosu-pp:
//! Decoding maps:            Median: 46.94ms | Mean: 47.21ms
//! Calculating difficulties: Median: 72.90ms | Mean: 73.13ms
//! Calculating performances: Median: 44.13µs | Mean: 45.53µs
//! ```
//!
//! ## Features
//!
//! | Flag              | Description                           | Dependencies
//! | ----------------- | ------------------------------------- | ------------
//! | `default`         | Enables the `compact_strains` feature |
//! | `compact_strains` | Storing internal strain values in a plain Vec introduces an out-of-memory risk on maliciously long maps (see [/b/3739922](https://osu.ppy.sh/b/3739922)). This feature stores strains more compactly, but comes with a ~5% loss in performance. |
//! | `sync`            | Some gradual calculation types can only be shared across threads if this feature is enabled. This adds a performance penalty so only enable this if really needed. |
//! | `tracing`         | Any error encountered during beatmap decoding will be logged through `tracing::error`. If this feature is **not** enabled, errors will be ignored. | [`tracing`]
//!
//! ## Bindings
//!
//! Using `rosu-pp` from other languages than Rust:
//! - JavaScript: [rosu-pp-js]
//! - Python: [rosu-pp-py]
//!
//! [osu!]: https://osu.ppy.sh/home
//! [osu!lazer]: https://github.com/ppy/osu
//! [osu!tools]: https://github.com/ppy/osu-tools
//! [`tracing`]: https://docs.rs/tracing
//! [rosu-pp-js]: https://github.com/MaxOhn/rosu-pp-js
//! [rosu-pp-py]: https://github.com/MaxOhn/rosu-pp-py
//! [benchmark]: https://gist.github.com/MaxOhn/625af10011f6d7e13a171b08ccf959ff
//! [`GradualDifficulty`]: crate::any::GradualDifficulty
//! [`GradualPerformance`]: crate::any::GradualPerformance

#![deny(rustdoc::broken_intra_doc_links, rustdoc::missing_crate_level_docs)]
#![warn(clippy::missing_const_for_fn, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools,
    clippy::match_same_arms,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::explicit_iter_loop,
    clippy::similar_names,
    clippy::cast_possible_wrap
)]

#[doc(inline)]
pub use self::{
    any::{Difficulty, GradualDifficulty, GradualPerformance, Performance},
    model::{
        beatmap::{Beatmap, Converted},
        mods::GameMods,
    },
};

/// Types for calculations of any mode.
pub mod any;

/// Types for osu!standard calculations.
pub mod osu;

/// Types for osu!taiko calculations.
pub mod taiko;

/// Types for osu!catch calculations.
pub mod catch;

/// Types for osu!mania calculations.
pub mod mania;

/// Types used in and around this crate.
pub mod model;

mod util;

pub use catch::{CatchPP, CatchStars};
pub use mania::{ManiaPP, ManiaStars};
pub use osu::{OsuPP, OsuStars};
pub use taiko::{TaikoPP, TaikoStars};

pub use mods::Mods;
pub use parse::{ParseError, ParseResult};
pub use util::SortedVec;

use wasm_bindgen::prelude::*;

/// The result of calculating the strains on a map.
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug)]
pub enum Strains {
    /// osu!standard strain values.
    Osu(osu::OsuStrains),
    /// osu!taiko strain values.
    Taiko(taiko::TaikoStrains),
    /// osu!catch strain values.
    Catch(catch::CatchStrains),
    /// osu!mania strain values.
    Mania(mania::ManiaStrains),
}

impl Strains {
    /// Time in ms inbetween two strains.
    #[inline]
    pub fn section_len(&self) -> f64 {
        match self {
            Strains::Osu(strains) => strains.section_len,
            Strains::Taiko(strains) => strains.section_len,
            Strains::Catch(strains) => strains.section_len,
            Strains::Mania(strains) => strains.section_len,
        }
    }

    /// Returns the number of strain peaks per skill.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Strains::Osu(strains) => strains.len(),
            Strains::Taiko(strains) => strains.len(),
            Strains::Catch(strains) => strains.len(),
            Strains::Mania(strains) => strains.len(),
        }
    }
}

/// The result of a difficulty calculation based on the mode.
#[derive(Clone, Debug)]
pub enum DifficultyAttributes {
    /// osu!standard difficulty calculation result.
    Osu(osu::OsuDifficultyAttributes),
    /// osu!taiko difficulty calculation result.
    Taiko(taiko::TaikoDifficultyAttributes),
    /// osu!catch difficulty calculation result.
    Catch(catch::CatchDifficultyAttributes),
    /// osu!mania difficulty calculation result.
    Mania(mania::ManiaDifficultyAttributes),
}

impl DifficultyAttributes {
    /// The star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        match self {
            Self::Osu(attrs) => attrs.stars,
            Self::Taiko(attrs) => attrs.stars,
            Self::Catch(attrs) => attrs.stars,
            Self::Mania(attrs) => attrs.stars,
        }
    }

    /// The maximum combo of the map.
    #[inline]
    pub fn max_combo(&self) -> usize {
        match self {
            Self::Osu(attrs) => attrs.max_combo,
            Self::Taiko(attrs) => attrs.max_combo,
            Self::Catch(attrs) => attrs.max_combo(),
            Self::Mania(attrs) => attrs.max_combo,
        }
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

/// The result of a performance calculation based on the mode.
#[derive(Clone, Debug)]
pub enum PerformanceAttributes {
    /// osu!standard performance calculation result.
    Osu(osu::OsuPerformanceAttributes),
    /// osu!taiko performance calculation result.
    Taiko(taiko::TaikoPerformanceAttributes),
    /// osu!catch performance calculation result.
    Catch(catch::CatchPerformanceAttributes),
    /// osu!mania performance calculation result.
    Mania(mania::ManiaPerformanceAttributes),
}

impl PerformanceAttributes {
    /// The pp value.
    #[inline]
    pub fn pp(&self) -> f64 {
        match self {
            Self::Osu(attrs) => attrs.pp,
            Self::Taiko(attrs) => attrs.pp,
            Self::Catch(attrs) => attrs.pp,
            Self::Mania(attrs) => attrs.pp,
        }
    }

    /// The star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        match self {
            Self::Osu(attrs) => attrs.stars(),
            Self::Taiko(attrs) => attrs.stars(),
            Self::Catch(attrs) => attrs.stars(),
            Self::Mania(attrs) => attrs.stars(),
        }
    }

    /// Difficulty attributes that were used for the performance calculation.
    #[inline]
    pub fn difficulty_attributes(&self) -> DifficultyAttributes {
        match self {
            Self::Osu(attrs) => DifficultyAttributes::Osu(attrs.difficulty.clone()),
            Self::Taiko(attrs) => DifficultyAttributes::Taiko(attrs.difficulty.clone()),
            Self::Catch(attrs) => DifficultyAttributes::Catch(attrs.difficulty.clone()),
            Self::Mania(attrs) => DifficultyAttributes::Mania(attrs.difficulty),
        }
    }

    #[inline]
    /// The maximum combo of the map.
    pub fn max_combo(&self) -> usize {
        match self {
            Self::Osu(attrs) => attrs.difficulty.max_combo,
            Self::Taiko(attrs) => attrs.difficulty.max_combo,
            Self::Catch(attrs) => attrs.difficulty.max_combo(),
            Self::Mania(attrs) => attrs.difficulty.max_combo,
        }
    }
}

impl From<PerformanceAttributes> for DifficultyAttributes {
    #[inline]
    fn from(attributes: PerformanceAttributes) -> Self {
        match attributes {
            PerformanceAttributes::Osu(attrs) => Self::Osu(attrs.difficulty),
            PerformanceAttributes::Taiko(attrs) => Self::Taiko(attrs.difficulty),
            PerformanceAttributes::Catch(attrs) => Self::Catch(attrs.difficulty),
            PerformanceAttributes::Mania(attrs) => Self::Mania(attrs.difficulty),
        }
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

#[cfg(all(feature = "async_tokio", feature = "async_std"))]
compile_error!("Only one of the features `async_tokio` and `async_std` should be enabled");

/// Calculates online SR locally
#[wasm_bindgen]
pub fn calculate_sr(str: &[u8], mods: u32) -> f32{

    let sr = match Beatmap::from_bytes(str){
        Ok(map) => map.stars().mods(mods).calculate().stars(),
        Err(_reason) => -1.0,
    };

    sr as f32
}

