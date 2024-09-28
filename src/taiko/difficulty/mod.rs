use std::cmp;

use crate::{
    any::difficulty::skills::Skill,
    taiko::{
        difficulty::{
            color::preprocessor::ColorDifficultyPreprocessor,
            object::{TaikoDifficultyObject, TaikoDifficultyObjects},
        },
        object::TaikoObject,
    },
    Difficulty,
};

use self::skills::{color::Color, rhythm::Rhythm, stamina::Stamina, TaikoSkills};

use super::{attributes::TaikoDifficultyAttributes, convert::TaikoBeatmap};

mod color;
pub mod gradual;
mod object;
mod rhythm;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.084_375;
const RHYTHM_SKILL_MULTIPLIER: f64 = 0.2 * DIFFICULTY_MULTIPLIER;
const COLOR_SKILL_MULTIPLIER: f64 = 0.375 * DIFFICULTY_MULTIPLIER;
const STAMINA_SKILL_MULTIPLIER: f64 = 0.375 * DIFFICULTY_MULTIPLIER;

pub fn difficulty(
    difficulty: &Difficulty,
    converted: &TaikoBeatmap<'_>,
) -> TaikoDifficultyAttributes {
    let hit_window = converted
        .attributes()
        .difficulty(difficulty)
        .hit_windows()
        .od;

    let DifficultyValues {
        skills: TaikoSkills {
            rhythm,
            color,
            stamina,
        },
        max_combo,
    } = DifficultyValues::calculate(difficulty, converted);

    let mut attrs = TaikoDifficultyAttributes {
        hit_window,
        max_combo,
        is_convert: converted.is_convert,
        ..Default::default()
    };

    let color_rating = color.as_difficulty_value();
    let rhythm_rating = rhythm.as_difficulty_value();
    let stamina_rating = stamina.as_difficulty_value();
    let combined_rating = combined_difficulty_value(color, rhythm, stamina);

    DifficultyValues::eval(
        &mut attrs,
        color_rating,
        rhythm_rating,
        stamina_rating,
        combined_rating,
    );

    attrs
}

fn combined_difficulty_value(color: Color, rhythm: Rhythm, stamina: Stamina) -> f64 {
    fn norm(p: f64, values: [f64; 2]) -> f64 {
        values
            .into_iter()
            .fold(0.0, |sum, x| sum + x.powf(p))
            .powf(p.recip())
    }

    let color_peaks = color.get_curr_strain_peaks();
    let rhythm_peaks = rhythm.get_curr_strain_peaks();
    let stamina_peaks = stamina.get_curr_strain_peaks();

    let cap = cmp::min(
        cmp::min(color_peaks.len(), rhythm_peaks.len()),
        stamina_peaks.len(),
    );
    let mut peaks = Vec::with_capacity(cap);

    let iter = color_peaks
        .iter()
        .zip(rhythm_peaks.iter())
        .zip(stamina_peaks.iter());

    for ((mut color_peak, mut rhythm_peak), mut stamina_peak) in iter {
        color_peak *= COLOR_SKILL_MULTIPLIER;
        rhythm_peak *= RHYTHM_SKILL_MULTIPLIER;
        stamina_peak *= STAMINA_SKILL_MULTIPLIER;

        let mut peak = norm(1.5, [color_peak, stamina_peak]);
        peak = norm(2.0, [peak, rhythm_peak]);

        // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
        // * These sections will not contribute to the difficulty.
        if peak > 0.0 {
            peaks.push(peak);
        }
    }

    let mut difficulty = 0.0;
    let mut weight = 1.0;

    peaks.sort_by(|a, b| b.total_cmp(a));

    for strain in peaks {
        difficulty += strain * weight;
        weight *= 0.9;
    }

    difficulty
}

fn rescale(stars: f64) -> f64 {
    if stars < 0.0 {
        stars
    } else {
        10.43 * (stars / 8.0 + 1.0).ln()
    }
}

pub struct DifficultyValues {
    pub skills: TaikoSkills,
    pub max_combo: u32,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, converted: &TaikoBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut n_diff_objects = 0;
        let mut max_combo = 0;

        let diff_objects = Self::create_difficulty_objects(
            converted,
            take as u32,
            clock_rate,
            &mut max_combo,
            &mut n_diff_objects,
        );

        // The first two hit objects have no difficulty object
        n_diff_objects = n_diff_objects.saturating_sub(2);

        let mut skills = TaikoSkills::new();

        {
            let mut rhythm = Skill::new(&mut skills.rhythm, &diff_objects);
            let mut color = Skill::new(&mut skills.color, &diff_objects);
            let mut stamina = Skill::new(&mut skills.stamina, &diff_objects);

            for hit_object in diff_objects.iter().take(n_diff_objects) {
                rhythm.process(&hit_object.get());
                color.process(&hit_object.get());
                stamina.process(&hit_object.get());
            }
        }

        Self { skills, max_combo }
    }

    pub fn eval(
        attrs: &mut TaikoDifficultyAttributes,
        color_difficulty_value: f64,
        rhythm_difficulty_value: f64,
        stamina_difficulty_value: f64,
        combined_difficulty_value: f64,
    ) {
        let color_rating = color_difficulty_value * COLOR_SKILL_MULTIPLIER;
        let rhythm_rating = rhythm_difficulty_value * RHYTHM_SKILL_MULTIPLIER;
        let stamina_rating = stamina_difficulty_value * STAMINA_SKILL_MULTIPLIER;
        let combined_rating = combined_difficulty_value;

        let star_rating = rescale(combined_rating * 1.4);

        attrs.stamina = stamina_rating;
        attrs.rhythm = rhythm_rating;
        attrs.color = color_rating;
        attrs.peak = combined_rating;
        attrs.stars = star_rating;
    }

    pub fn create_difficulty_objects(
        converted: &TaikoBeatmap<'_>,
        take: u32,
        clock_rate: f64,
        max_combo: &mut u32,
        n_diff_objects: &mut usize,
    ) -> TaikoDifficultyObjects {
        let mut hit_objects_iter = converted
            .hit_objects
            .iter()
            .zip(converted.hit_sounds.iter())
            .map(|(h, s)| TaikoObject::new(h, *s))
            .inspect(|h| {
                if *max_combo < take {
                    *n_diff_objects += 1;
                    *max_combo += u32::from(h.is_hit());
                }
            });

        let Some((mut last_last, mut last)) = hit_objects_iter.next().zip(hit_objects_iter.next())
        else {
            return TaikoDifficultyObjects::with_capacity(0);
        };

        let mut diff_objects =
            TaikoDifficultyObjects::with_capacity(converted.hit_objects.len() - 2);

        for (i, curr) in hit_objects_iter.enumerate() {
            let diff_object = TaikoDifficultyObject::new(
                &curr,
                &last,
                &last_last,
                clock_rate,
                i,
                &mut diff_objects,
            );

            diff_objects.push(diff_object);

            last_last = last;
            last = curr;
        }

        ColorDifficultyPreprocessor::process_and_assign(&diff_objects);

        diff_objects
    }
}
