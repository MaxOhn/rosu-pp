use std::cmp;

use crate::{
    any::difficulty::{skills::Skill, ModeDifficulty},
    osu::{
        convert::convert_objects,
        difficulty::{object::OsuDifficultyObject, scaling_factor::ScalingFactor},
        object::OsuObject,
        performance::PERFORMANCE_BASE_MULTIPLIER,
    },
    util::mods::Mods,
};

use self::skills::{aim::Aim, flashlight::Flashlight, speed::Speed};

use super::{attributes::OsuDifficultyAttributes, convert::OsuBeatmap};

mod object;
pub mod scaling_factor;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.0675;

const HD_FADE_IN_DURATION_MULTIPLIER: f64 = 0.4;
const HD_FADE_OUT_DURATION_MULTIPLIER: f64 = 0.3;

pub fn difficulty(
    difficulty: &ModeDifficulty,
    converted: &OsuBeatmap<'_>,
) -> OsuDifficultyAttributes {
    let DifficultyValues {
        aim,
        aim_no_sliders,
        speed,
        flashlight,
        mut attrs,
    } = DifficultyValues::calculate(difficulty, converted);

    let mut aim_rating = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    let aim_rating_no_sliders = aim_no_sliders.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let speed_notes = speed.relevant_note_count();
    let mut speed_rating = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let mut flashlight_rating = flashlight.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let slider_factor = if aim_rating > 0.0 {
        aim_rating_no_sliders / aim_rating
    } else {
        1.0
    };

    let mods = difficulty.get_mods();

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
    let base_speed_performance = (5.0 * (speed_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

    let base_flashlight_performance = if mods.fl() {
        flashlight_rating * flashlight_rating * 25.0
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
    attrs.speed_note_count = speed_notes;

    attrs
}

pub struct DifficultyValues {
    pub aim: Aim,
    pub aim_no_sliders: Aim,
    pub speed: Speed,
    pub flashlight: Flashlight,
    pub attrs: OsuDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &ModeDifficulty, converted: &OsuBeatmap<'_>) -> Self {
        let mods = difficulty.get_mods();
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let map_attrs = converted
            .attributes()
            .mods(mods)
            .clock_rate(clock_rate)
            .build();

        let scaling_factor = ScalingFactor::new(map_attrs.cs);
        let hr = mods.hr();
        let hit_window = 2.0 * map_attrs.hit_windows.od;
        let time_preempt = f64::from((map_attrs.hit_windows.ar * clock_rate) as f32);

        // * Preempt time can go below 450ms. Normally, this is achieved via the DT mod
        // * which uniformly speeds up all animations game wide regardless of AR.
        // * This uniform speedup is hard to match 1:1, however we can at least make
        // * AR>10 (via mods) feel good by extending the upper linear function above.
        // * Note that this doesn't exactly match the AR>10 visuals as they're
        // * classically known, but it feels good.
        // * This adjustment is necessary for AR>10, otherwise TimePreempt can
        // * become smaller leading to hitcircles not fully fading in.
        let time_fade_in = if mods.hd() {
            time_preempt * HD_FADE_IN_DURATION_MULTIPLIER
        } else {
            400.0 * (time_preempt / OsuObject::PREEMPT_MIN).min(1.0)
        };

        let mut attrs = OsuDifficultyAttributes {
            ar: map_attrs.ar,
            hp: map_attrs.hp,
            od: map_attrs.od,
            ..Default::default()
        };

        let mut osu_objects = convert_objects(
            converted,
            &scaling_factor,
            hr,
            time_preempt,
            take,
            &mut attrs,
        );

        let mut osu_objects_iter = osu_objects
            .iter_mut()
            .map(|h| OsuDifficultyObject::compute_slider_cursor_pos(h, scaling_factor.radius));

        let aim = Aim::new(true);
        let aim_no_sliders = Aim::new(false);
        let speed = Speed::new(hit_window);
        let flashlight = Flashlight::new(mods, scaling_factor.radius, time_preempt, time_fade_in);

        let Some(mut last) = osu_objects_iter.next().filter(|_| take > 0) else {
            return Self {
                aim,
                aim_no_sliders,
                speed,
                flashlight,
                attrs,
            };
        };

        let mut last_last = None;

        let diff_objects: Vec<_> = osu_objects_iter
            .enumerate()
            .map(|(idx, h)| {
                let diff_object =
                    OsuDifficultyObject::new(h, last, last_last, clock_rate, idx, &scaling_factor);

                last_last = Some(last);
                last = h;

                diff_object
            })
            .collect();

        let mut aim = Skill::new(aim, &diff_objects);
        let mut aim_no_sliders = Skill::new(aim_no_sliders, &diff_objects);
        let mut speed = Skill::new(speed, &diff_objects);
        let mut flashlight = Skill::new(flashlight, &diff_objects);

        // The first hit object has no difficulty object
        let take_diff_objects = cmp::min(converted.map.hit_objects.len(), take) - 1;

        for hit_object in diff_objects.iter().take(take_diff_objects) {
            aim.process(hit_object);
            aim_no_sliders.process(hit_object);
            speed.process(hit_object);
            flashlight.process(hit_object);
        }

        Self {
            aim: aim.inner,
            aim_no_sliders: aim_no_sliders.inner,
            speed: speed.inner,
            flashlight: flashlight.inner,
            attrs,
        }
    }
}
