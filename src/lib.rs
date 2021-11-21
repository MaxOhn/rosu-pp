//! A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.
//!
//! Conversions between gamemodes are generally not supported.
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
//! ## Features
//!
//! | Flag | Description |
//! |-----|-----|
//! | `default` | Enable all modes. |
//! | `osu` | Enable osu!standard. |
//! | `taiko` | Enable osu!taiko. |
//! | `fruits` | Enable osu!ctb. |
//! | `mania` | Enable osu!mania. |
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

#[cfg(feature = "fruits")]
#[cfg_attr(docsrs, doc(cfg(feature = "fruits")))]
/// Everything about osu!ctb.
pub mod fruits;

#[cfg(feature = "mania")]
#[cfg_attr(docsrs, doc(cfg(feature = "mania")))]
/// Everything about osu!mania.
pub mod mania;

#[cfg(feature = "osu")]
#[cfg_attr(docsrs, doc(cfg(feature = "osu")))]
/// Everything about osu!standard.
pub mod osu;

#[cfg(feature = "taiko")]
#[cfg_attr(docsrs, doc(cfg(feature = "taiko")))]
/// Everything about osu!taiko.
pub mod taiko;

/// Beatmap parsing and the contained types.
pub mod parse;

mod pp;
pub use pp::{AnyPP, AttributeProvider};

mod curve;
mod mods;

#[cfg(feature = "sliders")]
pub(crate) mod control_point_iter;

#[cfg(feature = "sliders")]
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

/// Provides some additional methods on [`Beatmap`](crate::Beatmap).
pub trait BeatmapExt {
    /// Calculate the stars and other attributes of a beatmap which are required for pp calculation.
    fn stars(&self, mods: impl Mods, passed_objects: Option<usize>) -> DifficultyAttributes;

    /// Calculate the max pp of a beatmap.
    ///
    /// If you seek more fine-tuning you can use the [`pp`](BeatmapExt::pp) method.
    fn max_pp(&self, mods: u32) -> PerformanceAttributes;

    /// Returns a builder for performance calculation.
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
    fn stars(&self, mods: impl Mods, passed_objects: Option<usize>) -> DifficultyAttributes {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                DifficultyAttributes::Osu(osu::stars(self, mods, passed_objects))
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                DifficultyAttributes::Mania(mania::stars(self, mods, passed_objects))
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`taiko` feature is not enabled");

                #[cfg(feature = "taiko")]
                DifficultyAttributes::Taiko(taiko::stars(self, mods, passed_objects))
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                DifficultyAttributes::Fruits(fruits::stars(self, mods, passed_objects))
            }
        }
    }

    fn max_pp(&self, mods: u32) -> PerformanceAttributes {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                PerformanceAttributes::Osu(OsuPP::new(self).mods(mods).calculate())
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                PerformanceAttributes::Mania(ManiaPP::new(self).mods(mods).calculate())
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`taiko` feature is not enabled");

                #[cfg(feature = "taiko")]
                PerformanceAttributes::Taiko(TaikoPP::new(self).mods(mods).calculate())
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                PerformanceAttributes::Fruits(FruitsPP::new(self).mods(mods).calculate())
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
#[derive(Clone, Debug, Default)]
pub struct Strains {
    /// Time in ms inbetween two strains.
    pub section_length: f64,
    /// Summed strains for each skill of the map's mode.
    pub strains: Vec<f64>,
}

/// The result of a difficulty calculation based on the mode.
#[derive(Clone, Debug)]
pub enum DifficultyAttributes {
    #[cfg(feature = "fruits")]
    /// osu!ctb difficulty calculation reseult.
    Fruits(fruits::FruitsDifficultyAttributes),
    #[cfg(feature = "mania")]
    /// osu!mania difficulty calculation reseult.
    Mania(mania::ManiaDifficultyAttributes),
    #[cfg(feature = "osu")]
    /// osu!standard difficulty calculation reseult.
    Osu(osu::OsuDifficultyAttributes),
    #[cfg(feature = "taiko")]
    /// osu!taiko difficulty calculation reseult.
    Taiko(taiko::TaikoDifficultyAttributes),
}

impl DifficultyAttributes {
    /// The star value.
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

    /// The maximum combo of the map.
    ///
    /// This will only be `None` for attributes of osu!mania maps.
    #[inline]
    #[cfg(feature = "mania")]
    pub fn max_combo(&self) -> Option<usize> {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(attributes) => Some(attributes.max_combo),
            Self::Mania(_) => None,
            #[cfg(feature = "osu")]
            Self::Osu(attributes) => Some(attributes.max_combo),
            #[cfg(feature = "taiko")]
            Self::Taiko(attributes) => Some(attributes.max_combo),
        }
    }

    #[cfg(not(feature = "mania"))]
    #[inline]
    /// The maximum combo of the map.
    pub fn max_combo(&self) -> usize {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(attributes) => attributes.max_combo,
            #[cfg(feature = "osu")]
            Self::Osu(attributes) => attributes.max_combo,
            #[cfg(feature = "taiko")]
            Self::Taiko(attributes) => attributes.max_combo,
        }
    }
}

