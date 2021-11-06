//! The positional offset of notes created by stack leniency is not considered.
//! This means the jump distance inbetween notes might be slightly off, resulting in small inaccuracies.
//! Since calculating these offsets is relatively expensive though,
//! this version is generally faster than `all_included`.

#![cfg(feature = "no_leniency")]

use std::mem;

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

use crate::{curve::CurveBuffers, Beatmap, Mods, Strains};

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
pub fn stars(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> DifficultyAttributes {
    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());

    let map_attributes = map.attributes().mods(mods);
    let hit_window = super::difficulty_range_od(map_attributes.od) / map_attributes.clock_rate;
    let od = (80.0 - hit_window) / 6.0;

    let mut diff_attributes = DifficultyAttributes {
        ar: map_attributes.ar,
        hp: map_attributes.hp,
        od,
        ..Default::default()
    };

    if take < 2 {
        return diff_attributes;
    }

    let radius = OBJECT_RADIUS * (1.0 - 0.7 * (map_attributes.cs - 5.0) / 5.0) / 2.0;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut slider_state = SliderState::new(map);
    let mut curve_bufs = CurveBuffers::default();
    let mut ticks_buf = Vec::new();

    let mut hit_objects = map
        .hit_objects
        .iter()
        .take(take)
        .filter_map(|h| {
            OsuObject::new(
                h,
                map,
                radius,
                scaling_factor,
                &mut ticks_buf,
                &mut diff_attributes,
                &mut slider_state,
                &mut curve_bufs,
            )
        })
        .map(|mut h| {
            h.time /= map_attributes.clock_rate;

            h
        });

    let fl = mods.fl();
    let mut skills = Vec::with_capacity(2 + fl as usize);

    skills.push(Skill::new(SkillKind::Aim));
    skills.push(Skill::new(SkillKind::speed(hit_window)));

    if fl {
        skills.push(Skill::new(SkillKind::flashlight(scaling_factor)));
    };

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end = (prev.time / SECTION_LEN).ceil() * SECTION_LEN;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

    while h.base.time > current_section_end {
        for skill in skills.iter_mut() {
            skill.start_new_section_from(current_section_end);
        }

        current_section_end += SECTION_LEN;
    }

    for skill in skills.iter_mut() {
        skill.process(&h);
    }

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

        while h.base.time > current_section_end {
            for skill in skills.iter_mut() {
                skill.save_current_peak();
                skill.start_new_section_from(current_section_end);
            }

            current_section_end += SECTION_LEN;
        }

        for skill in skills.iter_mut() {
            skill.process(&h);
        }

        prev_prev = Some(prev);
        prev_vals = Some((h.jump_dist, h.strain_time));
        prev = curr;
    }

    for skill in skills.iter_mut() {
        skill.save_current_peak();
    }

    let aim_rating = skills[0].difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let speed_rating = if mods.rx() {
        0.0
    } else {
        skills[1].difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER
    };

    let flashlight_rating = skills.get_mut(2).map_or(0.0, |skill| {
        skill.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER
    });

    let base_aim_performance = {
        let base = 5.0 * (aim_rating / 0.0675).max(1.0) - 4.0;

        base * base * base / 100_000.0
    };

    let base_speed_performance = {
        let base = 5.0 * (speed_rating / 0.0675).max(1.0) - 4.0;

        base * base * base / 100_000.0
    };

    let base_flashlight_performance = if fl {
        flashlight_rating * flashlight_rating * 25.0
    } else {
        0.0
    };

    let base_performance = (base_aim_performance.powf(1.1)
        + base_speed_performance.powf(1.1)
        + base_flashlight_performance.powf(1.1))
    .powf(1.0 / 1.1);

    let star_rating = if base_performance > 0.00001 {
        1.12_f32.cbrt()
            * 0.027
            * ((100_000.0 / (1.0_f32 / 1.1).exp2() * base_performance).cbrt() + 4.0)
    } else {
        0.0
    };

    diff_attributes.aim_strain = aim_rating;
    diff_attributes.speed_strain = speed_rating;
    diff_attributes.flashlight_rating = flashlight_rating;
    diff_attributes.n_circles = map.n_circles as usize;
    diff_attributes.n_sliders = map.n_sliders as usize;
    diff_attributes.n_spinners = map.n_spinners as usize;
    diff_attributes.stars = star_rating;

    diff_attributes
}

