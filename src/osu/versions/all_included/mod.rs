//! Every aspect of osu!'s pp calculation is being used.
//! This should result in the most accurate values but with
//! drawback of being slower than the other versions.

#![cfg(feature = "all_included")]

use super::super::DifficultyAttributes;
use crate::parse::Pos2;

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
const STACK_DISTANCE: f32 = 3.0;

/// Star calculation for osu!standard maps.
///
/// Both slider paths and stack leniency are considered.
/// Since taking stack leniency into account is fairly expensive,
/// this version is slower than the others but in turn gives the
/// most precise results.
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

    let mut raw_ar = map.ar;
    let hr = mods.hr();

    if hr {
        raw_ar = (raw_ar * 1.4).min(10.0);
    } else if mods.ez() {
        raw_ar *= 0.5;
    }

    let time_preempt = difficulty_range_ar(raw_ar);
    let scale = (1.0 - 0.7 * (map_attributes.cs - 5.0) / 5.0) / 2.0;
    let radius = OBJECT_RADIUS * scale;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut slider_state = SliderState::new(map);
    let mut ticks_buf = Vec::new();

    let hit_objects_iter = map.hit_objects.iter().take(take).filter_map(|h| {
        OsuObject::new(
            h,
            map,
            radius,
            scaling_factor,
            hr,
            &mut ticks_buf,
            &mut diff_attributes,
            &mut slider_state,
        )
    });

    let mut hit_objects = Vec::with_capacity(take);
    hit_objects.extend(hit_objects_iter);

    let stack_threshold = time_preempt * map.stack_leniency;

    if map.version >= 6 {
        stacking(&mut hit_objects, stack_threshold);
    } else {
        old_stacking(&mut hit_objects, stack_threshold);
    }

    let scale_factor = scale * -6.4;

    let mut hit_objects = hit_objects.into_iter().map(|mut h| {
        let stack_offset = h.stack_height * scale_factor;

        h.time /= map_attributes.clock_rate;
        h.pos += Pos2 {
            x: stack_offset,
            y: stack_offset,
        };

        h
    });

    let mut aim = Skill::new(SkillKind::Aim);
    let mut speed = Skill::new(SkillKind::Speed);

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end = (prev.time / SECTION_LEN).ceil() * SECTION_LEN;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

    while h.base.time > current_section_end {
        current_section_end += SECTION_LEN;
    }

    aim.process(&h);
    speed.process(&h);

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

        while h.base.time > current_section_end {
            aim.save_current_peak();
            aim.start_new_section_from(current_section_end);
            speed.save_current_peak();
            speed.start_new_section_from(current_section_end);

            current_section_end += SECTION_LEN;
        }

        aim.process(&h);
        speed.process(&h);

        prev_prev = Some(prev);
        prev_vals = Some((h.jump_dist, h.strain_time));
        prev = curr;
    }

    aim.save_current_peak();
    speed.save_current_peak();

    let aim_rating = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    let speed_rating = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let stars = aim_rating + speed_rating + (aim_rating - speed_rating).abs() / 2.0;

    diff_attributes.n_circles = map.n_circles as usize;
    diff_attributes.n_spinners = map.n_spinners as usize;
    diff_attributes.stars = stars;
    diff_attributes.speed_strain = speed_rating;
    diff_attributes.aim_strain = aim_rating;

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

    let mut raw_ar = map.ar;
    let hr = mods.hr();

    if hr {
        raw_ar *= 1.4;
    } else if mods.ez() {
        raw_ar *= 0.5;
    }

    let time_preempt = difficulty_range_ar(raw_ar);
    let scale = (1.0 - 0.7 * (map_attributes.cs - 5.0) / 5.0) / 2.0;
    let radius = OBJECT_RADIUS * scale;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut slider_state = SliderState::new(map);
    let mut ticks_buf = Vec::new();

    let hit_objects_iter = map.hit_objects.iter().filter_map(|h| {
        OsuObject::new(
            h,
            map,
            radius,
            scaling_factor,
            hr,
            &mut ticks_buf,
            &mut diff_attributes,
            &mut slider_state,
        )
    });

    let mut hit_objects = Vec::with_capacity(map.hit_objects.len());
    hit_objects.extend(hit_objects_iter);

    let stack_threshold = time_preempt * map.stack_leniency;

    if map.version >= 6 {
        stacking(&mut hit_objects, stack_threshold);
    } else {
        old_stacking(&mut hit_objects, stack_threshold);
    }

    let scale_factor = scale * -6.4;

    let mut hit_objects = hit_objects.into_iter().map(|mut h| {
        let stack_offset = h.stack_height * scale_factor;

        h.time /= map_attributes.clock_rate;
        h.pos += Pos2 {
            x: stack_offset,
            y: stack_offset,
        };

        h
    });

    let mut aim = Skill::new(SkillKind::Aim);
    let mut speed = Skill::new(SkillKind::Speed);

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end = (prev.time / SECTION_LEN).ceil() * SECTION_LEN;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

    while h.base.time > current_section_end {
        current_section_end += SECTION_LEN;
    }

    aim.process(&h);
    speed.process(&h);

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(&curr, &prev, prev_vals, prev_prev, scaling_factor);

        while h.base.time > current_section_end {
            aim.save_current_peak();
            aim.start_new_section_from(current_section_end);
            speed.save_current_peak();
            speed.start_new_section_from(current_section_end);

            current_section_end += SECTION_LEN;
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
        section_length: SECTION_LEN,
        strains,
    }
}

