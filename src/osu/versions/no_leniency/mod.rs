//! The positional offset of notes created by stack leniency is not considered.
//! This means the jump distance inbetween notes might be slightly off, resulting in small inaccuracies.
//! Since calculating these offsets is relatively expensive though, this version is faster than `all_included`.

#![cfg(feature = "no_leniency")]

use super::super::DifficultyAttributes;

mod difficulty_object;
mod osu_object;
mod skill;
mod skill_kind;
mod slider_state;

use difficulty_object::DifficultyObject;
use osu_object::OsuObject;
use skill::Skill;
use skill_kind::SkillKind;
use slider_state::SliderState;

use crate::{Beatmap, Mods, StarResult, Strains};

const OBJECT_RADIUS: f32 = 64.0;
const SECTION_LEN: f32 = 400.0;
const DIFFICULTY_MULTIPLIER: f32 = 0.0675;
const NORMALIZED_RADIUS: f32 = 52.0;

/// Star calculation for osu!standard maps.
///
/// Slider paths are considered but stack leniency is ignored.
/// As most maps don't even make use of leniency and even if,
/// it has generally little effect on stars, the results are close to perfect.
/// This version is considerably more efficient than `all_included` since
/// processing stack leniency is relatively expensive.
///
/// In case of a partial play, e.g. a fail, one can specify the amount of passed objects.
pub fn stars(map: &Beatmap, mods: impl Mods, passed_objects: Option<usize>) -> StarResult {
    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());

    let map_attributes = map.attributes().mods(mods);
    let hitwindow = super::difficulty_range(map_attributes.od).floor() / map_attributes.clock_rate;
    let od = (80.0 - hitwindow) / 6.0;

    let mut diff_attributes = DifficultyAttributes {
        ar: map_attributes.ar,
        od,
        ..Default::default()
    };

    if take < 2 {
        return StarResult::Osu(diff_attributes);
    }

    let section_len = SECTION_LEN * map_attributes.clock_rate;
    let radius = OBJECT_RADIUS * (1.0 - 0.7 * (map_attributes.cs - 5.0) / 5.0) / 2.0;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut slider_state = SliderState::new(map);
    let mut ticks_buf = Vec::new();

    let mut hit_objects = map.hit_objects.iter().take(take).filter_map(|h| {
        OsuObject::new(
            h,
            map,
            radius,
            scaling_factor,
            &mut ticks_buf,
            &mut diff_attributes,
            &mut slider_state,
        )
    });

    let mut aim = Skill::new(SkillKind::Aim);
    let mut speed = Skill::new(SkillKind::Speed);

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(
        &curr,
        &prev,
        prev_vals,
        prev_prev,
        map_attributes.clock_rate,
        scaling_factor,
    );

    while h.base.time > current_section_end {
        current_section_end += section_len;
    }

    aim.process(&h);
    speed.process(&h);

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(
            &curr,
            &prev,
            prev_vals,
            prev_prev,
            map_attributes.clock_rate,
            scaling_factor,
        );

        while h.base.time > current_section_end {
            aim.save_current_peak();
            aim.start_new_section_from(current_section_end);
            speed.save_current_peak();
            speed.start_new_section_from(current_section_end);

            current_section_end += section_len;
        }

        aim.process(&h);
        speed.process(&h);

        prev_prev = Some(prev);
        prev_vals = Some((h.jump_dist, h.strain_time));
        prev = curr;
    }

    aim.save_current_peak();
    speed.save_current_peak();

    let aim_strain = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    let speed_strain = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let stars = aim_strain + speed_strain + (aim_strain - speed_strain).abs() / 2.0;

    diff_attributes.stars = stars;
    diff_attributes.speed_strain = speed_strain;
    diff_attributes.aim_strain = aim_strain;

    StarResult::Osu(diff_attributes)
}

/// Essentially the same as the `stars` function but instead of
/// evaluating the final strains, it just returns them as is.
///
/// Suitable to plot the difficulty of a map over time.
pub fn strains(map: &Beatmap, mods: impl Mods) -> Strains {
    let map_attributes = map.attributes().mods(mods);
    let hitwindow = super::difficulty_range(map_attributes.od).floor() / map_attributes.clock_rate;
    let od = (80.0 - hitwindow) / 6.0;

    let mut diff_attributes = DifficultyAttributes {
        ar: map_attributes.ar,
        od,
        ..Default::default()
    };

    if map.hit_objects.len() < 2 {
        return Strains::default();
    }

    let section_len = SECTION_LEN * map_attributes.clock_rate;
    let radius = OBJECT_RADIUS * (1.0 - 0.7 * (map_attributes.cs - 5.0) / 5.0) / 2.0;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut slider_state = SliderState::new(map);
    let mut ticks_buf = Vec::new();

    let mut hit_objects = map.hit_objects.iter().filter_map(|h| {
        OsuObject::new(
            h,
            map,
            radius,
            scaling_factor,
            &mut ticks_buf,
            &mut diff_attributes,
            &mut slider_state,
        )
    });

    let mut aim = Skill::new(SkillKind::Aim);
    let mut speed = Skill::new(SkillKind::Speed);

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(
        &curr,
        &prev,
        prev_vals,
        prev_prev,
        map_attributes.clock_rate,
        scaling_factor,
    );

    while h.base.time > current_section_end {
        current_section_end += section_len;
    }

    aim.process(&h);
    speed.process(&h);

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(
            &curr,
            &prev,
            prev_vals,
            prev_prev,
            map_attributes.clock_rate,
            scaling_factor,
        );

        while h.base.time > current_section_end {
            aim.save_current_peak();
            aim.start_new_section_from(current_section_end);
            speed.save_current_peak();
            speed.start_new_section_from(current_section_end);

            current_section_end += section_len;
        }

        aim.process(&h);
        speed.process(&h);

        prev_prev = Some(prev);
        prev_vals = Some((h.jump_dist, h.strain_time));
        prev = curr;
    }

    aim.save_current_peak();
    speed.save_current_peak();

    let strains = aim
        .strain_peaks
        .into_iter()
        .zip(speed.strain_peaks.into_iter())
        .map(|(aim, speed)| aim + speed)
        .collect();

    Strains {
        section_length: section_len,
        strains,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::OsuPP;
    use crate::Beatmap;
    use std::fs::File;

    #[test]
    #[ignore]
    fn no_leniency_single() {
        let file = match File::open("./maps/1791963.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let result = OsuPP::new(&map)
            .n300(1206)
            .n100(15)
            .n50(0)
            .combo(1643)
            .mods(24)
            .calculate();

        println!("Stars: {}", result.stars());
        println!("PP: {}", result.pp());
    }
}
