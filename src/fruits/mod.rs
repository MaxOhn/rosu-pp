mod catch_object;
mod difficulty_object;
mod movement;
mod pp;

use catch_object::CatchObject;
use difficulty_object::DifficultyObject;
use movement::Movement;
pub use pp::*;

use crate::{curve::Curve, Beatmap, HitObjectKind, Mods, PathType};

use std::cmp::Ordering;
use std::convert::identity;

const SECTION_LENGTH: f32 = 750.0;
const STAR_SCALING_FACTOR: f32 = 0.153;

const ALLOWED_CATCH_RANGE: f32 = 0.8;
const CATCHER_SIZE: f32 = 106.75;

macro_rules! binary_search {
    ($slice:expr, $target:expr) => {
        $slice.binary_search_by(|p| p.time.partial_cmp(&$target).unwrap_or(Ordering::Equal))
    };
}

/// Star calculation for osu!ctb maps
// Slider parsing based on https://github.com/osufx/catch-the-pp
pub fn stars(map: &Beatmap, mods: impl Mods) -> DifficultyAttributes {
    if map.hit_objects.len() < 2 {
        return DifficultyAttributes::default();
    }

    let attributes = map.attributes().mods(mods);
    let with_hr = mods.hr();
    let mut ticks = Vec::new(); // using the same buffer for all sliders

    let mut fruits = 0;
    let mut droplets = 0;

    // BUG: Incorrect object order on 2B maps that have fruits within sliders
    let mut hit_objects = map
        .hit_objects
        .iter()
        .scan((None, 0.0), |(last_pos, last_time), h| match &h.kind {
            HitObjectKind::Circle => {
                let mut h = CatchObject::new((h.pos, h.start_time));

                if with_hr {
                    h = h.with_hr(last_pos, last_time);
                }

                fruits += 1;

                Some(Some(FruitOrJuice::Fruit(Some(h))))
            }
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                curve_points,
                path_type,
            } => {
                // HR business
                last_pos
                    .replace(h.pos.x + curve_points[curve_points.len() - 1].x - curve_points[0].x);
                *last_time = h.start_time;

                let (beat_len, timing_time) = {
                    match binary_search!(map.timing_points, h.start_time) {
                        Ok(idx) => {
                            let point = &map.timing_points[idx];
                            (point.beat_len, point.time)
                        }
                        Err(0) => (1000.0, 0.0),
                        Err(idx) => {
                            let point = &map.timing_points[idx - 1];
                            (point.beat_len, point.time)
                        }
                    }
                };

                let (speed_multiplier, diff_time) = {
                    match binary_search!(map.difficulty_points, h.start_time) {
                        Ok(idx) => {
                            let point = &map.difficulty_points[idx];
                            (point.speed_multiplier, point.time)
                        }
                        Err(0) => (1.0, 0.0),
                        Err(idx) => {
                            let point = &map.difficulty_points[idx - 1];
                            (point.speed_multiplier, point.time)
                        }
                    }
                };

                let mut tick_distance = 100.0 * map.sv / map.tick_rate;

                if map.version >= 8 {
                    tick_distance /= (100.0 / speed_multiplier).max(10.0).min(1000.0) / 100.0;
                }

                let spm = if timing_time > diff_time {
                    1.0
                } else {
                    speed_multiplier
                };

                let duration = *repeats as f32 * beat_len * *pixel_len / (map.sv * spm) / 100.0;

                let path_type = if *path_type == PathType::PerfectCurve && curve_points.len() > 3 {
                    PathType::Bezier
                } else if curve_points.len() == 2 {
                    PathType::Linear
                } else {
                    *path_type
                };

                let curve = match path_type {
                    PathType::Linear => Curve::linear(curve_points[0], curve_points[1]),
                    PathType::Bezier => Curve::bezier(curve_points),
                    PathType::Catmull => Curve::catmull(curve_points),
                    PathType::PerfectCurve => Curve::perfect(curve_points),
                };

                let mut current_distance = tick_distance;
                let time_add = duration * (tick_distance / (*pixel_len * *repeats as f32));

                let target = *pixel_len - tick_distance / 8.0;
                ticks.reserve((target / tick_distance) as usize);

                while current_distance < target {
                    let pos = curve.point_at_distance(current_distance);

                    ticks.push((pos, h.start_time + time_add * (ticks.len() + 1) as f32));
                    current_distance += tick_distance;
                }

                let mut slider_objects = Vec::with_capacity(repeats * (ticks.len() + 1));
                slider_objects.push((h.pos, h.start_time));

                if *repeats <= 1 {
                    slider_objects.append(&mut ticks); // automatically empties buffer for next slider
                } else {
                    slider_objects.append(&mut ticks.clone());

                    for repeat_id in 1..*repeats - 1 {
                        let dist = (repeat_id % 2) as f32 * *pixel_len;
                        let time_offset = (duration / *repeats as f32) * repeat_id as f32;
                        let pos = curve.point_at_distance(dist);

                        // Reverse tick / last legacy tick
                        slider_objects.push((pos, h.start_time + time_offset));

                        ticks.reverse();
                        slider_objects.extend_from_slice(&ticks); // tick time doesn't need to be adjusted for some reason
                    }

                    // Handling last span separatly so that `ticks` vector isn't cloned again
                    let dist = ((*repeats - 1) % 2) as f32 * *pixel_len;
                    let time_offset = (duration / *repeats as f32) * (*repeats - 1) as f32;
                    let pos = curve.point_at_distance(dist);

                    slider_objects.push((pos, h.start_time + time_offset));

                    ticks.reverse();
                    slider_objects.append(&mut ticks); // automatically empties buffer for next slider
                }

                // Slider tail
                let dist_end = (*repeats % 2) as f32 * *pixel_len;
                let pos = curve.point_at_distance(dist_end);
                slider_objects.push((pos, h.start_time + duration));

                fruits += 1 + *repeats;
                droplets += slider_objects.len() - 1 - *repeats;

                let iter = slider_objects.into_iter().map(CatchObject::new);

                Some(Some(FruitOrJuice::Juice(iter)))
            }
            HitObjectKind::Spinner { .. } | HitObjectKind::Hold { .. } => Some(None),
        })
        .filter_map(identity)
        .flatten();

    // Hyper dash business
    let half_catcher_width = calculate_catch_width(attributes.cs) / 2.0 / ALLOWED_CATCH_RANGE;
    let mut last_direction = 0;
    let mut last_excess = half_catcher_width;

    // Strain business
    let mut movement = Movement::new(attributes.cs);
    let section_len = SECTION_LENGTH * attributes.clock_rate;
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev = hit_objects.next().unwrap();
    let mut curr = hit_objects.next().unwrap();

    prev.init_hyper_dash(
        half_catcher_width,
        &curr,
        &mut last_direction,
        &mut last_excess,
    );

    for next in hit_objects {
        curr.init_hyper_dash(
            half_catcher_width,
            &next,
            &mut last_direction,
            &mut last_excess,
        );

        let h = DifficultyObject::new(
            &curr,
            &prev,
            movement.half_catcher_width,
            attributes.clock_rate,
        );

        while h.base.time > current_section_end {
            movement.save_current_peak();
            movement.start_new_section_from(current_section_end);
            current_section_end += section_len;
        }

        movement.process(&h);

        prev = curr;
        curr = next;
    }

    // Same as in loop but without init_hyper_dash because `curr` is the last element
    let h = DifficultyObject::new(
        &curr,
        &prev,
        movement.half_catcher_width,
        attributes.clock_rate,
    );

    while h.base.time > current_section_end {
        movement.save_current_peak();
        movement.start_new_section_from(current_section_end);

        current_section_end += section_len;
    }

    movement.process(&h);
    movement.save_current_peak();

    let stars = movement.difficulty_value().sqrt() * STAR_SCALING_FACTOR;

    DifficultyAttributes {
        stars,
        n_fruits: fruits,
        n_droplets: droplets,
        max_combo: fruits + droplets,
    }
}

