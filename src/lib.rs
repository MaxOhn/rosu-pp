//! A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.
//!
//! Conversions between gamemodes are generally not supported.
//!
//! Async is supported through features, see below.
//!
//! ## Usage
//!
//! ```no_run
//! use std::fs::File;
//! use rosu_pp::{Beatmap, BeatmapExt};
//!
//! # /*
//! let file = match File::open("/path/to/file.osu") {
//!     Ok(file) => file,
//!     Err(why) => panic!("Could not open file: {}", why),
//! };
//!
//! // Parse the map yourself
//! let map = match Beatmap::parse(file) {
//!     Ok(map) => map,
//!     Err(why) => panic!("Error while parsing map: {}", why),
//! };
//! # */ let map = Beatmap::default();
//!
//! // If `BeatmapExt` is included, you can make use of
//! // some methods on `Beatmap` to make your life simpler.
//! // If the mode is known, it is recommended to use the
//! // mode's pp calculator, e.g. `TaikoPP`, manually.
//! let result = map.pp()
//!     .mods(24) // HDHR
//!     .combo(1234)
//!     .misses(2)
//!     .accuracy(99.2)
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
//!     .passed_objects(600)
//!     .accuracy(96.5)
//!     .calculate();
//!
//! println!("Next PP: {}", next_result.pp());
//!
//! let stars = map.stars(16, None).stars(); // HR
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
//! # /*
//! use async_std::fs::File;
//! # */
//! // use tokio::fs::File;
//!
//! # /*
//! let file = match File::open("/path/to/file.osu").await {
//!     Ok(file) => file,
//!     Err(why) => panic!("Could not open file: {}", why),
//! };
//!
//! // Parse the map asynchronously
//! let map = match Beatmap::parse(file).await {
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
//! ## osu!standard versions
//!
//! - `osu_precise`: Both stack leniency & slider paths are considered so that the difficulty and pp calculation immitates osu! as close as possible. Pro: Very accurate values; Con: Less performant.
//! - `osu_fast` (i.e. [oppai](https://github.com/Francesco149/oppai-ng)): Fully ignoring sliders aswell as the positional offset caused by stack leniency. This means the stacked position and travel distance of notes is completely omitted which results in notable inaccuracies but is also considerably faster than `osu_precise`.
//!
//! **Note**: If the `fruits` feature is enabled, sliders will be parsed regardless, resulting in a reduced performance advantage of `osu_fast`. Hence, it is only recommended to use `osu_fast` if `fruits` is not enabled.
//!
//! ## Features
//!
//! | Flag | Description |
//! |-----|-----|
//! | `default` | Enable all modes and choose the `osu_precise` version for osu!standard. |
//! | `taiko` | Enable osu!taiko. |
//! | `fruits` | Enable osu!ctb. |
//! | `mania` | Enable osu!mania. |
//! | `osu_fast` | When calculating difficulty attributes in osu!standard, ignore stack leniency and sliders. Great performance but less precise values. |
//! | `osu_precise` | When calculating difficulty attributes in osu!standard, consider both stack leniency and sliders. Great precision but significantly worse performance than `osu_fast`. |
//! | `async_tokio` | Beatmap parsing will be async through [tokio](https://github.com/tokio-rs/tokio) |
//! | `async_std` | Beatmap parsing will be async through [async-std](https://github.com/async-rs/async-std) |
//!

#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

#[cfg(feature = "fruits")]
#[cfg_attr(docsrs, doc(cfg(feature = "fruits")))]
pub mod fruits;

#[cfg(feature = "mania")]
#[cfg_attr(docsrs, doc(cfg(feature = "mania")))]
pub mod mania;

#[cfg(feature = "osu")]
#[cfg_attr(docsrs, doc(cfg(feature = "osu")))]
pub mod osu;

#[cfg(feature = "taiko")]
#[cfg_attr(docsrs, doc(cfg(feature = "taiko")))]
pub mod taiko;

pub mod parse;

mod pp;
pub use pp::{AnyPP, AttributeProvider};

mod curve;
mod math_util;
mod mods;

#[cfg(any(feature = "osu", feature = "fruits"))]
pub(crate) mod control_point_iter;

#[cfg(any(feature = "osu", feature = "fruits"))]
pub(crate) use control_point_iter::{ControlPoint, ControlPointIter};

#[cfg(feature = "fruits")]
pub use fruits::FruitsPP;

#[cfg(feature = "mania")]
pub use mania::ManiaPP;

#[cfg(feature = "osu")]
pub use osu::OsuPP;

#[cfg(feature = "taiko")]
pub use taiko::TaikoPP;

pub use mods::Mods;
pub use parse::{Beatmap, BeatmapAttributes, GameMode, ParseError, ParseResult};

pub trait BeatmapExt {
    /// Calculate the stars and other attributes of a beatmap which are required for pp calculation.
    fn stars(&self, mods: impl Mods, passed_objects: Option<usize>) -> StarResult;

    /// Calculate the max pp of a beatmap.
    ///
    /// If you seek more fine-tuning and options you need to match on the map's
    /// mode and use the mode's corresponding calculator, e.g. [`TaikoPP`](crate::TaikoPP) for taiko.
    fn max_pp(&self, mods: u32) -> PpResult;

    /// Returns a builder to calculate pp and difficulty values.
    ///
    /// Convenient method that matches on the map's mode to choose the appropriate calculator.
    fn pp(&self) -> AnyPP<'_>;

