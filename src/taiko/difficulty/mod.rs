use std::cmp;

use rhythm::preprocessor::RhythmDifficultyPreprocessor;
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
    util::difficulty::norm,
    Beatmap, Difficulty,
};

use self::skills::TaikoSkills;

use super::attributes::TaikoDifficultyAttributes;

mod color;
pub mod gradual;
mod object;
mod rhythm;
mod skills;

#[allow(clippy::unreadable_literal)]
const DIFFICULTY_MULTIPLIER: f64 = 0.084375;
const RHYTHM_SKILL_MULTIPLIER: f64 = 0.65 * DIFFICULTY_MULTIPLIER;
const READING_SKILL_MULTIPLIER: f64 = 0.100 * DIFFICULTY_MULTIPLIER;
const COLOR_SKILL_MULTIPLIER: f64 = 0.375 * DIFFICULTY_MULTIPLIER;
const STAMINA_SKILL_MULTIPLIER: f64 = 0.445 * DIFFICULTY_MULTIPLIER;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<TaikoDifficultyAttributes, ConvertError> {
    let map = map.convert_ref(GameMode::Taiko, difficulty.get_mods())?;

    let HitWindows {
        od_great,
        od_ok,
        od_meh: _,
        ar: _,
    } = map.attributes().difficulty(difficulty).hit_windows();

    let DifficultyValues { skills, max_combo } =
        DifficultyValues::calculate(difficulty, &map, od_great);

    let mut attrs = TaikoDifficultyAttributes {
        great_hit_window: od_great,
        ok_hit_window: od_ok.unwrap_or(0.0),
        max_combo,
        is_convert: map.is_convert,
        ..Default::default()
    };

    let is_relax = difficulty.get_mods().rx();

    DifficultyValues::eval(&mut attrs, skills, is_relax);

    Ok(attrs)
}

fn combined_difficulty_value(
    skills: TaikoSkills,
    is_relax: bool,
    is_convert: bool,
    pattern_multiplier: f64,
    strain_length_bonus: f64,
) -> f64 {
    let TaikoSkills {
        rhythm,
        reading,
        color,
        stamina,
        single_color_stamina: _,
    } = skills;

    let rhythm_peaks = rhythm.get_curr_strain_peaks();
    let reading_peaks = reading.get_curr_strain_peaks();
    let color_peaks = color.get_curr_strain_peaks();
    let stamina_peaks = stamina.get_curr_strain_peaks();

    let cap = cmp::min(
        cmp::min(color_peaks.len(), rhythm_peaks.len()),
        stamina_peaks.len(),
    );
    let mut peaks = Vec::with_capacity(cap);

    let iter = rhythm_peaks
        .iter()
        .zip(reading_peaks.iter())
        .zip(color_peaks.iter())
        .zip(stamina_peaks.iter());

    for (((mut rhythm_peak, mut reading_peak), mut color_peak), mut stamina_peak) in iter {
        rhythm_peak *= RHYTHM_SKILL_MULTIPLIER * pattern_multiplier;
        reading_peak *= READING_SKILL_MULTIPLIER;

        color_peak *= if is_relax {
            0.0 // * There is no colour difficulty in relax.
        } else {
            COLOR_SKILL_MULTIPLIER
        };

        stamina_peak *= STAMINA_SKILL_MULTIPLIER * strain_length_bonus;

        // * Available finger count is increased by 150%, thus we adjust accordingly.
        stamina_peak /= if is_convert || is_relax { 1.5 } else { 1.0 };

        let peak = norm(
            2.0,
            [
                norm(1.5, [color_peak, stamina_peak]),
                rhythm_peak,
                reading_peak,
            ],
        );

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
        10.43 * f64::ln(stars / 8.0 + 1.0)
    }
}

