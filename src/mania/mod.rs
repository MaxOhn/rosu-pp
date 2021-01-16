mod pp;
mod strain;

pub use pp::*;
use strain::Strain;

use crate::{Beatmap, HitObject, Mods};

const SECTION_LEN: f32 = 400.0;
const STAR_SCALING_FACTOR: f32 = 0.018;

/// Star calculation for osu!mania maps
pub fn stars(map: &Beatmap, mods: impl Mods) -> f32 {
    if map.hit_objects.len() < 2 {
        return 0.0;
    }

    let clock_rate = mods.speed();
    let section_len = SECTION_LEN * clock_rate;
    let mut strain = Strain::new(map.cs as u8);

    let mut hit_objects = map
        .hit_objects
        .iter()
        .skip(1)
        .zip(map.hit_objects.iter())
        .map(|(base, prev)| DifficultyHitObject::new(base, prev, map.cs, clock_rate));

    // No strain for first object
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    // Handle second object separately to remove later if-branching
    let h = hit_objects.next().unwrap();

    while h.base.start_time > current_section_end {
        current_section_end += section_len;
    }

    strain.process(&h);

    // Handle all other objects
    for h in hit_objects {
        while h.base.start_time > current_section_end {
            strain.save_current_peak();
            strain.start_new_section_from(current_section_end);

            current_section_end += section_len;
        }

        strain.process(&h);
    }

    strain.save_current_peak();

    strain.difficulty_value() * STAR_SCALING_FACTOR
}

#[derive(Debug)]
pub(crate) struct DifficultyHitObject<'o> {
    base: &'o HitObject,
    column: usize,
    delta: f32,
}

impl<'o> DifficultyHitObject<'o> {
    fn new(base: &'o HitObject, prev: &'o HitObject, cs: f32, clock_rate: f32) -> Self {
        let x_divisor = 512.0 / cs;
        let column = (base.pos.x / x_divisor).floor() as usize;

        Self {
            base,
            column,
            delta: (base.start_time - prev.start_time) / clock_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    #[ignore]
    fn mania_single() {
        let file = match File::open("E:/Games/osu!/beatmaps/1355822.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let result = PpCalculator::new(&map).mods(0).calculate();

        println!("Stars: {}", result.stars);
        println!("PP: {}", result.pp);
    }
}