#[inline]
pub(crate) fn calculate_catch_width(cs: f32) -> f32 {
    let scale = 1.0 - 0.7 * (cs - 5.0) / 5.0;

    CATCHER_SIZE * scale.abs() * ALLOWED_CATCH_RANGE
}

enum FruitOrJuice<I> {
    Fruit(Option<CatchObject>),
    Juice(I),
}

impl<I: Iterator<Item = CatchObject>> Iterator for FruitOrJuice<I> {
    type Item = CatchObject;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Fruit(fruit) => fruit.take(),
            Self::Juice(slider) => slider.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Fruit(Some(_)) => (1, Some(1)),
            Self::Fruit(None) => (0, Some(0)),
            Self::Juice(slider) => slider.size_hint(),
        }
    }
}

#[derive(Default)]
pub struct DifficultyAttributes {
    pub stars: f32,
    pub max_combo: usize,
    pub n_fruits: usize,
    pub n_droplets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_single() {
        let map_id = 1972149;
        let file = match File::open(format!("E:/Games/osu!/beatmaps/{}.osu", map_id)) {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };
        // let file = match File::open(format!("E:/Games/osu!/beatmaps/{}.osu", map_id)) {
        //     Ok(file) => file,
        //     Err(why) => panic!("Could not open file: {}", why),
        // };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let mods = 0;
        let stars = stars(&map, mods).stars;

        println!("Stars: {} [map={} | mods={}]", stars, map_id, mods);
    }

