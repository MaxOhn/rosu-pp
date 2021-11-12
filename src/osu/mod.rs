#![cfg(feature = "osu")]

mod pp;

pub use pp::*;

#[cfg(feature = "osu_precise")]
#[cfg_attr(docsrs, doc(cfg(feature = "osu_precise")))]
mod precise;

#[cfg(feature = "osu_precise")]
pub use precise::*;

#[cfg(feature = "osu_fast")]
#[cfg_attr(docsrs, doc(cfg(feature = "osu_fast")))]
mod fast;

#[cfg(feature = "osu_fast")]
pub use fast::*;

/// Various data created through the star calculation.
/// This data is necessary to calculate PP.
#[derive(Clone, Debug, Default)]
pub struct DifficultyAttributes {
    pub aim_strain: f64,
    pub speed_strain: f64,
    pub flashlight_rating: f64,
    pub ar: f64,
    pub od: f64,
    pub hp: f64,
    pub n_circles: usize,
    pub n_sliders: usize,
    pub n_spinners: usize,
    pub stars: f64,
    pub max_combo: usize,
}

/// Various data created through the pp calculation.
#[derive(Clone, Debug, Default)]
pub struct PerformanceAttributes {
    pub attributes: DifficultyAttributes,
    pub pp_acc: f64,
    pub pp_aim: f64,
    pub pp_flashlight: f64,
    pub pp_speed: f64,
    pub pp: f64,
}

impl PerformanceAttributes {
    /// Return the star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        self.attributes.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f64 {
        self.pp
    }
}

#[inline]
fn difficulty_range_od(od: f64) -> f64 {
    super::difficulty_range(od, 20.0, 50.0, 80.0)
}

#[test]
// #[ignore]
fn custom_osu() {
    use std::{fs::File, time::Instant};

    use crate::{Beatmap, OsuPP};

    let path = "E:Games/osu!/beatmaps/70090_.osu";
    let file = File::open(path).unwrap();

    let start = Instant::now();
    let map = Beatmap::parse(file).unwrap();

    let iters = 100;
    let accum = start.elapsed();

    // * Tiny benchmark for map parsing
    // let mut accum = accum;

    // for _ in 0..iters {
    //     let file = File::open(path).unwrap();
    //     let start = Instant::now();
    //     let _map = Beatmap::parse(file).unwrap();
    //     accum += start.elapsed();
    // }

    println!("Parsing average: {:?}", accum / iters);

    let start = Instant::now();
    let result = OsuPP::new(&map).mods(16 + 64).calculate();

    let iters = 100;
    let accum = start.elapsed();

    // * Tiny benchmark for pp calculation
    // let mut accum = accum;

    // for _ in 0..iters {
    //     let start = Instant::now();
    //     let _result = OsuPP::new(&map).mods(0).calculate();
    //     accum += start.elapsed();
    // }

    println!("{:#?}", result);
    println!("Calculation average: {:?}", accum / iters);
}