    /// Calculate the strains of a map.
    /// This essentially performs the same calculation as a `stars` function but
    /// instead of evaluating the final strains, they are just returned as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    fn strains(&self, mods: impl Mods) -> Strains;
}

impl BeatmapExt for Beatmap {
    fn stars(&self, mods: impl Mods, passed_objects: Option<usize>) -> StarResult {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                StarResult::Osu(osu::stars(self, mods, passed_objects))
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                StarResult::Mania(mania::stars(self, mods, passed_objects))
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`taiko` feature is not enabled");

                #[cfg(feature = "taiko")]
                StarResult::Taiko(taiko::stars(self, mods, passed_objects))
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                StarResult::Fruits(fruits::stars(self, mods, passed_objects))
            }
        }
    }

    fn max_pp(&self, mods: u32) -> PpResult {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                PpResult::Osu(OsuPP::new(self).mods(mods).calculate())
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                PpResult::Mania(ManiaPP::new(self).mods(mods).calculate())
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`taiko` feature is not enabled");

                #[cfg(feature = "taiko")]
                PpResult::Taiko(TaikoPP::new(self).mods(mods).calculate())
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                PpResult::Fruits(FruitsPP::new(self).mods(mods).calculate())
            }
        }
    }

    #[inline]
    fn pp(&self) -> AnyPP<'_> {
        AnyPP::new(self)
    }

    fn strains(&self, mods: impl Mods) -> Strains {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                osu::strains(self, mods)
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                mania::strains(self, mods)
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`taiko` feature is not enabled");

                #[cfg(feature = "taiko")]
                taiko::strains(self, mods)
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                fruits::strains(self, mods)
            }
        }
    }
}

/// The result of calculating the strains on a map.
/// Suitable to plot the difficulty of a map over time.
///
/// `strains` will be the summed strains for each skill of the map's mode.
///
/// `section_length` is the time in ms inbetween two strains.
#[derive(Clone, Debug, Default)]
pub struct Strains {
    pub section_length: f64,
    pub strains: Vec<f64>,
}

/// Basic enum containing the result of a star calculation based on the mode.
#[derive(Clone, Debug)]
pub enum StarResult {
    #[cfg(feature = "fruits")]
    Fruits(fruits::DifficultyAttributes),
    #[cfg(feature = "mania")]
    Mania(mania::DifficultyAttributes),
    #[cfg(feature = "osu")]
    Osu(osu::DifficultyAttributes),
    #[cfg(feature = "taiko")]
    Taiko(taiko::DifficultyAttributes),
}

impl StarResult {
    /// The final star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(attributes) => attributes.stars,
            #[cfg(feature = "mania")]
            Self::Mania(attributes) => attributes.stars,
            #[cfg(feature = "osu")]
            Self::Osu(attributes) => attributes.stars,
            #[cfg(feature = "taiko")]
            Self::Taiko(attributes) => attributes.stars,
        }
    }
}

/// Basic struct containing the result of a PP calculation.
#[derive(Clone, Debug)]
pub enum PpResult {
    #[cfg(feature = "fruits")]
    Fruits(fruits::PerformanceAttributes),
    #[cfg(feature = "mania")]
    Mania(mania::PerformanceAttributes),
    #[cfg(feature = "osu")]
    Osu(osu::PerformanceAttributes),
    #[cfg(feature = "taiko")]
    Taiko(taiko::PerformanceAttributes),
}

impl PpResult {
    /// The final pp value.
    #[inline]
    pub fn pp(&self) -> f64 {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(attributes) => attributes.pp,
            #[cfg(feature = "mania")]
            Self::Mania(attributes) => attributes.pp,
            #[cfg(feature = "osu")]
            Self::Osu(attributes) => attributes.pp,
            #[cfg(feature = "taiko")]
            Self::Taiko(attributes) => attributes.pp,
        }
    }

    /// The final star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(attributes) => attributes.stars(),
            #[cfg(feature = "mania")]
            Self::Mania(attributes) => attributes.stars(),
            #[cfg(feature = "osu")]
            Self::Osu(attributes) => attributes.stars(),
            #[cfg(feature = "taiko")]
            Self::Taiko(attributes) => attributes.stars(),
        }
    }
}

#[cfg(any(feature = "osu", feature = "taiko"))]
#[inline]
fn difficulty_range(val: f64, max: f64, avg: f64, min: f64) -> f64 {
    if val > 5.0 {
        avg + (max - avg) * (val - 5.0) / 5.0
    } else if val < 5.0 {
        avg - (avg - min) * (5.0 - val) / 5.0
    } else {
        avg
    }
}

#[cfg(not(any(
    feature = "osu",
    feature = "taiko",
    feature = "fruits",
    feature = "mania"
)))]
compile_error!("At least one of the features `osu`, `taiko`, `fruits`, `mania` must be enabled");

#[cfg(all(
    feature = "osu",
    not(any(feature = "osu_precise", feature = "osu_fast"))
))]
compile_error!(
    "Since the `osu` feature is enabled, either `osu_precise` or `osu_fast` must be enabled aswell"
);

#[cfg(any(all(feature = "osu_precise", feature = "osu_fast"),))]
compile_error!("Only one of the features `osu_precise` and `osu_fast` should be enabled");

#[cfg(all(feature = "async_tokio", feature = "async_std"))]
compile_error!("Only one of the features `async_tokio` and `async_std` should be enabled");
