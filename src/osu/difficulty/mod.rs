use std::{cmp, pin::Pin};

use rosu_map::section::general::GameMode;
use skills::{aim::Aim, flashlight::Flashlight, speed::Speed};

use crate::{
    Beatmap,
    any::{
        CalculateError,
        difficulty::{
            Difficulty,
            skills_new::{
                harmonic_skill::HarmonicSkill,
                variable_length_strain_skill::VariableLengthStrainSkill,
            },
        },
    },
    model::{beatmap::BeatmapAttributes, mode::ConvertError, mods::GameMods},
    osu::{
        convert::{convert_objects, prepare_map},
        difficulty::{
            object::OsuDifficultyObject, scaling_factor::ScalingFactor, skills::reading::Reading,
        },
        legacy_score_simulator::OsuLegacyScoreSimulator,
        object::OsuObject,
        performance::{PERFORMANCE_BASE_MULTIPLIER, PERFORMANCE_NORM_EXPONENT},
        utils::legacy_score::NestedScorePerObject,
    },
    util::difficulty::norm,
};

use self::skills::OsuSkills;

use super::attributes::OsuDifficultyAttributes;

mod evaluators;
pub mod gradual;
mod object;
pub mod scaling_factor;
pub mod skills;

const HD_FADE_IN_DURATION_MULTIPLIER: f64 = 0.4;
const HD_FADE_OUT_DURATION_MULTIPLIER: f64 = 0.3;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<OsuDifficultyAttributes, ConvertError> {
    let map = prepare_map(difficulty, map)?;

    Ok(calculate_difficulty(difficulty, &map))
}

pub fn checked_difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<OsuDifficultyAttributes, CalculateError> {
    let map = prepare_map(difficulty, map)?;
    map.check_suspicion()?;

    Ok(calculate_difficulty(difficulty, &map))
}

fn calculate_difficulty(difficulty: &Difficulty, map: &Beatmap) -> OsuDifficultyAttributes {
    debug_assert_eq!(map.mode, GameMode::Osu);

    let DifficultyValues {
        osu_objects,
        skills,
        mut attrs,
    } = DifficultyValues::calculate(difficulty, map);

    let mods = difficulty.get_mods();
    let passed_objects = difficulty.get_passed_objects();

    DifficultyValues::eval(&mut attrs, mods, &skills);

    let mut simulator = OsuLegacyScoreSimulator::new(&osu_objects, map, passed_objects);

    let score_attrs = simulator.simulate();
    attrs.maximum_legacy_combo_score = score_attrs.combo_score as f64;

    let map_attrs = map.attributes().difficulty(difficulty).build();

    attrs.legacy_score_base_multiplier = f64::from(OsuLegacyScoreSimulator::score_multiplier(
        map,
        &map_attrs,
        passed_objects,
    ));

    let slider_nested_score_per_object =
        NestedScorePerObject::calculate(&osu_objects, passed_objects);
    attrs.nested_score_per_object = slider_nested_score_per_object;

    attrs
}

pub struct OsuDifficultySetup {
    scaling_factor: ScalingFactor,
    map_attrs: BeatmapAttributes,
    attrs: OsuDifficultyAttributes,
    time_preempt: f64,
}

impl OsuDifficultySetup {
    pub fn new(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let clock_rate = difficulty.get_clock_rate();
        let map_attrs = map.attributes().difficulty(difficulty).build();
        let hit_windows = map_attrs.hit_windows();
        let scaling_factor = ScalingFactor::new(map_attrs.cs());

        let attrs = OsuDifficultyAttributes {
            ar: map_attrs.apply_clock_rate().ar,
            hp: f64::from(map_attrs.hp()),
            great_hit_window: hit_windows.od_great.unwrap_or(0.0),
            ok_hit_window: hit_windows.od_ok.unwrap_or(0.0),
            meh_hit_window: hit_windows.od_meh.unwrap_or(0.0),
            ..Default::default()
        };

        let time_preempt = f64::from((hit_windows.ar.unwrap_or(0.0) * clock_rate).trunc() as f32);

        Self {
            scaling_factor,
            map_attrs,
            attrs,
            time_preempt,
        }
    }
}

