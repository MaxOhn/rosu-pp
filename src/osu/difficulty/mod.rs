use std::{cmp, pin::Pin};

use crate::{
    any::difficulty::{skills::Skill, ModeDifficulty},
    model::beatmap::BeatmapAttributes,
    osu::{
        convert::convert_objects,
        difficulty::{object::OsuDifficultyObject, scaling_factor::ScalingFactor},
        object::OsuObject,
        performance::PERFORMANCE_BASE_MULTIPLIER,
    },
    util::mods::Mods,
};

use self::skills::OsuSkills;

use super::{attributes::OsuDifficultyAttributes, convert::OsuBeatmap};

pub mod gradual;
mod object;
pub mod scaling_factor;
pub mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.0675;

const HD_FADE_IN_DURATION_MULTIPLIER: f64 = 0.4;
const HD_FADE_OUT_DURATION_MULTIPLIER: f64 = 0.3;

pub fn difficulty(
    difficulty: &ModeDifficulty,
    converted: &OsuBeatmap<'_>,
) -> OsuDifficultyAttributes {
    let DifficultyValues {
        skills:
            OsuSkills {
                aim,
                aim_no_sliders,
                speed,
                flashlight,
            },
        mut attrs,
    } = DifficultyValues::calculate(difficulty, converted);

    let aim_difficulty_value = aim.difficulty_value();
    let aim_no_sliders_difficulty_value = aim_no_sliders.difficulty_value();
    let speed_relevant_note_count = speed.relevant_note_count();
    let speed_difficulty_value = speed.difficulty_value();
    let flashlight_difficulty_value = flashlight.difficulty_value();

    let mods = difficulty.get_mods();

    DifficultyValues::eval(
        &mut attrs,
        mods,
        aim_difficulty_value,
        aim_no_sliders_difficulty_value,
        speed_difficulty_value,
        speed_relevant_note_count,
        flashlight_difficulty_value,
    );

    attrs
}

pub struct OsuDifficultySetup {
    scaling_factor: ScalingFactor,
    map_attrs: BeatmapAttributes,
    attrs: OsuDifficultyAttributes,
    time_preempt: f64,
}

impl OsuDifficultySetup {
    pub fn new(difficulty: &ModeDifficulty, converted: &OsuBeatmap) -> Self {
        let mods = difficulty.get_mods();
        let clock_rate = difficulty.get_clock_rate();

        let map_attrs = converted
            .attributes()
            .mods(mods)
            .clock_rate(clock_rate)
            .build();

        let scaling_factor = ScalingFactor::new(map_attrs.cs);

        let attrs = OsuDifficultyAttributes {
            ar: map_attrs.ar,
            hp: map_attrs.hp,
            od: map_attrs.od,
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
    pub skills: OsuSkills,
    pub attrs: OsuDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &ModeDifficulty, converted: &OsuBeatmap<'_>) -> Self {
        let mods = difficulty.get_mods();
        let take = difficulty.get_passed_objects();

        let OsuDifficultySetup {
            scaling_factor,
            map_attrs,
            mut attrs,
            time_preempt,
        } = OsuDifficultySetup::new(difficulty, converted);

        let mut osu_objects = convert_objects(
            converted,
            &scaling_factor,
            mods.hr(),
            time_preempt,
            take,
            &mut attrs,
        );

        let osu_object_iter = osu_objects.iter_mut().map(Pin::new);

        let diff_objects =
            Self::create_difficulty_objects(difficulty, &scaling_factor, osu_object_iter);

        let mut skills = OsuSkills::new(mods, &scaling_factor, &map_attrs, time_preempt);

        {
            let mut aim = Skill::new(&mut skills.aim, &diff_objects);
            let mut aim_no_sliders = Skill::new(&mut skills.aim_no_sliders, &diff_objects);
            let mut speed = Skill::new(&mut skills.speed, &diff_objects);
            let mut flashlight = Skill::new(&mut skills.flashlight, &diff_objects);

            // The first hit object has no difficulty object
            let take_diff_objects =
                cmp::min(converted.map.hit_objects.len(), take).saturating_sub(1);

            for hit_object in diff_objects.iter().take(take_diff_objects) {
                aim.process(hit_object);
                aim_no_sliders.process(hit_object);
                speed.process(hit_object);
                flashlight.process(hit_object);
            }
        }

        Self { skills, attrs }
    }

    /// Process the difficulty values and store the results in `attrs`.
    pub fn eval(
        attrs: &mut OsuDifficultyAttributes,
        mods: u32,
        aim_difficulty_value: f64,
        aim_no_sliders_difficulty_value: f64,
        speed_difficulty_value: f64,
        speed_relevant_note_count: f64,
        flashlight_difficulty_value: f64,
    ) {
        let mut aim_rating = aim_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let aim_rating_no_sliders = aim_no_sliders_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let mut speed_rating = speed_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let mut flashlight_rating = flashlight_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;

        let slider_factor = if aim_rating > 0.0 {
            aim_rating_no_sliders / aim_rating
        } else {
            1.0
        };

        if mods.td() {
            aim_rating = aim_rating.powf(0.8);
            flashlight_rating = flashlight_rating.powf(0.8);
        }

        if mods.rx() {
            aim_rating *= 0.9;
            speed_rating = 0.0;
            flashlight_rating *= 0.7;
        }

        let base_aim_performance = (5.0 * (aim_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;
        let base_speed_performance =
            (5.0 * (speed_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let base_flashlight_performance = if mods.fl() {
            flashlight_rating.powi(2) * 25.0
        } else {
            0.0
        };

        let base_performance = ((base_aim_performance).powf(1.1)
            + (base_speed_performance).powf(1.1)
            + (base_flashlight_performance).powf(1.1))
        .powf(1.0 / 1.1);

        let star_rating = if base_performance > 0.00001 {
            PERFORMANCE_BASE_MULTIPLIER.cbrt()
                * 0.027
                * ((100_000.0 / 2.0_f64.powf(1.0 / 1.1) * base_performance).cbrt() + 4.0)
        } else {
            0.0
        };

        attrs.aim = aim_rating;
        attrs.speed = speed_rating;
        attrs.flashlight = flashlight_rating;
        attrs.slider_factor = slider_factor;
        attrs.stars = star_rating;
        attrs.speed_note_count = speed_relevant_note_count;
    }

    pub fn create_difficulty_objects<'a>(
        difficulty: &ModeDifficulty,
        scaling_factor: &ScalingFactor,
        osu_objects: impl Iterator<Item = Pin<&'a mut OsuObject>>,
    ) -> Vec<OsuDifficultyObject<'a>> {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut osu_objects_iter = osu_objects
            .map(|h| OsuDifficultyObject::compute_slider_cursor_pos(h, scaling_factor.radius))
            .map(Pin::into_ref);

        let Some(mut last) = osu_objects_iter.next().filter(|_| take > 0) else {
            return Vec::new();
        };

        let mut last_last = None;

        osu_objects_iter
            .enumerate()
            .map(|(idx, h)| {
                let diff_object = OsuDifficultyObject::new(
                    h.get_ref(),
                    last.get_ref(),
                    last_last.as_deref(),
                    clock_rate,
                    idx,
                    &scaling_factor,
                );

                last_last = Some(last);
                last = h;

                diff_object
            })
            .collect()
    }
}
