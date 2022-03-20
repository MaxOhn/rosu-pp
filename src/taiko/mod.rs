mod difficulty_object;
mod gradual_difficulty;
mod gradual_performance;
mod hitobject_rhythm;
mod limited_queue;
mod pp;
mod rim;
mod skill;
mod skill_kind;
mod stamina_cheese;
mod taiko_object;

use difficulty_object::DifficultyObject;
pub use gradual_difficulty::*;
pub use gradual_performance::*;
use hitobject_rhythm::{closest_rhythm, HitObjectRhythm};
use limited_queue::LimitedQueue;
pub use pp::*;
use rim::Rim;
use skill_kind::SkillKind;
use stamina_cheese::StaminaCheeseDetector;
use taiko_object::IntoTaikoObjectIter;

use crate::taiko::skill::Skills;
use crate::{Beatmap, Mods, Strains};

use std::cmp::Ordering;
use std::f64::consts::PI;

const SECTION_LEN: f64 = 400.0;

const COLOR_SKILL_MULTIPLIER: f64 = 0.01;
const RHYTHM_SKILL_MULTIPLIER: f64 = 0.014;
const STAMINA_SKILL_MULTIPLIER: f64 = 0.02;

/// Difficulty calculator on osu!taiko maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{TaikoStars, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let difficulty_attrs = TaikoStars::new(&map)
///     .mods(8 + 64) // HDDT
///     .calculate();
///
/// println!("Stars: {}", difficulty_attrs.stars);
/// ```
#[derive(Clone, Debug)]
pub struct TaikoStars<'map> {
    map: &'map Beatmap,
    mods: u32,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,
}

impl<'map> TaikoStars<'map> {
    /// Create a new difficulty calculator for osu!taiko maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map,
            mods: 0,
            passed_objects: None,
            clock_rate: None,
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the difficulty after every few objects, instead of
    /// using [`TaikoStars`] multiple times with different `passed_objects`, you should use
    /// [`TaikoGradualDifficultyAttributes`](crate::taiko::TaikoGradualDifficultyAttributes).
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    #[inline]
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Calculate all difficulty related values, including stars.
    #[inline]
    pub fn calculate(self) -> TaikoDifficultyAttributes {
        let (skills, max_combo) = calculate_skills(self);
        let mut buf = vec![0.0; skills.strain_peaks_len()];

        skills.color.copy_strain_peaks(&mut buf);
        let color_rating = skills.color.difficulty_value(&mut buf) * COLOR_SKILL_MULTIPLIER;

        skills.rhythm.copy_strain_peaks(&mut buf);
        let rhythm_rating = skills.rhythm.difficulty_value(&mut buf) * RHYTHM_SKILL_MULTIPLIER;

        skills.stamina_right.copy_strain_peaks(&mut buf);
        let stamina_right = skills.stamina_right.difficulty_value(&mut buf);

        skills.stamina_left.copy_strain_peaks(&mut buf);
        let stamina_left = skills.stamina_left.difficulty_value(&mut buf);

        let mut stamina_rating = (stamina_right + stamina_left) * STAMINA_SKILL_MULTIPLIER;

        let stamina_penalty = simple_color_penalty(stamina_rating, color_rating);
        stamina_rating *= stamina_penalty;

        let combined_rating = locally_combined_difficulty(&mut buf, &skills, stamina_penalty);
        let separate_rating = norm(1.5, color_rating, rhythm_rating, stamina_rating);

        let stars = rescale(1.4 * separate_rating + 0.5 * combined_rating);

        TaikoDifficultyAttributes { stars, max_combo }
    }

    /// Calculate the skill strains.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> Strains {
        let clock_rate = self.clock_rate.unwrap_or_else(|| self.mods.speed());
        let (skills, _) = calculate_skills(self);

        let strains = skills
            .color
            .strain_peaks
            .iter()
            .zip(skills.rhythm.strain_peaks.iter())
            .zip(skills.stamina_right.strain_peaks.iter())
            .zip(skills.stamina_left.strain_peaks.iter())
            .map(|(((color, rhythm), stamina_right), stamina_left)| {
                color + rhythm + stamina_right + stamina_left
            })
            .collect();

        Strains {
            section_length: SECTION_LEN * clock_rate,
            strains,
        }
    }
}

fn calculate_skills(params: TaikoStars<'_>) -> (Skills, usize) {
    let TaikoStars {
        map,
        mods,
        passed_objects,
        clock_rate,
    } = params;

    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());
    let clock_rate = clock_rate.unwrap_or_else(|| mods.speed());

    // True if the object at that index is stamina cheese
    let cheese = map.find_cheese();
    let mut skills = Skills::new();
    let mut max_combo = 0;

    match map.hit_objects.get(0) {
        Some(h) => max_combo += h.is_circle() as usize,
        None => return (skills, max_combo),
    }

    match map.hit_objects.get(1) {
        Some(h) => max_combo += h.is_circle() as usize,
        None => return (skills, max_combo),
    }

    let mut hit_objects = map
        .taiko_objects()
        .take(take)
        .enumerate()
        .skip(2)
        .zip(map.taiko_objects().skip(1))
        .zip(map.taiko_objects())
        .inspect(|(((_, base), _), _)| max_combo += base.h.is_circle() as usize)
        .map(|(((idx, base), prev), prev_prev)| {
            DifficultyObject::new(idx, base, prev, prev_prev, clock_rate)
        });

    // Handle first element distinctly
    let h = match hit_objects.next() {
        Some(h) => h,
        None => return (skills, max_combo),
    };

    // No strain for first object
    let mut curr_section_end = (h.start_time / SECTION_LEN).ceil() * SECTION_LEN;
    skills.process(&h, &cheese);

    // Handle all other objects
    for h in hit_objects {
        while h.start_time > curr_section_end {
            skills.save_peak_and_start_new_section(curr_section_end);
            curr_section_end += SECTION_LEN;
        }

        skills.process(&h, &cheese);
    }

    skills.save_current_peak();

    (skills, max_combo)
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

fn locally_combined_difficulty(peaks: &mut Vec<f64>, skills: &Skills, stamina_penalty: f64) -> f64 {
    peaks.clear();

    let iter = skills
        .color
        .strain_peaks
        .iter()
        .zip(skills.rhythm.strain_peaks.iter())
        .zip(skills.stamina_right.strain_peaks.iter())
        .zip(skills.stamina_left.strain_peaks.iter())
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
        difficulty += *strain * weight;
        weight *= 0.9;
    }

    difficulty
}

#[inline]
fn norm(p: f64, a: f64, b: f64, c: f64) -> f64 {
    (a.powf(p) + b.powf(p) + c.powf(p)).powf(p.recip())
}

/// The result of a difficulty calculation on an osu!taiko map.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TaikoDifficultyAttributes {
    /// The final star rating.
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: usize,
}

impl TaikoDifficultyAttributes {
    /// Return the maximum combo.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.max_combo
    }
}

/// The result of a performance calculation on an osu!taiko map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaikoPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: TaikoDifficultyAttributes,
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
        self.difficulty.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.difficulty.max_combo
    }
}

impl From<TaikoPerformanceAttributes> for TaikoDifficultyAttributes {
    fn from(attributes: TaikoPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