fn stacking(hit_objects: &mut [OsuObject], stack_threshold: f32) {
    let mut extended_start_idx = 0;
    let extended_end_idx = hit_objects.len() - 1;

    for mut i in (1..=extended_end_idx).rev() {
        let mut n = i;

        if hit_objects[i].stack_height.abs() > 0.0 || hit_objects[i].is_spinner() {
            continue;
        }

        if hit_objects[i].is_circle() {
            loop {
                n = match n.checked_sub(1) {
                    Some(n) => n,
                    None => break,
                };

                if hit_objects[n].is_spinner() {
                    continue;
                } else if hit_objects[i].time - hit_objects[n].end_time() > stack_threshold {
                    break;
                } else if n < extended_start_idx {
                    hit_objects[n].stack_height = 0.0;
                    extended_start_idx = n;
                }

                if hit_objects[n].is_slider()
                    && hit_objects[n].end_pos().distance(hit_objects[i].pos) < STACK_DISTANCE
                {
                    let offset = hit_objects[i].stack_height - hit_objects[n].stack_height + 1.0;

                    for j in n + 1..=i {
                        if hit_objects[n].pos.distance(hit_objects[j].pos) < STACK_DISTANCE {
                            hit_objects[j].stack_height -= offset;
                        }
                    }

                    break;
                } else if hit_objects[n].pos.distance(hit_objects[i].pos) < STACK_DISTANCE {
                    hit_objects[n].stack_height = hit_objects[i].stack_height + 1.0;
                    i = n;
                }
            }
        } else if hit_objects[i].is_slider() {
            loop {
                n = match n.checked_sub(1) {
                    Some(n) => n,
                    None => break,
                };

                if hit_objects[n].is_spinner() {
                    continue;
                } else if hit_objects[i].time - hit_objects[n].time > stack_threshold {
                    break;
                } else if hit_objects[n].end_pos().distance(hit_objects[i].pos) < STACK_DISTANCE {
                    hit_objects[n].stack_height = hit_objects[i].stack_height + 1.0;
                    i = n;
                }
            }
        }
    }
}

fn old_stacking(hit_objects: &mut [OsuObject], stack_threshold: f32) {
    for i in 0..hit_objects.len() {
        if hit_objects[i].stack_height != 0.0 && !hit_objects[i].is_slider() {
            continue;
        }

        let mut start_time = hit_objects[i].end_time();
        let end_pos = hit_objects[i].end_pos();

        let mut slider_stack = 0.0;

        for j in i + 1..hit_objects.len() {
            if hit_objects[j].time - stack_threshold > start_time {
                break;
            }

            if hit_objects[j].pos.distance(hit_objects[i].pos) < STACK_DISTANCE {
                hit_objects[i].stack_height += 1.0;
                start_time = hit_objects[j].end_time();
            } else if hit_objects[j].pos.distance(end_pos) < STACK_DISTANCE {
                slider_stack += 1.0;
                hit_objects[j].stack_height -= slider_stack;
                start_time = hit_objects[j].end_time();
            }
        }
    }
}

const OSU_AR_MAX: f32 = 450.0;
const OSU_AR_AVG: f32 = 1200.0;
const OSU_AR_MIN: f32 = 1800.0;

#[inline]
fn difficulty_range_ar(ar: f32) -> f32 {
    crate::difficulty_range(ar, OSU_AR_MAX, OSU_AR_AVG, OSU_AR_MIN)
}