/// Essentially the same as the `stars` function but instead of
/// evaluating the final strains, it just returns them as is.
///
/// Suitable to plot the difficulty of a map over time.
pub fn strains(map: &Beatmap, mods: impl Mods) -> Strains {
    let map_attributes = map.attributes().mods(mods);
    let hit_window =
        super::difficulty_range_od(map_attributes.od).floor() / map_attributes.clock_rate;
    let od = (80.0 - hit_window) / 6.0;

    let mut diff_attributes = DifficultyAttributes {
        ar: map_attributes.ar,
        hp: map_attributes.hp,
        od,
        ..Default::default()
    };

    if map.hit_objects.len() < 2 {
        return Strains::default();
    }

    let radius = OBJECT_RADIUS * (1.0 - 0.7 * (map_attributes.cs - 5.0) / 5.0) / 2.0;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut slider_state = SliderState::new(map);
    let mut ticks_buf = Vec::new();
    let mut curve_bufs = CurveBuffers::default();

    let mut hit_objects = map.hit_objects.iter().filter_map(|h| {
        OsuObject::new(
            h,
            map,
            radius,
            scaling_factor,
            &mut ticks_buf,
            &mut diff_attributes,
            &mut slider_state,
            &mut curve_bufs,
        )
    });

    let fl = mods.fl();
    let mut skills = Vec::with_capacity(2 + fl as usize);

    skills.push(Skill::new(SkillKind::Aim));
    skills.push(Skill::new(SkillKind::speed(hit_window)));

    if fl {
        skills.push(Skill::new(SkillKind::flashlight(scaling_factor)));
    };

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end = (prev.time / SECTION_LEN).ceil() * SECTION_LEN;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

    while h.base.time > current_section_end {
        for skill in skills.iter_mut() {
            skill.start_new_section_from(current_section_end);
        }

        current_section_end += SECTION_LEN;
    }

    for skill in skills.iter_mut() {
        skill.process(&h);
    }

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

        while h.base.time > current_section_end {
            for skill in skills.iter_mut() {
                skill.save_current_peak();
                skill.start_new_section_from(current_section_end);
            }

            current_section_end += SECTION_LEN;
        }

        prev_prev = Some(prev);
        prev_vals = Some((h.jump_dist, h.strain_time));
        prev = curr;
    }

    for skill in skills.iter_mut() {
        skill.save_current_peak();
    }

    let mut speed_strains = skills.pop().unwrap().strain_peaks;
    let mut aim_strains = skills.pop().unwrap().strain_peaks;

    let strains = if let Some(mut flashlight_strains) = skills.pop().map(|s| s.strain_peaks) {
        mem::swap(&mut speed_strains, &mut aim_strains);
        mem::swap(&mut aim_strains, &mut flashlight_strains);

        aim_strains
            .into_iter()
            .zip(speed_strains)
            .zip(flashlight_strains)
            .map(|((aim, speed), flashlight)| aim + speed + flashlight)
            .collect()
    } else {
        aim_strains
            .into_iter()
            .zip(speed_strains)
            .map(|(aim, speed)| aim + speed)
            .collect()
    };

    Strains {
        section_length: SECTION_LEN,
        strains,
    }
}

#[test]
fn custom_osu() {
    use std::{fs::File, time::Instant};

    let path = "./maps/2753127.osu";
    // let path = "E:Games/osu!/beatmaps/2571051.osu";
    let file = File::open(path).unwrap();

    let start = Instant::now();
    let map = Beatmap::parse(file).unwrap();

    let iters = 100;
    let accum = start.elapsed();

    // * Tiny benchmark for map parsing
    let mut accum = accum;

    for _ in 0..iters {
        let file = File::open(path).unwrap();
        let start = Instant::now();
        let _map = Beatmap::parse(file).unwrap();
        accum += start.elapsed();
    }

    println!("Parsing average: {:?}", accum / iters);

    let start = Instant::now();
    let result = crate::OsuPP::new(&map).mods(0).calculate();

    let iters = 100;
    let accum = start.elapsed();

    // * Tiny benchmark for pp calculation
    // let mut accum = accum;

    // for _ in 0..iters {
    //     let start = Instant::now();
    //     let _result = crate::OsuPP::new(&map).mods(0).calculate();
    //     accum += start.elapsed();
    // }

    println!("{:#?}", result);
    println!("Calculation average: {:?}", accum / iters);
}
