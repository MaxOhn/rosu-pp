#![cfg(feature = "osu")]

mod pp;
mod versions;

pub use pp::*;
pub use versions::*;

/// Various data created through the star calculation.
/// This data is necessary to calculate PP.
#[derive(Clone, Debug, Default)]
pub struct DifficultyAttributes {
    pub aim_strain: f32,
    pub speed_strain: f32,
    pub flashlight_rating: f32,
    pub ar: f32,
    pub od: f32,
    pub hp: f32,
    pub n_circles: usize,
    pub n_sliders: usize,
    pub n_spinners: usize,
    pub stars: f32,
    pub max_combo: usize,
}

/// Various data created through the pp calculation.
#[derive(Clone, Debug, Default)]
pub struct PerformanceAttributes {
    pub attributes: DifficultyAttributes,
    pub pp_acc: f32,
    pub pp_aim: f32,
    pub pp_flashlight: f32,
    pub pp_speed: f32,
    pub pp: f32,
}

impl PerformanceAttributes {
    /// Return the star value.
    #[inline]
    pub fn stars(&self) -> f32 {
        self.attributes.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f32 {
        self.pp
    }
}

#[test]
fn custom_osu() {
    use std::{fs::File, time::Instant};

    use crate::{Beatmap, OsuPP};

    let path = "E:Games/osu!/beatmaps/1402167_.osu";
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
    let result = OsuPP::new(&map).mods(1024).calculate();

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