pub struct DifficultyValues {
    pub osu_objects: Box<[OsuObject]>,
    pub skills: OsuSkills,
    pub attrs: OsuDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let mods = difficulty.get_mods();
        let take = difficulty.get_passed_objects();

        let OsuDifficultySetup {
            scaling_factor,
            map_attrs,
            mut attrs,
            time_preempt,
        } = OsuDifficultySetup::new(difficulty, map);

        let mut osu_objects = convert_objects(
            map,
            &scaling_factor,
            mods.reflection(),
            time_preempt,
            take,
            &mut attrs,
        );

        let osu_object_iter = osu_objects.iter_mut().map(Pin::new);

        let diff_objects =
            Self::create_difficulty_objects(difficulty, &scaling_factor, osu_object_iter);

        let great_hit_window = map_attrs.hit_windows().od_great.unwrap_or(0.0);
        let clock_rate = difficulty.get_clock_rate();

        let take_hit_objects = cmp::min(map.hit_objects.len(), take);

        // The first hit object has no difficulty object
        let take_diff_objects = take_hit_objects.saturating_sub(1);

        let mut skills = OsuSkills::new(
            mods,
            &scaling_factor,
            great_hit_window,
            time_preempt,
            clock_rate,
            take_hit_objects,
        );

        for hit_object in diff_objects.iter().take(take_diff_objects) {
            skills.process(hit_object, &diff_objects);
        }

