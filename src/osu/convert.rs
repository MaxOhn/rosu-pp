use rosu_map::section::hit_objects::CurveBuffers;

use crate::model::{beatmap::Beatmap, mods::Reflection};

use super::{
    attributes::OsuDifficultyAttributes,
    difficulty::scaling_factor::ScalingFactor,
    object::{NestedSliderObjectKind, OsuObject, OsuObjectKind},
};

pub fn convert_objects(
    map: &Beatmap,
    scaling_factor: &ScalingFactor,
    reflection: Reflection,
    time_preempt: f64,
    mut take: usize,
    attrs: &mut OsuDifficultyAttributes,
) -> Box<[OsuObject]> {
    let mut curve_bufs = CurveBuffers::default();
    // mean=5.16 | median=4
    let mut ticks_buf = Vec::new();

    let mut osu_objects: Box<[_]> = map
        .hit_objects
        .iter()
        .map(|h| OsuObject::new(h, map, &mut curve_bufs, &mut ticks_buf))
        .inspect(|h| {
            if take == 0 {
                return;
            }

            take -= 1;
            attrs.max_combo += 1;

            match h.kind {
                OsuObjectKind::Circle => attrs.n_circles += 1,
                OsuObjectKind::Slider(ref slider) => {
                    attrs.n_sliders += 1;
                    attrs.n_large_ticks += slider.large_tick_count() as u32;
                    attrs.max_combo += slider.nested_objects.len() as u32;
                }
                OsuObjectKind::Spinner(_) => attrs.n_spinners += 1,
            }
        })
        .collect();

    match reflection {
        Reflection::None => osu_objects.iter_mut().for_each(OsuObject::finalize_nested),
        Reflection::Vertical => osu_objects
            .iter_mut()
            .for_each(OsuObject::reflect_vertically),
        Reflection::Horizontal => osu_objects
            .iter_mut()
            .for_each(OsuObject::reflect_horizontally),
        Reflection::Both => osu_objects
            .iter_mut()
            .for_each(OsuObject::reflect_both_axes),
    }

    let stack_threshold = time_preempt * f64::from(map.stack_leniency);

    if map.version >= 6 {
        stacking(&mut osu_objects, stack_threshold);
    } else {
        old_stacking(&mut osu_objects, stack_threshold);
    }

    for h in osu_objects.iter_mut() {
        h.stack_offset = scaling_factor.stack_offset(h.stack_height);
    }

    osu_objects
}

const STACK_DISTANCE: f32 = 3.0;