    #[test]
    fn test_fruits() {
        let margin = 0.005;

        #[rustfmt::skip]
        let data = vec![
            (1977380, 1 << 8, 2.0564713386286573),// HT
            (1977380, 0, 2.5695489769068742),     // NM
            (1977380, 1 << 6, 3.589887228221038), // DT
            (1977380, 1 << 4, 3.1515873669521928),// HR
            (1977380, 1 << 1, 3.0035260129778396),// EZ

            (1974968, 1 << 8, 1.9544305373156605),// HT
            (1974968, 0, 2.521701539665241),      // NM
            (1974968, 1 << 6, 3.650649037957456), // DT
            (1974968, 1 << 4, 3.566302788963401), // HR
            (1974968, 1 << 1, 2.2029392066882654),// EZ

            (2420076, 1 << 8, 4.791039358886245), // HT
            (2420076, 0, 6.223136555625056),      // NM
            (2420076, 1 << 6, 8.908315960310958), // DT
            (2420076, 1 << 4, 6.54788067620051),  // HR
            (2420076, 1 << 1, 6.067971540209479), // EZ

            (2206596, 1 << 8, 4.767182611189798), // HT
            (2206596, 0, 6.157660207091584),      // NM
            (2206596, 1 << 6, 8.93391286552717),  // DT
            (2206596, 1 << 4, 6.8639096665110735),// HR
            (2206596, 1 << 1, 5.60279198088948),  // EZ

            // Super long juice stream towards end
            // (1972149, 1 << 8, 4.671425766413811), // HT
            // (1972149, 0, 6.043742871084152),      // NM
            // (1972149, 1 << 6, 8.469259368304225), // DT
            // (1972149, 1 << 4, 6.81222485322862),  // HR
            // (1972149, 1 << 1, 5.289343020686747), // EZ

            // Convert slider fiesta
            // (1657535, 1 << 8, 3.862453635711741), // HT
            // (1657535, 0, 4.792543335869686),      // NM
            // (1657535, 1 << 6, 6.655478646330863), // DT
            // (1657535, 1 << 4, 5.259728567781568), // HR
            // (1657535, 1 << 1, 4.127535166776765), // EZ
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

            let stars = stars(&map, mods).stars;

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
