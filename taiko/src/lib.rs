mod difficulty_object;
mod hitobject_rhythm;
mod limited_queue;
mod rim;
mod skill;
mod skill_kind;
mod stamina_cheese;

use difficulty_object::DifficultyObject;
use hitobject_rhythm::{closest_rhythm, HitObjectRhythm};
use limited_queue::LimitedQueue;
use rim::Rim;
use skill::Skill;
use skill_kind::SkillKind;
use stamina_cheese::StaminaCheeseDetector;

use parse::{Beatmap, Mods};

use std::cmp::Ordering;
use std::f32::consts::PI;

const SECTION_LEN: f32 = 400.0;

const COLOR_SKILL_MULTIPLIER: f32 = 0.01;
const RHYTHM_SKILL_MULTIPLIER: f32 = 0.014;
const STAMINA_SKILL_MULTIPLIER: f32 = 0.02;

/// Star calculation for osu!taiko maps
pub fn stars(map: &Beatmap, mods: impl Mods) -> f32 {
    if map.hit_objects.len() < 2 {
        return 0.0;
    }

    // True if the object at that index is stamina cheese
    let cheese = map.find_cheese();

    let mut skills = vec![
        Skill::new(SkillKind::color()),
        Skill::new(SkillKind::rhythm()),
        Skill::new(SkillKind::stamina(true)),
        Skill::new(SkillKind::stamina(false)),
    ];

    let clock_rate = mods.speed();
    let section_len = SECTION_LEN * clock_rate;

    // No strain for first object
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let hit_objects = map
        .hit_objects
        .iter()
        .enumerate()
        .skip(2)
        .zip(map.hit_objects.iter().skip(1))
        .zip(map.hit_objects.iter())
        .map(|(((idx, base), prev), prev_prev)| {
            DifficultyObject::new(idx, base, prev, prev_prev, clock_rate)
        });

    for h in hit_objects {
        while h.base.start_time > current_section_end {
            for skill in skills.iter_mut() {
                skill.save_current_peak();
                skill.start_new_section_from(current_section_end);
            }

            current_section_end += section_len;
        }

        for skill in skills.iter_mut().take(3) {
            skill.process(h.clone(), &cheese);
        }

        skills[3].process(h, &cheese);
    }

    for skill in skills.iter_mut() {
        skill.save_current_peak();
    }

    let mut buf = vec![0.0; skills[0].strain_peaks.len()];

    let color_rating = skills[0].difficulty_value(&mut buf) * COLOR_SKILL_MULTIPLIER;
    let rhythm_rating = skills[1].difficulty_value(&mut buf) * RHYTHM_SKILL_MULTIPLIER;

    let mut stamina_rating = (skills[2].difficulty_value(&mut buf)
        + skills[3].difficulty_value(&mut buf))
        * STAMINA_SKILL_MULTIPLIER;

    let stamina_penalty = simple_color_penalty(stamina_rating, color_rating);
    stamina_rating *= stamina_penalty;

    let combined_rating = locally_combined_difficulty(&skills, stamina_penalty);
    let separate_rating = norm(1.5, color_rating, rhythm_rating, stamina_rating);

    rescale(1.4 * separate_rating + 0.5 * combined_rating)
}

#[inline]
fn rescale(stars: f32) -> f32 {
    if stars < 0.0 {
        stars
    } else {
        10.43 * (stars / 8.0 + 1.0).ln()
    }
}

#[inline]
fn simple_color_penalty(stamina: f32, color: f32) -> f32 {
    if color <= 0.0 {
        0.79 - 0.25
    } else {
        0.79 - (stamina / color - 12.0).atan() / PI / 2.0
    }
}

fn locally_combined_difficulty(skills: &[Skill], stamina_penalty: f32) -> f32 {
    let mut peaks = Vec::with_capacity(skills[0].strain_peaks.len());

    let iter = skills[0]
        .strain_peaks
        .iter()
        .zip(skills[1].strain_peaks.iter())
        .zip(skills[2].strain_peaks.iter())
        .zip(skills[3].strain_peaks.iter())
        .map(|(((&color, &rhythm), &stamina_right), &stamina_left)| {
            norm(
                2.0,
                color * COLOR_SKILL_MULTIPLIER,
                rhythm * RHYTHM_SKILL_MULTIPLIER,
                (stamina_right + stamina_left) * STAMINA_SKILL_MULTIPLIER * stamina_penalty,
            )
        });

    peaks.extend(iter);
    peaks.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

    let mut difficulty = 0.0;
    let mut weight = 1.0;

    for strain in peaks {
        difficulty += strain * weight;
        weight *= 0.9;
    }

    difficulty
}

#[inline]
fn norm(p: f32, a: f32, b: f32, c: f32) -> f32 {
    (a.powf(p) + b.powf(p) + c.powf(p)).powf(p.recip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_single() {
        let file = match File::open("E:/Games/osu!/beatmaps/1097541.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let stars = stars(&map, 16);

        println!("Stars: {}", stars);
    }

    #[test]
    fn test_taiko() {
        let margin = 0.005;

        #[rustfmt::skip]
        let data = vec![
            (110219, 1 << 8, 4.090461690284154),  // HT
            (110219, 0, 5.137432251440863),       // NM
            (110219, 1 << 6, 6.785308286298745),  // DT

            (168450, 1 << 8, 3.9102755155437663), // HT
            (168450, 0, 4.740171803038067),       // NM
            (168450, 1 << 6, 5.894260068145283),  // DT

            (1097541, 1 << 8, 4.0027499635116595),// HT
            (1097541, 0, 4.891409786886079),      // NM
            (1097541, 1 << 6, 6.587467490088248), // DT

            (1432878, 1 << 8, 3.5850143199594258),// HT
            (1432878, 0, 4.416206873466799),      // NM
            (1432878, 1 << 6, 5.908970879987477), // DT
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
