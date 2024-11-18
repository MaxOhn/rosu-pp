use std::cmp;

use rosu_map::section::general::GameMode;

use crate::{
    any::difficulty::skills::Skill,
    model::{beatmap::HitWindows, mode::ConvertError},
    taiko::{
        difficulty::{
            color::preprocessor::ColorDifficultyPreprocessor,
            object::{TaikoDifficultyObject, TaikoDifficultyObjects},
        },
        object::TaikoObject,
    },
    Beatmap, Difficulty,
};

use self::skills::{color::Color, rhythm::Rhythm, stamina::Stamina, TaikoSkills};

use super::attributes::TaikoDifficultyAttributes;

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
    map: &Beatmap,
) -> Result<TaikoDifficultyAttributes, ConvertError> {
    let map = map.convert_ref(GameMode::Taiko, difficulty.get_mods())?;

    let HitWindows {
        od_great,
        od_ok,
        ar: _,
    } = map.attributes().difficulty(difficulty).hit_windows();

    let DifficultyValues { skills, max_combo } = DifficultyValues::calculate(difficulty, &map);

    let mut attrs = TaikoDifficultyAttributes {
        great_hit_window: od_great,
        ok_hit_window: od_ok.unwrap_or(0.0),
        max_combo,
        is_convert: map.is_convert,
        ..Default::default()
    };

    DifficultyValues::eval(&mut attrs, skills);

    Ok(attrs)
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
    pub fn calculate(difficulty: &Difficulty, converted: &Beatmap) -> Self {
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
            let mut single_color_stamina =
                Skill::new(&mut skills.single_color_stamina, &diff_objects);

            for hit_object in diff_objects.iter().take(n_diff_objects) {
                rhythm.process(&hit_object.get());
                color.process(&hit_object.get());
                stamina.process(&hit_object.get());
                single_color_stamina.process(&hit_object.get());
            }
        }

        Self { skills, max_combo }
    }

    pub fn eval(attrs: &mut TaikoDifficultyAttributes, skills: TaikoSkills) {
        let color_rating = skills.color.as_difficulty_value() * COLOR_SKILL_MULTIPLIER;
        let rhythm_rating = skills.rhythm.as_difficulty_value() * RHYTHM_SKILL_MULTIPLIER;
        let stamina_rating = skills.stamina.as_difficulty_value() * STAMINA_SKILL_MULTIPLIER;
        let mono_stamina_rating =
            skills.single_color_stamina.as_difficulty_value() * STAMINA_SKILL_MULTIPLIER;
        let mono_stamina_factor = if stamina_rating.abs() >= f64::EPSILON {
            (mono_stamina_rating / stamina_rating).powf(5.0)
        } else {
            1.0
        };
        let combined_rating =
            combined_difficulty_value(skills.color, skills.rhythm, skills.stamina);

        let mut star_rating = rescale(combined_rating * 1.4);

        // * TODO: This is temporary measure as we don't detect abuse of multiple-input playstyles of converts within the current system.
        if attrs.is_convert {
            star_rating *= 0.925;
            // * For maps with low colour variance and high stamina requirement, multiple inputs are more likely to be abused.
            if color_rating < 2.0 && stamina_rating > 8.0 {
                star_rating *= 0.80;
            }
        }

        attrs.stamina = stamina_rating;
        attrs.rhythm = rhythm_rating;
        attrs.color = color_rating;
        attrs.peak = combined_rating;
        attrs.mono_stamina_factor = mono_stamina_factor;
        attrs.stars = star_rating;
    }

    pub fn create_difficulty_objects(
        converted: &Beatmap,
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
