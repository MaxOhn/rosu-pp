mod strain;

use strain::Strain;

use crate::{Beatmap, HitObject, Mods};

const SECTION_LEN: f32 = 400.0;
const STAR_SCALING_FACTOR: f32 = 0.018;

/// Star calculation for osu!mania maps
pub fn stars(map: &Beatmap, mods: impl Mods) -> f32 {
    if map.hit_objects.is_empty() {
        return 0.0;
    }

    let clock_rate = mods.speed();
    let section_len = SECTION_LEN * clock_rate;
    let mut strain = Strain::new(map.cs as u8);

    let hit_objects = map
        .hit_objects
        .iter()
        .skip(1)
        .zip(map.hit_objects.iter())
        .map(|(base, prev)| DifficultyHitObject::new(base, prev, map.cs, clock_rate));

    // No strain for first object
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

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

pub struct PpResult {
    pub pp: f32,
    pub stars: f32,
}

pub trait PpProvider {
    fn pp(&self) -> PpCalculator;
}

impl PpProvider for Beatmap {
    #[inline]
    fn pp(&self) -> PpCalculator {
        PpCalculator::new(self)
    }
}

// TODO: Allow partial plays
pub struct PpCalculator<'m> {
    map: &'m Beatmap,
    stars: Option<f32>,
    mods: u32,
    score: f32,
}

impl<'m> PpCalculator<'m> {
    #[inline]
    pub fn new(map: &'m Beatmap) -> Self {
        Self {
            map,
            stars: None,
            mods: 0,
            score: 1_000_000.0,
        }
    }

    #[inline]
    pub fn stars(mut self, stars: f32) -> Self {
        self.stars.replace(stars);

        self
    }

    #[inline]
    pub fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    #[inline]
    pub fn score(mut self, score: u32) -> Self {
        self.score = score as f32;

        self
    }

    pub fn calculate(self) -> PpResult {
        let stars = self.stars.unwrap_or_else(|| stars(self.map, self.mods));

        let ez = self.mods.ez();
        let nf = self.mods.nf();
        let ht = self.mods.ht();

        let score_multiplier = 0.5_f32.powi(ez as i32 + nf as i32 + ht as i32);
        let scaled_score = self.score / score_multiplier;

        let mut multiplier = 0.8;

        if nf {
            multiplier *= 0.9;
        }

        if ez {
            multiplier *= 0.5;
        }

        let hit_window = {
            let od = (10.0 - self.map.od).max(0.0).min(10.0);
            let mut val = 34.0 + 3.0 * od;

            if ez {
                val *= 1.4;
            } else if self.mods.hr() {
                val /= 1.4;
            }

            val * self.mods.speed()
        };

        let strain_value = self.compute_strain(scaled_score, stars);
        let acc_value = self.compute_accuracy_value(scaled_score, strain_value, hit_window);

        let pp = (strain_value.powf(1.1) + acc_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

        PpResult { pp, stars }
    }

    fn compute_strain(&self, score: f32, stars: f32) -> f32 {
        let mut strain_value = (5.0 * (stars / 0.2).max(1.0) - 4.0).powf(2.2) / 135.0;

        strain_value *= 1.0 + 0.1 * (self.map.hit_objects.len() as f32 / 1500.0).min(1.0);

        if score <= 500_000.0 {
            strain_value = 0.0;
        } else if score <= 600_000.0 {
            strain_value *= (score - 500_000.0) / 100_000.0 * 0.3;
        } else if score <= 700_000.0 {
            strain_value *= 0.3 + (score - 600_000.0) / 100_000.0 * 0.25;
        } else if score <= 800_000.0 {
            strain_value *= 0.55 + (score - 700_000.0) / 100_000.0 * 0.2;
        } else if score <= 900_000.0 {
            strain_value *= 0.75 + (score - 800_000.0) / 100_000.0 * 0.15;
        } else {
            strain_value *= 0.9 + (score - 900_000.0) / 100_000.0 * 0.1;
        }

        strain_value
    }

    #[inline]
    fn compute_accuracy_value(&self, score: f32, strain: f32, hit_window: f32) -> f32 {
        (0.2 - (hit_window - 34.0) * 0.006667).max(0.0)
            * strain
            * ((score - 960_000.0).max(0.0) / 40_000.0).powf(1.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_single() {
        let file = match File::open("E:/Games/osu!/beatmaps/1355822.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let stars = stars(&map, 256);

        println!("Stars: {}", stars);
    }

    #[test]
    fn test_mania() {
        let margin = 0.005;

        #[rustfmt::skip]
        let data = vec![
            (1355822, 1 << 8, 2.2710870990702627), // HT
            (1355822, 0, 2.7966565927524574),      // NM
            (1355822, 1 << 6, 3.748525363730352),  // DT

            (1974394, 1 << 8, 3.8736942117487256), // HT
            (1974394, 0, 4.801793001581714),       // NM
            (1974394, 1 << 6, 6.517894438878535),  // DT

            (992512, 1 << 8, 5.29507262961579),    // HT
            (992512, 0, 6.536292432114728),        // NM
            (992512, 1 << 6, 8.944195050951032),   // DT
        ];

        for (map_id, mods, expected_stars) in data {
            let file = match File::open(format!("./test/{}.osu", map_id)) {
                Ok(file) => file,
                Err(why) => panic!("Could not open file {}.osu: {}", map_id, why),
            };

            let map = match Beatmap::parse(file) {
                Ok(map) => map,
                Err(why) => panic!("Error while parsing map {}: {}", map_id, why),
            };

            let stars = stars(&map, mods);

            assert!(
                (stars - expected_stars).abs() < margin,
                "Stars: {} | Expected: {} => {} margin [map {} | mods {}]",
                stars,
                expected_stars,
                (stars - expected_stars).abs(),
                map_id,
                mods
            );
        }
    }
}