        Self {
            osu_objects,
            skills,
            attrs,
        }
    }

    /// Process the difficulty values and store the results in `attrs`.
    pub fn eval(attrs: &mut OsuDifficultyAttributes, mods: &GameMods, skills: &OsuSkills) {
        let OsuSkills {
            aim,
            aim_no_sliders,
            speed,
            flashlight,
            reading,
        } = skills;

        let aim_difficulty_value = aim.cloned_difficulty_value();
        let aim_no_sliders_difficulty_value = aim_no_sliders.cloned_difficulty_value();
        let (speed_difficulty_value, speed_object_weight_sum) = speed.cloned_difficulty_value();
        let (reading_difficulty_value, reading_object_weight_sum) =
            reading.cloned_difficulty_value();

        let aim_difficult_strain_count = aim.count_top_weighted_strains(aim_difficulty_value);
        let speed_difficult_strain_count = speed.count_top_weighted_object_difficulties(
            speed_difficulty_value,
            speed_object_weight_sum,
        );
        let reading_difficult_note_count = reading.count_top_weighted_object_difficulties(
            reading_difficulty_value,
            reading_object_weight_sum,
        );

        let speed_notes = speed.relevant_object_count();

        let aim_no_sliders_top_weighted_slider_count =
            aim_no_sliders.count_top_weighted_sliders(aim_no_sliders_difficulty_value);
        let aim_no_sliders_difficult_strain_count =
            aim_no_sliders.count_top_weighted_strains(aim_no_sliders_difficulty_value);

        let aim_top_weighted_slider_factor = aim_no_sliders_top_weighted_slider_count
            / (aim_no_sliders_difficult_strain_count - aim_no_sliders_top_weighted_slider_count)
                .max(1.0);

        let speed_top_weighted_slider_count =
            speed.count_top_weighted_sliders(speed_difficulty_value, speed_object_weight_sum);
        let speed_top_weighted_slider_factor = speed_top_weighted_slider_count
            / (speed_difficult_strain_count - speed_top_weighted_slider_count).max(1.0);

        let difficult_sliders = aim.get_difficult_sliders();

        let aim_rating = calculate_aim_difficulty_rating(aim_difficulty_value);
        let aim_no_sliders_rating =
            calculate_aim_difficulty_rating(aim_no_sliders_difficulty_value);

        let slider_factor = if aim_difficulty_value > 0.0 {
            aim_no_sliders_rating / aim_rating
        } else {
            1.0
        };

        let speed_rating = calculate_difficulty_rating(speed_difficulty_value);
        let reading_rating = calculate_difficulty_rating(reading_difficulty_value);

        let flashlight_rating = if mods.fl() {
            let flashlight_difficulty_value = flashlight.cloned_difficulty_value();
            calculate_difficulty_rating(flashlight_difficulty_value)
        } else {
            0.0
        };

        let base_aim_performance = Aim::difficulty_to_performance(aim_rating);
        let base_speed_performance = Speed::difficulty_to_performance(speed_rating);
        let base_reading_performance = Reading::difficulty_to_performance(reading_rating);
        let base_flashlight_performance = Flashlight::difficulty_to_performance(flashlight_rating);
        let base_cognition_performance =
            sum_cognition_difficulty(base_reading_performance, base_flashlight_performance);

        let base_performance = norm(
            PERFORMANCE_NORM_EXPONENT,
            [
                base_aim_performance,
                base_speed_performance,
                base_cognition_performance,
            ],
        );

        let star_rating = calculate_star_rating(base_performance);

        attrs.aim = aim_rating;
        attrs.aim_difficult_slider_count = difficult_sliders;
        attrs.speed = speed_rating;
        attrs.flashlight = flashlight_rating;
        attrs.reading = reading_rating;
        attrs.slider_factor = slider_factor;
        attrs.aim_top_weighted_slider_factor = aim_top_weighted_slider_factor;
        attrs.speed_top_weighted_slider_factor = speed_top_weighted_slider_factor;
        attrs.aim_difficult_strain_count = aim_difficult_strain_count;
        attrs.speed_difficult_strain_count = speed_difficult_strain_count;
        attrs.stars = star_rating;
        attrs.speed_note_count = speed_notes;
        attrs.reading_difficult_note_count = reading_difficult_note_count;
    }

    pub fn create_difficulty_objects<'a>(
        difficulty: &Difficulty,
        scaling_factor: &ScalingFactor,
        osu_objects: impl ExactSizeIterator<Item = Pin<&'a mut OsuObject>>,
    ) -> Vec<OsuDifficultyObject<'a>> {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut osu_objects_iter = osu_objects.map(Pin::into_ref);

        let Some(mut last) = osu_objects_iter.next().filter(|_| take > 0) else {
            return Vec::new();
        };

        let mut diff_objects = Vec::with_capacity(osu_objects_iter.len());

        for (idx, h) in osu_objects_iter.enumerate() {
            let last_diff = if idx > 0 {
                diff_objects.get(idx - 1)
            } else {
                None
            };

            let last_last_diff = if idx > 1 {
                diff_objects.get(idx - 2)
            } else {
                None
            };

            let diff_object = OsuDifficultyObject::new(
                h.get_ref(),
                last.get_ref(),
                last_diff,
                last_last_diff,
                clock_rate,
                idx,
                scaling_factor,
            );

            last = h;

            diff_objects.push(diff_object);
        }

        diff_objects
    }
}

pub fn sum_cognition_difficulty(reading: f64, flashlight: f64) -> f64 {
    if reading <= 0.0 {
        return flashlight;
    }

    if flashlight <= 0.0 {
        return reading;
    }

    // * Nerf flashlight value in cognition sum when reading is greater than flashlight
    norm(
        PERFORMANCE_NORM_EXPONENT,
        [reading, flashlight * (flashlight / reading).clamp(0.25, 1.0)],
    )
}

fn calculate_aim_difficulty_rating(difficulty_value: f64) -> f64 {
    difficulty_value.powf(0.63) * 0.02275
}

fn calculate_difficulty_rating(difficulty_value: f64) -> f64 {
    difficulty_value.sqrt() * 0.0675
}

fn calculate_star_rating(base_performance: f64) -> f64 {
    f64::cbrt(base_performance * PERFORMANCE_BASE_MULTIPLIER)
}
