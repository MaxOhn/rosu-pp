//! A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.
//!
//! Conversions are generally not supported.
//!
//! ### Usage
//! ```rust,no_run
//! use std::fs::File;
//! use rosu_pp::{Beatmap, BeatmapExt};
//!
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
//!
//! // If `BeatmapExt` is included, you can make use of
//! // some methods on `Beatmap` to make your life simpler.
//! // However, to calculate specific pp values, it is recommended
//! // to match on the map's mode yourself and modify the mode's
//! // pp calculator, e.g. `TaikoPP`, manually.
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
//! ### osu!standard versions
//! - `all_included`: Both stack leniency & slider paths are considered so that the difficulty and pp calculation immitates osu! as close as possible. Pro: Most precise; Con: Least performant.
//! - `no_leniency`: The positional offset of notes created by stack leniency is not considered. This means the jump distance inbetween notes might be slightly off, resulting in small inaccuracies. Since calculating these offsets is relatively expensive though, this version is considerably faster than `all_included`.
//! - `no_slider_no_leniency` (i.e. [oppai](https://github.com/Francesco149/oppai-ng)): In addition to not considering the positional offset caused by stack leniency, slider paths are also ignored. This means the travel distance of notes is completely omitted which may cause further inaccuracies. Since the slider paths don't have to be computed though, it is generally faster than `no_leniency`.
//!
//! ### Features
//!
//! | Flag | Description |
//! |-----|-----|
//! | `default` | Enable all modes and choose the `no_leniency` version for osu!standard. |
//! | `taiko` | Enable osu!taiko. |
//! | `fruits` | Enable osu!ctb. |
//! | `mania` | Enable osu!mania. |
//! | `osu` | Enable osu!standard. Requires to also enable exactly one of the features `no_leniency`, `no_sliders_no_leniency`, or `all_included`. |
//! | `no_leniency` | When calculating difficulty attributes in osu!standard, ignore stack leniency but consider sliders. Solid middleground between performance and precision, suggested default version. |
//! | `no_sliders_no_leniency` | When calculating difficulty attributes in osu!standard, ignore stack leniency and sliders. Best performance but slightly less precision than `no_leniency`. |
//! | `all_included` | When calculating difficulty attributes in osu!standard, consider both stack leniency and sliders. Best precision but significantly worse performance than `no_leniency`. |
//!
//! ### Roadmap
//! - osu sr versions
//!   - [x] all included
//!   - [x] no_leniency
//!   - [x] no_sliders_no_leniency
//! - [x] taiko sr
//! - [x] ctb sr
//! - [x] mania sr
//! ---
//! - [x] osu pp
//! - [x] taiko pp
//! - [x] ctb pp
//! - [x] mania pp
//! ---
//! - [x] refactoring
//! - [ ] benchmarking

pub mod fruits;
pub mod mania;
pub mod osu;
pub mod parse;
pub mod taiko;

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
pub use parse::{
    Beatmap, BeatmapAttributes, DifficultyPoint, GameMode, HitObject, HitObjectKind, HitSound,
    PathType, Pos2, TimingPoint,
};

pub trait BeatmapExt {
    /// Calculate the stars and other attributes of a beatmap which are required for pp calculation.
    ///
    /// For osu!standard maps, the `no_leniency` version will be used.
    fn stars(&self, mods: impl Mods, passed_objects: Option<usize>) -> StarResult;

    /// Calculate the max pp of a beatmap if that is all you want.
    ///
    /// For osu!standard maps, the `no_leniency` version will be used.
    ///
    /// If you seek more fine-tuning and options you need to match on the map's
    /// mode and use the mode's corresponding calculator, e.g. [`TaikoPP`](crate::TaikoPP) for taiko.
    fn max_pp(&self, mods: u32) -> PpResult;

    /// Returns a builder to calculate a pp value.
    ///
    /// Although this method is not terribly bad, it is recommended to match on the
    /// map's mode yourself and construct the pp builder accordingly.
    fn pp(&self) -> AnyPP;

    /// Calculate the strains of a map.
    /// This essentially performs the same calculation as a `stars` function but
    /// instead of evaluating the final strains, they are just returned as is.
    ///
    /// Suitable to plot the difficulty of a map over time.
    ///
    /// For osu!standard maps, the `no_leniency` version will be used.
    fn strains(&self, mods: impl Mods) -> Strains;
}