fn stacking(hit_objects: &mut [OsuObject], stack_threshold: f64) {
    let mut extended_start_idx = 0;

    let Some(extended_end_idx) = hit_objects.len().checked_sub(1) else {
        return;
    };

    // First big `if` in osu!lazer's function can be skipped

    for i in (1..=extended_end_idx).rev() {
        let mut n = i;
        let mut obj_i_idx = i;
        // * We should check every note which has not yet got a stack.
        // * Consider the case we have two interwound stacks and this will make sense.
        // *   o <-1      o <-2
        // *    o <-3      o <-4
        // * We first process starting from 4 and handle 2,
        // * then we come backwards on the i loop iteration until we reach 3 and handle 1.
        // * 2 and 1 will be ignored in the i loop because they already have a stack value.

        if hit_objects[obj_i_idx].stack_height != 0 || hit_objects[obj_i_idx].is_spinner() {
            continue;
        }

        // * If this object is a hitcircle, then we enter this "special" case.
        // * It either ends with a stack of hitcircles only,
        // * or a stack of hitcircles that are underneath a slider.
        // * Any other case is handled by the "is_slider" code below this.
        if hit_objects[obj_i_idx].is_circle() {
            loop {
                n = match n.checked_sub(1) {
                    Some(n) => n,
                    None => break,
                };

                if hit_objects[n].is_spinner() {
                    continue;
                }

                if hit_objects[obj_i_idx].start_time - hit_objects[n].end_time() > stack_threshold {
                    break; // * We are no longer within stacking range of the previous object.
                }

                // * HitObjects before the specified update range haven't been reset yet
                if n < extended_start_idx {
                    hit_objects[n].stack_height = 0;
                    extended_start_idx = n;
                }

                // * This is a special case where hticircles are moved DOWN and RIGHT (negative stacking)
                // * if they are under the *last* slider in a stacked pattern.
                // *    o==o <- slider is at original location
                // *        o <- hitCircle has stack of -1
                // *         o <- hitCircle has stack of -2
                if hit_objects[n].is_slider()
                    && hit_objects[n]
                        .end_pos()
                        .distance(hit_objects[obj_i_idx].pos)
                        < STACK_DISTANCE
                {
                    let offset =
                        hit_objects[obj_i_idx].stack_height - hit_objects[n].stack_height + 1;

                    for j in n + 1..=i {
                        // * For each object which was declared under this slider, we will offset
                        // * it to appear *below* the slider end (rather than above).
                        if hit_objects[n].end_pos().distance(hit_objects[j].pos) < STACK_DISTANCE {
                            hit_objects[j].stack_height -= offset;
                        }
                    }

                    // * We have hit a slider. We should restart calculation using this as the new base.
                    // * Breaking here will mean that the slider still has StackCount of 0,
                    // * so will be handled in the i-outer-loop.
                    break;
                }

                if hit_objects[n].pos.distance(hit_objects[obj_i_idx].pos) < STACK_DISTANCE {
                    // * Keep processing as if there are no sliders.
                    // * If we come across a slider, this gets cancelled out.
                    // * NOTE: Sliders with start positions stacking
                    // * are a special case that is also handled here.

                    hit_objects[n].stack_height = hit_objects[obj_i_idx].stack_height + 1;
                    obj_i_idx = n;
                }
            }
        } else if hit_objects[obj_i_idx].is_slider() {
            // * We have hit the first slider in a possible stack.
            // * From this point on, we ALWAYS stack positive regardless.
            loop {
                n = match n.checked_sub(1) {
                    Some(n) => n,
                    None => break,
                };

                if hit_objects[n].is_spinner() {
                    continue;
                }

                if hit_objects[obj_i_idx].start_time - hit_objects[n].start_time > stack_threshold {
                    break; // * We are no longer within stacking range of the previous object.
                }

                if hit_objects[n]
                    .end_pos()
                    .distance(hit_objects[obj_i_idx].pos)
                    < STACK_DISTANCE
                {
                    hit_objects[n].stack_height = hit_objects[obj_i_idx].stack_height + 1;
                    obj_i_idx = n;
                }
            }
        }
    }
}

fn old_stacking(hit_objects: &mut [OsuObject], stack_threshold: f64) {
    for i in 0..hit_objects.len() {
        if hit_objects[i].stack_height != 0 && !hit_objects[i].is_slider() {
            continue;
        }

        let mut start_time = hit_objects[i].end_time();

        let pos2 = {
            let h = &hit_objects[i];

            match h.kind {
                OsuObjectKind::Circle | OsuObjectKind::Spinner(_) => h.pos,
                OsuObjectKind::Slider(ref slider) => {
                    // We need the path endpos instead of the slider endpos
                    let repeat_count = slider.repeat_count();

                    let nested = if repeat_count % 2 == 0 {
                        slider.tail()
                    } else {
                        slider
                            .nested_objects
                            .iter()
                            .find(|nested| matches!(nested.kind, NestedSliderObjectKind::Repeat))
                    };

                    nested.map_or(h.pos, |nested| nested.pos)
                }
            }
        };

        let mut slider_stack = 0;

        for j in i + 1..hit_objects.len() {
            if hit_objects[j].start_time - stack_threshold > start_time {
                break;
            }

            // * Note the use of `StartTime` in the code below doesn't match stable's use of `EndTime`.
            // * This is because in the stable implementation, `UpdateCalculations` is not called on the inner-loop hitobject (j)
            // * and therefore it does not have a correct `EndTime`, but instead the default of `EndTime = StartTime`.
            // *
            // * Effects of this can be seen on https://osu.ppy.sh/beatmapsets/243#osu/1146 at sliders around 86647 ms, where
            // * if we use `EndTime` here it would result in unexpected stacking.

            if hit_objects[j].pos.distance(hit_objects[i].pos) < STACK_DISTANCE {
                hit_objects[i].stack_height += 1;
                start_time = hit_objects[j].start_time;
            } else if hit_objects[j].pos.distance(pos2) < STACK_DISTANCE {
                slider_stack += 1;
                hit_objects[j].stack_height -= slider_stack;
                start_time = hit_objects[j].start_time;
            }
        }
    }
}
