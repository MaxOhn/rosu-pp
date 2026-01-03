use std::cmp;

use rhythm::preprocessor::RhythmDifficultyPreprocessor;
use rosu_map::section::general::GameMode;
use skills::{color::Color, reading::Reading, rhythm::Rhythm, stamina::Stamina};

use crate::{
    Beatmap, Difficulty, GameMods,
    any::difficulty::skills::StrainSkill,
    model::{beatmap::HitWindows, mode::ConvertError},
    taiko::{
        difficulty::{
            color::preprocessor::ColorDifficultyPreprocessor,
            object::{TaikoDifficultyObject, TaikoDifficultyObjects},
        },
        object::TaikoObject,
    },
    util::difficulty::{norm, reverse_lerp},
};

pub(crate) use self::skills::TaikoSkills;

use super::{attributes::TaikoDifficultyAttributes, convert};

mod color;
mod evaluators;
pub mod gradual;
mod object;
mod rhythm;
mod skills;
mod utils;

#[expect(clippy::unreadable_literal, reason = "staying in-sync with lazer")]
const DIFFICULTY_MULTIPLIER: f64 = 0.084375;
const RHYTHM_SKILL_MULTIPLIER: f64 = 0.75 * DIFFICULTY_MULTIPLIER;
const READING_SKILL_MULTIPLIER: f64 = 0.100 * DIFFICULTY_MULTIPLIER;
const COLOR_SKILL_MULTIPLIER: f64 = 0.375 * DIFFICULTY_MULTIPLIER;
const STAMINA_SKILL_MULTIPLIER: f64 = 0.445 * DIFFICULTY_MULTIPLIER;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<TaikoDifficultyAttributes, ConvertError> {
    let mut map = map.convert_ref(GameMode::Taiko, difficulty.get_mods())?;

    if let Some(seed) = difficulty.get_mods().random_seed() {
        convert::apply_random_to_beatmap(map.to_mut(), seed);
    }

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

/// Returns the combined rating and the consistency factor
#[expect(clippy::too_many_arguments, reason = "it is what it is /shrug")]
fn combined_difficulty_value(
    rhythm: Rhythm,
    reading: Reading,
    color: Color,
    stamina: Stamina,
    is_relax: bool,
    is_convert: bool,
    pattern_multiplier: f64,
    strain_length_bonus: f64,
) -> (f64, f64) {
    let mut hit_object_strain_peaks = combine_peaks(
        rhythm.object_strains().iter().copied(),
        reading.object_strains().iter().copied(),
        color.object_strains().iter().copied(),
        stamina.object_strains().iter().copied(),
        rhythm.object_strains().len(),
        is_relax,
        is_convert,
        pattern_multiplier,
        strain_length_bonus,
    );

    let rhythm_peaks = rhythm.into_current_strain_peaks();
    let reading_peaks = reading.into_current_strain_peaks();
    let color_peaks = color.into_current_strain_peaks();
    let stamina_peaks = stamina.into_current_strain_peaks();

    let len = cmp::min(
        rhythm_peaks.len(),
        cmp::min(
            reading_peaks.len(),
            cmp::min(color_peaks.len(), stamina_peaks.len()),
        ),
    );

    let mut peaks = combine_peaks(
        rhythm_peaks.iter(),
        reading_peaks.iter(),
        color_peaks.iter(),
        stamina_peaks.iter(),
        len,
        is_relax,
        is_convert,
        pattern_multiplier,
        strain_length_bonus,
    );

    if peaks.is_empty() {
        return (0.0, 0.0);
    }

    let mut difficulty = 0.0;
    let mut weight = 1.0;

    peaks.sort_by(|a, b| b.total_cmp(a));

    for strain in peaks {
        difficulty += strain * weight;
        weight *= 0.9;
    }

    if hit_object_strain_peaks.is_empty() {
        return (0.0, 0.0);
    }

    // * The average of the top 5% of strain peaks from hit objects.
    let take = cmp::min(
        1 + hit_object_strain_peaks.len() / 20,
        hit_object_strain_peaks.len(),
    );

    let (bigger, nth, _) =
        hit_object_strain_peaks.select_nth_unstable_by(take - 1, |a, b| b.total_cmp(a));

    let top_average_hit_object_strain = (bigger.iter().sum::<f64>() + *nth) / take as f64;

    // * Calculates a consistency factor as the sum of difficulty from hit objects compared to if every object were as hard as the hardest.
    // * The top average strain is used instead of the very hardest to prevent exceptionally hard objects lowering the factor.
    let consistency_factor = hit_object_strain_peaks.iter().sum::<f64>()
        / (top_average_hit_object_strain * hit_object_strain_peaks.len() as f64);

    (difficulty, consistency_factor)
}

#[expect(clippy::too_many_arguments, reason = "it is what it is /shrug")]
fn combine_peaks<I>(
    rhythm_peaks: I,
    reading_peaks: I,
    color_peaks: I,
    stamina_peaks: I,
    len: usize,
    is_relax: bool,
    is_convert: bool,
    pattern_multiplier: f64,
    strain_length_bonus: f64,
) -> Vec<f64>
where
    I: IntoIterator<Item = f64>,
{
    // Combatting size bloat by monomorphization due to generics
    #[expect(clippy::too_many_arguments, reason = "it is what it is /shrug")]
    fn iteration(
        rhythm_peak: &mut f64,
        reading_peak: &mut f64,
        color_peak: &mut f64,
        stamina_peak: &mut f64,
        is_relax: bool,
        is_convert: bool,
        pattern_multiplier: f64,
        strain_length_bonus: f64,
        combined_peaks: &mut Vec<f64>,
    ) {
        *rhythm_peak *= RHYTHM_SKILL_MULTIPLIER;
        *rhythm_peak *= pattern_multiplier;

        *color_peak *= if is_relax {
            0.0 // * There is no colour difficulty in relax.
        } else {
            COLOR_SKILL_MULTIPLIER
        };

        *reading_peak *= READING_SKILL_MULTIPLIER;

        *stamina_peak *= STAMINA_SKILL_MULTIPLIER;
        *stamina_peak *= strain_length_bonus;

        // * Available finger count is increased by 150%, thus we adjust accordingly.
        *stamina_peak /= if is_convert || is_relax { 1.5 } else { 1.0 };

        let peak = norm(
            2.0,
            [
                norm(1.5, [*color_peak, *stamina_peak]),
                *rhythm_peak,
                *reading_peak,
            ],
        );

        // * Sections with 0 strain are excluded to avoid worst-case time complexity of the following sort (e.g. /b/2351871).
        // * These sections will not contribute to the difficulty.
        if peak > 0.0 {
            combined_peaks.push(peak);
        }
    }

    let mut combined_peaks = Vec::with_capacity(len);

    let iter = rhythm_peaks
        .into_iter()
        .zip(reading_peaks)
        .zip(color_peaks)
        .zip(stamina_peaks);

    for (((mut rhythm_peak, mut reading_peak), mut color_peak), mut stamina_peak) in iter {
        iteration(
            &mut rhythm_peak,
            &mut reading_peak,
            &mut color_peak,
            &mut stamina_peak,
            is_relax,
            is_convert,
            pattern_multiplier,
            strain_length_bonus,
            &mut combined_peaks,
        );
    }

    combined_peaks
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
    pub fn calculate(difficulty: &Difficulty, map: &Beatmap, great_hit_window: f64) -> Self {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut n_diff_objects = 0;
        let mut max_combo = 0;

        let diff_objects = Self::create_difficulty_objects(
            map,
            take as u32,
            clock_rate,
            &mut max_combo,
            &mut n_diff_objects,
            difficulty.get_mods(),
        );

        // The first hit object has no difficulty object
        n_diff_objects = n_diff_objects.saturating_sub(1);

        let mut skills = TaikoSkills::new(great_hit_window, map.is_convert);

        for hit_object in diff_objects.iter().take(n_diff_objects) {
            skills.rhythm.process(&hit_object.get(), &diff_objects);
            skills.reading.process(&hit_object.get(), &diff_objects);
            skills.color.process(&hit_object.get(), &diff_objects);
            skills.stamina.process(&hit_object.get(), &diff_objects);
            skills
                .single_color_stamina
                .process(&hit_object.get(), &diff_objects);
        }

        Self { skills, max_combo }
    }

    pub fn eval(attrs: &mut TaikoDifficultyAttributes, skills: TaikoSkills, is_relax: bool) {
        let TaikoSkills {
            rhythm,
            reading,
            color,
            stamina,
            single_color_stamina,
        } = skills;

        let rhythm_difficulty_value = rhythm.cloned_difficulty_value();
        let reading_difficulty_value = reading.cloned_difficulty_value();
        let color_difficulty_value = color.cloned_difficulty_value();
        let stamina_difficulty_value = stamina.cloned_difficulty_value();

        let rhythm_skill = rhythm_difficulty_value * RHYTHM_SKILL_MULTIPLIER;
        let reading_skill = reading_difficulty_value * READING_SKILL_MULTIPLIER;
        let color_skill = color_difficulty_value * COLOR_SKILL_MULTIPLIER;
        let stamina_skill = stamina_difficulty_value * STAMINA_SKILL_MULTIPLIER;
        let mono_stamina_rating =
            single_color_stamina.into_difficulty_value() * STAMINA_SKILL_MULTIPLIER;
        let mono_stamina_factor = if stamina_skill.abs() >= f64::EPSILON {
            (mono_stamina_rating / stamina_skill).powf(5.0)
        } else {
            1.0
        };

        let stamina_difficult_strains =
            stamina.count_top_weighted_strains(stamina_difficulty_value);

        // * As we don't have pattern integration in osu!taiko, we apply the other two skills relative to rhythm.
        let pattern_multiplier = f64::powf(stamina_skill * color_skill, 0.10);

        let strain_length_bonus =
            1.0 + 0.15 * reverse_lerp(stamina_difficult_strains, 1000.0, 1555.0);

        // TODO: UPDATE ME
        let (combined_rating, consistency_factor) = combined_difficulty_value(
            rhythm,
            reading,
            color,
            stamina,
            is_relax,
            attrs.is_convert,
            pattern_multiplier,
            strain_length_bonus,
        );
        let star_rating = rescale(combined_rating * 1.4);

        // * Calculate proportional contribution of each skill to the combinedRating.
        let skill_rating =
            star_rating / (rhythm_skill + reading_skill + color_skill + stamina_skill);
        let rhythm_difficulty = rhythm_skill * skill_rating;
        let reading_difficulty = reading_skill * skill_rating;
        let color_difficulty = color_skill * skill_rating;
        let stamina_difficulty = stamina_skill * skill_rating;

        // * Mechanical difficulty is the sum of colour and stamina difficulties.
        let mechanical_difficulty = color_difficulty + stamina_difficulty;

        attrs.rhythm = rhythm_difficulty;
        attrs.reading = reading_difficulty;
        attrs.color = color_difficulty;
        attrs.stamina = stamina_difficulty;
        attrs.mono_stamina_factor = mono_stamina_factor;
        attrs.mechanical_difficulty = mechanical_difficulty;
        attrs.consistency_factor = consistency_factor;
        attrs.stars = star_rating;
    }

    pub fn create_difficulty_objects(
        converted: &Beatmap,
        take: u32,
        clock_rate: f64,
        max_combo: &mut u32,
        n_diff_objects: &mut usize,
        mods: &GameMods,
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
            })
            .skip(1);

        let Some(mut last) = hit_objects_iter.next() else {
            return TaikoDifficultyObjects::with_capacity(0);
        };

        let mut diff_objects =
            TaikoDifficultyObjects::with_capacity(converted.hit_objects.len() - 2);

        let mut global_slider_velocity = converted.slider_multiplier;

        if mods.hr() {
            const SLIDER_MULTIPLIER: f64 = 1.4 * 4.0 / 3.0;

            global_slider_velocity *= SLIDER_MULTIPLIER;
        } else if mods.ez() {
            const SLIDER_MULTIPLIER: f64 = 0.8;

            global_slider_velocity *= SLIDER_MULTIPLIER;
        }

        if let Some(scroll_speed) = mods.scroll_speed() {
            global_slider_velocity *= f64::from(scroll_speed as f32);
        }

        for (i, curr) in hit_objects_iter.enumerate() {
            let diff_object = TaikoDifficultyObject::new(
                &curr,
                &last,
                clock_rate,
                i,
                converted,
                global_slider_velocity,
                &mut diff_objects,
            );

            diff_objects.push(diff_object);
            last = curr;
        }

        // The first hit object is currently straight up skipped and not
        // considered for the first (or any other) difficulty object
        if take > 0 && *n_diff_objects > 0 {
            *n_diff_objects -= 1;
        }

        ColorDifficultyPreprocessor::process_and_assign(&diff_objects);
        RhythmDifficultyPreprocessor::process_and_assign(&diff_objects);

        diff_objects
    }
}