impl BeatmapExt for Beatmap {
    fn stars(&self, mods: impl Mods, passed_objects: Option<usize>) -> StarResult {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                {
                    #[cfg(feature = "no_leniency")]
                    {
                        osu::no_leniency::stars(self, mods, passed_objects)
                    }

                    #[cfg(all(not(feature = "no_leniency"), feature = "no_sliders_no_leniency"))]
                    {
                        osu::no_sliders_no_leniency::stars(self, mods, passed_objects)
                    }

                    #[cfg(all(
                        not(feature = "no_leniency"),
                        not(feature = "no_sliders_no_leniency"),
                        feature = "all_included"
                    ))]
                    {
                        osu::all_included::stars(self, mods, passed_objects)
                    }

                    #[cfg(not(any(
                        feature = "no_leniency",
                        feature = "no_sliders_no_leniency",
                        feature = "all_included"
                    )))]
                    panic!("either of the features `no_leniency`, `no_sliders_no_leniency`, or `all_included` must be enabled");
                }
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                mania::stars(self, mods, passed_objects)
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "taiko")]
                taiko::stars(self, mods, passed_objects)
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                fruits::stars(self, mods, passed_objects)
            }
        }
    }

    fn max_pp(&self, mods: u32) -> PpResult {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                OsuPP::new(self).mods(mods).calculate()
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                ManiaPP::new(self).mods(mods).calculate()
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "taiko")]
                TaikoPP::new(self).mods(mods).calculate()
            }
            GameMode::CTB => {
                #[cfg(not(feature = "fruits"))]
                panic!("`fruits` feature is not enabled");

                #[cfg(feature = "fruits")]
                FruitsPP::new(self).mods(mods).calculate()
            }
        }
    }

    fn pp(&self) -> AnyPP {
        AnyPP::new(self)
    }

    fn strains(&self, mods: impl Mods) -> Strains {
        match self.mode {
            GameMode::STD => {
                #[cfg(not(feature = "osu"))]
                panic!("`osu` feature is not enabled");

                #[cfg(feature = "osu")]
                {
                    #[cfg(feature = "no_leniency")]
                    {
                        osu::no_leniency::strains(self, mods)
                    }

                    #[cfg(all(not(feature = "no_leniency"), feature = "no_sliders_no_leniency"))]
                    {
                        osu::no_sliders_no_leniency::strains(self, mods)
                    }

                    #[cfg(all(
                        not(feature = "no_leniency"),
                        not(feature = "no_sliders_no_leniency"),
                        feature = "all_included"
                    ))]
                    {
                        osu::all_included::strains(self, mods)
                    }

                    #[cfg(not(any(
                        feature = "no_leniency",
                        feature = "no_sliders_no_leniency",
                        feature = "all_included"
                    )))]
                    panic!("either of the features `no_leniency`, `no_sliders_no_leniency`, or `all_included` must be enabled");
                }
            }
            GameMode::MNA => {
                #[cfg(not(feature = "mania"))]
                panic!("`mania` feature is not enabled");

                #[cfg(feature = "mania")]
                mania::strains(self, mods)
            }
            GameMode::TKO => {
                #[cfg(not(feature = "taiko"))]
                panic!("`osu` feature is not enabled");

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
    pub section_length: f32,
    pub strains: Vec<f32>,
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
    pub fn stars(&self) -> f32 {
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
pub struct PpResult {
    pub pp: f32,
    pub attributes: StarResult,
}

impl PpResult {
    /// The final pp value.
    #[inline]
    pub fn pp(&self) -> f32 {
        self.pp
    }

    /// The final star value.
    #[inline]
    pub fn stars(&self) -> f32 {
        self.attributes.stars()
    }
}

#[cfg(any(feature = "osu", feature = "taiko"))]
#[inline]
fn difficulty_range(val: f32, max: f32, avg: f32, min: f32) -> f32 {
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
    not(any(
        feature = "all_included",
        feature = "no_leniency",
        feature = "no_sliders_no_leniency"
    ))
))]
compile_error!("Since the `osu` feature is enabled, either `no_leniency`, `no_sliders_no_leniency`, or `all_included` must be enabled aswell");

#[cfg(any(
    all(feature = "no_leniency", feature = "no_sliders_no_leniency"),
    all(feature = "no_leniency", feature = "all_included"),
    all(feature = "all_included", feature = "no_sliders_no_leniency"),
))]
compile_error!("Only one of the features `no_leniency`, `no_sliders_no_leniency`, `all_included` can be enabled");

#[cfg(all(
    not(feature = "osu"),
    any(
        feature = "no_leniency",
        feature = "no_sliders_no_leniency",
        feature = "all_included"
    )
))]
compile_error!("The features `no_leniency`, `no_sliders_no_leniency`, and `all_included` should only be enabled in combination with the `osu` feature");
