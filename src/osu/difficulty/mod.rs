use std::{cmp, pin::Pin};

use rosu_map::section::general::GameMode;
use skills::{aim::Aim, flashlight::Flashlight, speed::Speed, strain::OsuStrainSkill};

use crate::{
    Beatmap,
    any::difficulty::{Difficulty, skills::StrainSkill},
    model::{beatmap::BeatmapAttributes, mode::ConvertError, mods::GameMods},
    osu::{
        convert::convert_objects,
        difficulty::{
            object::OsuDifficultyObject, rating::OsuRatingCalculator,
            scaling_factor::ScalingFactor, skills::strain::count_top_weighted_sliders,
        },
        legacy_score_simulator::OsuLegacyScoreSimulator,
        object::OsuObject,
        performance::PERFORMANCE_BASE_MULTIPLIER,
    },
};

use self::skills::OsuSkills;

use super::attributes::OsuDifficultyAttributes;

pub mod gradual;
mod object;
pub mod rating;
pub mod scaling_factor;
pub mod skills;

const STAR_RATING_MULTIPLIER: f64 = 0.0265;

const HD_FADE_IN_DURATION_MULTIPLIER: f64 = 0.4;
const HD_FADE_OUT_DURATION_MULTIPLIER: f64 = 0.3;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<OsuDifficultyAttributes, ConvertError> {
    let map = map.convert_ref(GameMode::Osu, difficulty.get_mods())?;

    let DifficultyValues {
        osu_objects,
        skills,
        mut attrs,
        map_attrs,
    } = DifficultyValues::calculate(difficulty, &map);

    let mods = difficulty.get_mods();

    DifficultyValues::eval(&mut attrs, mods, &skills);

    let mut simulator = OsuLegacyScoreSimulator::new(
        &osu_objects,
        &map.breaks,
        &map_attrs,
        difficulty.get_passed_objects(),
    );

    let score_attrs = simulator.simulate();
    attrs.maximum_legacy_combo_score = score_attrs.combo_score as f64;
    attrs.legacy_score_base_multiplier = simulator.score_multiplier;

    Ok(attrs)
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
        let scaling_factor = ScalingFactor::new(map_attrs.cs);

        let attrs = OsuDifficultyAttributes {
            ar: map_attrs.ar,
            hp: map_attrs.hp,
            great_hit_window: map_attrs.hit_windows.od_great,
            ok_hit_window: map_attrs.hit_windows.od_ok.unwrap_or(0.0),
            meh_hit_window: map_attrs.hit_windows.od_meh.unwrap_or(0.0),
            ..Default::default()
        };

        let time_preempt = f64::from((map_attrs.hit_windows.ar * clock_rate) as f32);

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
    pub map_attrs: BeatmapAttributes,
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

        let mut skills = OsuSkills::new(mods, &scaling_factor, &map_attrs, time_preempt);

        // The first hit object has no difficulty object
        let take_diff_objects = cmp::min(map.hit_objects.len(), take).saturating_sub(1);

        for hit_object in diff_objects.iter().take(take_diff_objects) {
            skills.process(hit_object, &diff_objects);
        }

        Self {
            osu_objects,
            skills,
            attrs,
            map_attrs,
        }
    }

    /// Process the difficulty values and store the results in `attrs`.
    pub fn eval(attrs: &mut OsuDifficultyAttributes, mods: &GameMods, skills: &OsuSkills) {
        let OsuSkills {
            aim,
            aim_no_sliders,
            speed,
            flashlight,
        } = skills;

        let aim_difficulty_value = aim.cloned_difficulty_value();

        let aim_difficult_strain_count = aim.count_top_weighted_strains(aim_difficulty_value);

        let difficult_sliders = aim.get_difficult_sliders();

        let aim_no_sliders_difficulty_value = aim_no_sliders.cloned_difficulty_value();

        let aim_no_sliders_top_weighted_slider_count = count_top_weighted_sliders(
            aim_no_sliders.slider_strains(),
            aim_no_sliders_difficulty_value,
        );

        let aim_no_sliders_difficult_strain_count =
            aim_no_sliders.count_top_weighted_strains(aim_no_sliders_difficulty_value);

        let aim_top_weighted_slider_factor = aim_no_sliders_top_weighted_slider_count
            / (aim_no_sliders_difficult_strain_count - aim_no_sliders_top_weighted_slider_count)
                .max(1.0);

        let slider_factor = if aim_difficulty_value > 0.0 {
            OsuRatingCalculator::calculate_difficulty_rating(aim_no_sliders_difficulty_value)
                / OsuRatingCalculator::calculate_difficulty_rating(aim_difficulty_value)
        } else {
            1.0
        };

        let speed_difficulty_value = speed.cloned_difficulty_value();
        let speed_top_weighted_slider_count =
            count_top_weighted_sliders(speed.slider_strains(), speed_difficulty_value);

        let speed_difficult_strain_count = speed.count_top_weighted_strains(speed_difficulty_value);

        let speed_top_weighted_slider_factor = speed_top_weighted_slider_count
            / (speed_difficult_strain_count - speed_top_weighted_slider_count).max(1.0);

        let mechanical_difficulty_rating =
            calculate_mechanical_difficulty_rating(aim_difficulty_value, speed_difficulty_value);

        let osu_rating_calculator = OsuRatingCalculator::new(
            mods,
            attrs.n_objects(),
            attrs.ar,
            attrs.od(),
            mechanical_difficulty_rating,
            slider_factor,
        );

        let aim_rating = osu_rating_calculator.compute_aim_rating(aim_difficulty_value);
        let speed_rating = osu_rating_calculator.compute_speed_rating(speed_difficulty_value);

        let flashlight_rating = if mods.fl() {
            let flashlight_difficulty_value = flashlight.cloned_difficulty_value();

            osu_rating_calculator.compute_flashlight_rating(flashlight_difficulty_value)
        } else {
            0.0
        };

        // TODO: sliderNestedScorePerObject

        let base_aim_performance = Aim::difficulty_to_performance(aim_rating);
        let base_speed_performance = Speed::difficulty_to_performance(speed_rating);
        let base_flashlight_performance = Flashlight::difficulty_to_performance(flashlight_rating);

        let base_performance = ((base_aim_performance).powf(1.1)
            + (base_speed_performance).powf(1.1)
            + (base_flashlight_performance).powf(1.1))
        .powf(1.0 / 1.1);

        let star_rating = calculate_star_rating(base_performance);

        attrs.aim = aim_rating;
        attrs.aim_difficult_slider_count = difficult_sliders;
        attrs.speed = speed_rating;
        attrs.flashlight = flashlight_rating;
        attrs.slider_factor = slider_factor;
        attrs.aim_top_weighted_slider_factor = aim_top_weighted_slider_factor;
        attrs.speed_top_weighted_slider_factor = speed_top_weighted_slider_factor;
        attrs.aim_difficult_strain_count = aim_difficult_strain_count;
        attrs.speed_difficult_strain_count = speed_difficult_strain_count;
        attrs.stars = star_rating;
        attrs.speed_note_count = speed.relevant_note_count();
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

fn calculate_mechanical_difficulty_rating(
    aim_difficulty_value: f64,
    speed_difficulty_value: f64,
) -> f64 {
    let aim_value = Aim::difficulty_to_performance(
        OsuRatingCalculator::calculate_difficulty_rating(aim_difficulty_value),
    );
    let speed_value = Speed::difficulty_to_performance(
        OsuRatingCalculator::calculate_difficulty_rating(speed_difficulty_value),
    );

    let total_value = (aim_value.powf(1.1) + speed_value.powf(1.1)).powf(1.0 / 1.1);

    calculate_star_rating(total_value)
}

fn calculate_star_rating(base_performance: f64) -> f64 {
    if base_performance <= 0.00001 {
        return 0.0;
    }

    PERFORMANCE_BASE_MULTIPLIER.cbrt()
        * STAR_RATING_MULTIPLIER
        * ((100_000.0 / 2.0_f64.powf(1.0 / 1.1) * base_performance).cbrt() + 4.0)
}
