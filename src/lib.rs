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
