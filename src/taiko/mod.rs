#![cfg(feature = "taiko")]

mod difficulty_object;
mod hitobject_rhythm;
mod limited_queue;
mod pp;
mod rim;
mod skill;
mod skill_kind;
mod stamina_cheese;

use difficulty_object::DifficultyObject;
use hitobject_rhythm::{closest_rhythm, HitObjectRhythm};
use limited_queue::LimitedQueue;
pub use pp::*;
use rim::Rim;
use skill::Skill;
use skill_kind::SkillKind;
use stamina_cheese::StaminaCheeseDetector;

use crate::{Beatmap, Mods, Strains};

use std::cmp::Ordering;
use std::f64::consts::PI;

const SECTION_LEN: f64 = 400.0;

const COLOR_SKILL_MULTIPLIER: f64 = 0.01;
const RHYTHM_SKILL_MULTIPLIER: f64 = 0.014;
const STAMINA_SKILL_MULTIPLIER: f64 = 0.02;

/// Difficulty calculation for osu!taiko maps.
///
/// In case of a partial play, e.g. a fail, one can specify the amount of passed objects.
pub fn stars(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> TaikoDifficultyAttributes {
    let max_combo = map.n_circles as usize;

    let skills = match calculate_skills(map, mods, passed_objects) {
        Some(skills) => skills,
        None => {
            return TaikoDifficultyAttributes {
                max_combo,
                ..Default::default()
            }
        }
    };

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

    let stars = rescale(1.4 * separate_rating + 0.5 * combined_rating);

    TaikoDifficultyAttributes { stars, max_combo }
}

/// Essentially the same as the [`stars`] function but instead of
/// evaluating the final strains, it just returns them as is.
///
/// Suitable to plot the difficulty of a map over time.
pub fn strains(map: &Beatmap, mods: impl Mods) -> Strains {
    let skills = match calculate_skills(map, mods, None) {
        Some(skills) => skills,
        None => return Strains::default(),
    };

    let strains = skills[0]
        .strain_peaks
        .iter()
        .zip(skills[1].strain_peaks.iter())
        .zip(skills[2].strain_peaks.iter())
        .zip(skills[3].strain_peaks.iter())
        .map(|(((color, rhythm), stamina_right), stamina_left)| {
            color + rhythm + stamina_right + stamina_left
        })
        .collect();

    Strains {
        section_length: SECTION_LEN * mods.speed(),
        strains,
    }
}

fn calculate_skills(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> Option<Vec<Skill>> {
    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());

    if take < 2 {
        return None;
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

    let mut hit_objects = map
        .hit_objects
        .iter()
        .take(take)
        .enumerate()
        .skip(2)
        .zip(map.hit_objects.iter().skip(1))
        .zip(map.hit_objects.iter())
        .map(|(((idx, base), prev), prev_prev)| {
            DifficultyObject::new(idx, base, prev, prev_prev, clock_rate)
        });

    // Handle second object separately to remove later if-branching
    let h = hit_objects.next().unwrap();

    while h.base.start_time > current_section_end {
        current_section_end += section_len;
    }

    for skill in skills.iter_mut() {
        skill.process(&h, &cheese);
    }

    // Handle all other objects
    for h in hit_objects {
        while h.base.start_time > current_section_end {
            for skill in skills.iter_mut() {
                skill.save_current_peak();
                skill.start_new_section_from(current_section_end / clock_rate);
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

    Some(skills)
}

#[inline]
fn rescale(stars: f64) -> f64 {
    if stars < 0.0 {
        stars
    } else {
        10.43 * (stars / 8.0).ln_1p()
    }
}

#[inline]
fn simple_color_penalty(stamina: f64, color: f64) -> f64 {
    if color <= 0.0 {
        0.79 - 0.25
    } else {
        0.79 - (stamina / color - 12.0).atan() / PI / 2.0
    }
}

fn locally_combined_difficulty(skills: &[Skill], stamina_penalty: f64) -> f64 {
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
fn norm(p: f64, a: f64, b: f64, c: f64) -> f64 {
    (a.powf(p) + b.powf(p) + c.powf(p)).powf(p.recip())
}

/// The result of a difficulty calculation on an osu!taiko map.
#[derive(Clone, Debug, Default)]
pub struct TaikoDifficultyAttributes {
    /// The final star rating.
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: usize,
}

/// The result of a performance calculation on an osu!taiko map.
#[derive(Clone, Debug, Default)]
pub struct TaikoPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub attributes: TaikoDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
    /// The accuracy portion of the final pp.
    pub pp_acc: f64,
    /// The strain portion of the final pp.
    pub pp_strain: f64,
}

impl TaikoPerformanceAttributes {
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

impl From<TaikoPerformanceAttributes> for TaikoDifficultyAttributes {
    fn from(attributes: TaikoPerformanceAttributes) -> Self {
        attributes.attributes
    }
}