#[cfg(feature = "fruits")]
impl From<fruits::FruitsDifficultyAttributes> for DifficultyAttributes {
    fn from(attributes: fruits::FruitsDifficultyAttributes) -> Self {
        Self::Fruits(attributes)
    }
}

#[cfg(feature = "mania")]
impl From<mania::ManiaDifficultyAttributes> for DifficultyAttributes {
    fn from(attributes: mania::ManiaDifficultyAttributes) -> Self {
        Self::Mania(attributes)
    }
}

#[cfg(feature = "osu")]
impl From<osu::OsuDifficultyAttributes> for DifficultyAttributes {
    fn from(attributes: osu::OsuDifficultyAttributes) -> Self {
        Self::Osu(attributes)
    }
}

#[cfg(feature = "taiko")]
impl From<taiko::TaikoDifficultyAttributes> for DifficultyAttributes {
    fn from(attributes: taiko::TaikoDifficultyAttributes) -> Self {
        Self::Taiko(attributes)
    }
}

/// The result of a performance calculation based on the mode.
#[derive(Clone, Debug)]
pub enum PerformanceAttributes {
    #[cfg(feature = "fruits")]
    /// osu!ctb performance calculation result.
    Fruits(fruits::FruitsPerformanceAttributes),
    #[cfg(feature = "mania")]
    /// osu!mania performance calculation result.
    Mania(mania::ManiaPerformanceAttributes),
    #[cfg(feature = "osu")]
    /// osu!standard performance calculation result.
    Osu(osu::OsuPerformanceAttributes),
    #[cfg(feature = "taiko")]
    /// osu!taiko performance calculation result.
    Taiko(taiko::TaikoPerformanceAttributes),
}

impl PerformanceAttributes {
    /// The pp value.
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

    /// The star value.
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

    /// Difficulty attributes that were used for the performance calculation.
    #[inline]
    pub fn difficulty_attributes(&self) -> DifficultyAttributes {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(attributes) => DifficultyAttributes::Fruits(attributes.attributes.clone()),
            #[cfg(feature = "mania")]
            Self::Mania(attributes) => DifficultyAttributes::Mania(attributes.attributes),
            #[cfg(feature = "osu")]
            Self::Osu(attributes) => DifficultyAttributes::Osu(attributes.attributes.clone()),
            #[cfg(feature = "taiko")]
            Self::Taiko(attributes) => DifficultyAttributes::Taiko(attributes.attributes),
        }
    }

    #[cfg(feature = "mania")]
    #[inline]
    /// The maximum combo of the map.
    ///
    /// This will only be `None` for attributes of osu!mania maps.
    pub fn max_combo(&self) -> Option<usize> {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => Some(f.attributes.max_combo),
            Self::Mania(_) => None,
            #[cfg(feature = "osu")]
            Self::Osu(o) => Some(o.attributes.max_combo),
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => Some(t.attributes.max_combo),
        }
    }

    #[cfg(not(feature = "mania"))]
    #[inline]
    /// The maximum combo of the map.
    pub fn max_combo(&self) -> usize {
        match self {
            #[cfg(feature = "fruits")]
            Self::Fruits(f) => f.attributes.max_combo,
            #[cfg(feature = "osu")]
            Self::Osu(o) => o.attributes.max_combo,
            #[cfg(feature = "taiko")]
            Self::Taiko(t) => t.attributes.max_combo,
        }
    }
}

impl From<PerformanceAttributes> for DifficultyAttributes {
    fn from(attributes: PerformanceAttributes) -> Self {
        match attributes {
            #[cfg(feature = "fruits")]
            PerformanceAttributes::Fruits(attributes) => Self::Fruits(attributes.attributes),
            #[cfg(feature = "mania")]
            PerformanceAttributes::Mania(attributes) => Self::Mania(attributes.attributes),
            #[cfg(feature = "osu")]
            PerformanceAttributes::Osu(attributes) => Self::Osu(attributes.attributes),
            #[cfg(feature = "taiko")]
            PerformanceAttributes::Taiko(attributes) => Self::Taiko(attributes.attributes),
        }
    }
}

#[cfg(feature = "fruits")]
impl From<fruits::FruitsPerformanceAttributes> for PerformanceAttributes {
    fn from(attributes: fruits::FruitsPerformanceAttributes) -> Self {
        Self::Fruits(attributes)
    }
}

#[cfg(feature = "mania")]
impl From<mania::ManiaPerformanceAttributes> for PerformanceAttributes {
    fn from(attributes: mania::ManiaPerformanceAttributes) -> Self {
        Self::Mania(attributes)
    }
}

#[cfg(feature = "osu")]
impl From<osu::OsuPerformanceAttributes> for PerformanceAttributes {
    fn from(attributes: osu::OsuPerformanceAttributes) -> Self {
        Self::Osu(attributes)
    }
}

#[cfg(feature = "taiko")]
impl From<taiko::TaikoPerformanceAttributes> for PerformanceAttributes {
    fn from(attributes: taiko::TaikoPerformanceAttributes) -> Self {
        Self::Taiko(attributes)
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

#[cfg(all(feature = "async_tokio", feature = "async_std"))]
compile_error!("Only one of the features `async_tokio` and `async_std` should be enabled");