pub struct DifficultyValues {
    pub skills: TaikoSkills,
    pub max_combo: u32,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, converted: &Beatmap, great_hit_window: f64) -> Self {
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

        let mut skills = TaikoSkills::new(great_hit_window, converted.is_convert);

        {
            let mut rhythm = Skill::new(&mut skills.rhythm, &diff_objects);
            let mut reading = Skill::new(&mut skills.reading, &diff_objects);
            let mut color = Skill::new(&mut skills.color, &diff_objects);
            let mut stamina = Skill::new(&mut skills.stamina, &diff_objects);
            let mut single_color_stamina =
                Skill::new(&mut skills.single_color_stamina, &diff_objects);

            for hit_object in diff_objects.iter().take(n_diff_objects) {
                rhythm.process(&hit_object.get());
                reading.process(&hit_object.get());
                color.process(&hit_object.get());
                stamina.process(&hit_object.get());
                single_color_stamina.process(&hit_object.get());
            }
        }

        Self { skills, max_combo }
    }

    pub fn eval(attrs: &mut TaikoDifficultyAttributes, skills: TaikoSkills, is_relax: bool) {
        let used_rhythm = skills.rhythm.as_difficulty_value();
        let used_color = skills.color.as_difficulty_value();
        let used_stamina = skills.stamina.as_difficulty_value();

        let rhythm_rating = used_rhythm.difficulty_value() * RHYTHM_SKILL_MULTIPLIER;
        let reading_rating = skills.reading.as_difficulty_value() * READING_SKILL_MULTIPLIER;
        let color_rating = used_color.difficulty_value() * COLOR_SKILL_MULTIPLIER;
        let stamina_rating = used_stamina.difficulty_value() * STAMINA_SKILL_MULTIPLIER;
        let mono_stamina_rating = skills
            .single_color_stamina
            .as_difficulty_value()
            .difficulty_value()
            * STAMINA_SKILL_MULTIPLIER;
        let mono_stamina_factor = if stamina_rating.abs() >= f64::EPSILON {
            (mono_stamina_rating / stamina_rating).powf(5.0)
        } else {
            1.0
        };

        let color_difficult_strains = used_color.count_top_weighted_strains();
        let rhythm_difficult_strains = used_rhythm.count_top_weighted_strains();
        let stamina_difficult_strains = used_stamina.count_top_weighted_strains();

        // * As we don't have pattern integration in osu!taiko, we apply the other two skills relative to rhythm.
        let pattern_multiplier = f64::powf(stamina_rating * color_rating, 0.10);

        #[allow(clippy::manual_clamp)]
        let strain_length_bonus =
            1.0 + f64::min(
                f64::max((stamina_difficult_strains - 1000.0) / 3700.0, 0.0),
                0.15,
            ) + f64::min(f64::max((stamina_rating - 7.0) / 1.0, 0.0), 0.05);

        let combined_rating = combined_difficulty_value(
            skills,
            is_relax,
            attrs.is_convert,
            pattern_multiplier,
            strain_length_bonus,
        );
        let star_rating = rescale(combined_rating * 1.4);

        attrs.rhythm = rhythm_rating;
        attrs.reading = reading_rating;
        attrs.color = color_rating;
        attrs.stamina = stamina_rating;
        attrs.mono_stamina_factor = mono_stamina_factor;
        attrs.rhythm_top_strains = rhythm_difficult_strains;
        attrs.color_top_strains = color_difficult_strains;
        attrs.stamina_top_strains = stamina_difficult_strains;
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

        let Some(mut last) = hit_objects_iter.next() else {
            return TaikoDifficultyObjects::with_capacity(0);
        };

        let mut diff_objects =
            TaikoDifficultyObjects::with_capacity(converted.hit_objects.len() - 2);

        for (i, curr) in hit_objects_iter.enumerate() {
            let diff_object = TaikoDifficultyObject::new(
                &curr,
                &last,
                clock_rate,
                i,
                converted,
                &mut diff_objects,
            );

            diff_objects.push(diff_object);
            last = curr;
        }

        ColorDifficultyPreprocessor::process_and_assign(&diff_objects);
        RhythmDifficultyPreprocessor::process_and_assign(&diff_objects);

        diff_objects
    }
}
