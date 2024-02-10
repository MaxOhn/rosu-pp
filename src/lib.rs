//! Library to calculate difficulty and performance attributes for all [osu!] gamemodes.
//!
//! ## Description
//!
//! A large part of `rosu-pp` is a port of [osu!lazer]'s difficulty and performance calculation
//! with emphasis on a precise translation to rust for the most accurate results.
//!
//! Last commits of the ported code:
//!   - [osu!lazer]: `7342fb7f51b34533a42bffda89c3d6c569cc69ce` (2022-10-11)
//!   - [osu!tools]: `146d5916937161ef65906aa97f85d367035f3712` (2022-10-08)
//!
//! Posts of the latest gamemode updates:
//!   - osu: <https://osu.ppy.sh/home/news/2022-09-30-changes-to-osu-sr-and-pp>
//!   - taiko: <https://osu.ppy.sh/home/news/2022-09-28-changes-to-osu-taiko-sr-and-pp>
//!   - catch: TODO
//!   - mania: <https://osu.ppy.sh/home/news/2022-10-09-changes-to-osu-mania-sr-and-pp>
//!
//! ## Usage
//!
//! ```
//! // Decode the map
//! let map = rosu_pp::Beatmap::from_path("./resources/2785319.osu").unwrap();
//!
//! // Calculate difficulty attributes
//! let diff_attrs = map.difficulty()
//!     .mods(8 + 16) // HDHR
//!     .calculate();
//!
//! let stars = diff_attrs.stars();
//!
//! // Calculate performance attributes
//! let perf_attrs = map.performance()
//!     // To speed up the calculation significantly, we can re-use the previous
//!     // attributes.
//!     // **Note** that this should only be done if the map, mode, mods, and
//!     // amount of passed objects stay the same. Otherwise, the resulting
//!     // attributes will be incorrect.
//!     .attributes(diff_attrs)
//!     .mods(24) // HDHR, same as before
//!     .combo(789)
//!     .accuracy(99.2)
//!     .misses(2)
//!     .calculate();
//!
//! let pp = perf_attrs.pp();
//!
//! // Again, we re-use the previous attributes for maximum efficiency.
//! // This time we do it directly instead of through the map.
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
//! TODO
//!
//! ## Features
//!
//! | Flag | Description | Dependencies
//! | - | - | -
//! | `default` | No features |
//! | `tracing` | Any error encountered during beatmap decoding will be logged through `tracing::error`. If this features is not enabled, errors will be ignored. | [`tracing`]
//!
//! [osu!]: https://osu.ppy.sh/home
//! [osu!lazer]: https://github.com/ppy/osu
//! [osu!tools]: https://github.com/ppy/osu-tools
//! [`tracing`]: https://docs.rs/tracing

#![deny(rustdoc::broken_intra_doc_links)]
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
    any::{Difficulty, ModeDifficulty, Performance},
    model::beatmap::{Beatmap, Converted},
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

// TODO: fix very low custom clock rates
