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

        for skill in skills.iter_mut() {
            skill.process(&h, &cheese);
        }
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
    max_combo: usize,
    combo: Option<usize>,
    acc: f32,
    n_misses: usize,
}

impl<'m> PpCalculator<'m> {
    #[inline]
    pub fn new(map: &'m Beatmap) -> Self {
        let max_combo = map.hit_objects.iter().filter(|h| h.is_circle()).count();

        Self {
            map,
            stars: None,
            mods: 0,
            max_combo,
            combo: None,
            acc: 100.0,
            n_misses: 0,
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
    pub fn combo(mut self, combo: usize) -> Self {
        self.combo.replace(combo);

        self
    }

    #[inline]
    pub fn misses(mut self, n_misses: usize) -> Self {
        self.n_misses = n_misses;

        self
    }

    /// Set the accuracy between 0.0 and 100.0;
    #[inline]
    pub fn accuracy(mut self, acc: f32) -> Self {
        self.acc = acc / 100.0;

        self
    }

    pub fn calculate(self) -> PpResult {
        let stars = self.stars.unwrap_or_else(|| stars(self.map, self.mods));

        let mut multiplier = 1.1;

        if self.mods.nf() {
            multiplier *= 0.9;
        }

        if self.mods.hd() {
            multiplier *= 1.1;
        }

        // TODO: Consider HR & co?
        let hit_window = difficulty_range(self.map.od) as i32 as f32 / self.mods.speed();

        let strain_value = self.compute_strain_value(stars);
        let acc_value = self.compute_accuracy_value(hit_window);

        let pp = (strain_value.powf(1.1) + acc_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

        PpResult { stars, pp }
    }

    fn compute_strain_value(&self, stars: f32) -> f32 {
        let exp_base = 5.0 * (stars / 0.0075).max(1.0) - 4.0;
        let mut strain = exp_base * exp_base / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 1.0 + 0.1 * (self.max_combo as f32 / 1500.0).min(1.0);
        strain *= len_bonus;

        // Penalize misses exponentially
        strain *= 0.985_f32.powi(self.n_misses as i32);

        // HD bonus
        if self.mods.hd() {
            strain *= 1.025;
        }

        // FL bonus
        if self.mods.fl() {
            strain *= 1.05 * len_bonus;
        }

        // Scale with accuracy
        strain * self.acc
    }

    #[inline]
    fn compute_accuracy_value(&self, hit_window: f32) -> f32 {
        (150.0 / hit_window).powf(1.1)
            * self.acc.powi(15)
            * 22.0
            * (self.max_combo as f32 / 1500.0).powf(0.3).min(1.15)
    }
}

const HITWINDOW_MIN: f32 = 50.0;
const HITWINDOW_AVG: f32 = 35.0;
const HITWINDOW_MAX: f32 = 20.0;

#[inline]
fn difficulty_range(od: f32) -> f32 {
    if od > 5.0 {
        HITWINDOW_AVG + (HITWINDOW_MAX - HITWINDOW_AVG) * (od - 5.0) / 5.0
    } else if od < 5.0 {
        HITWINDOW_AVG - (HITWINDOW_AVG - HITWINDOW_MIN) * (5.0 - od) / 5.0
    } else {
        HITWINDOW_AVG
    }
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
