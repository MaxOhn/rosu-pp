#![allow(unused)]

use crate::{curve::Curve, Beatmap, BeatmapAttributes, HitObject, HitObjectKind, PathType, Pos2};

use std::cmp::Ordering;

macro_rules! binary_search {
    ($slice:expr, $target:expr) => {
        $slice.binary_search_by(|p| p.time.partial_cmp(&$target).unwrap_or(Ordering::Equal))
    };
}

const OBJECT_RADIUS: f32 = 64.0;
const STACK_DIST: f32 = 3.0;
const LEGACY_LAST_TICK_OFFSET: f32 = 36.0;

#[derive(Clone)] // TODO: Remove clone
pub(crate) enum OsuObject {
    Circle {
        pos: Pos2,
        time: f32,

        scale: f32,
        stack_height: f32,
    },
    Slider {
        pixel_len: f32, // TODO: Redundant?
        repeats: usize, // TODO: Redundant?

        objects: Vec<SliderTick>,

        scale: f32,
        stack_height: f32,

        cursor_end_pos: Pos2,
        cursor_travel_dist: f32,
    },
    Spinner {
        pos: Pos2,
        time: f32,
    },
}

impl OsuObject {
    pub(crate) fn new(h: &HitObject, map: &Beatmap, attributes: &BeatmapAttributes) -> Self {
        let pos = h.pos;
        let time = h.start_time;

        let scale = (1.0 - 0.7 * (attributes.cs - 5.0) / 5.0) / 2.0;
        let mut stack_height = 0.0;

        match &h.kind {
            HitObjectKind::Circle => Self::Circle {
                pos,
                time,
                scale,
                stack_height,
            },
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                curve_points,
                path_type,
            } => {
                let (beat_len, timing_time) = {
                    match binary_search!(map.timing_points, time) {
                        Ok(idx) => {
                            let point = &map.timing_points[idx];
                            (point.beat_len, point.time)
                        }
                        Err(0) => (1000.0, 0.0),
                        Err(idx) => {
                            let point = &map.timing_points[idx - 1];
                            (point.beat_len, point.time)
                        }
                    }
                };

                let (speed_multiplier, diff_time) = {
                    match binary_search!(map.difficulty_points, time) {
                        Ok(idx) => {
                            let point = &map.difficulty_points[idx];
                            (point.speed_multiplier, point.time)
                        }
                        Err(0) => (1.0, 0.0),
                        Err(idx) => {
                            let point = &map.difficulty_points[idx - 1];
                            (point.speed_multiplier, point.time)
                        }
                    }
                };

                let mut tick_distance = 100.0 * map.sv / map.tick_rate;

                if map.version >= 8 {
                    tick_distance /= (100.0 / speed_multiplier).max(10.0).min(1000.0) / 100.0;
                }

                let spm = if timing_time > diff_time {
                    1.0
                } else {
                    speed_multiplier
                };

                let duration = *repeats as f32 * beat_len * pixel_len / (map.sv * spm) / 100.0;

                // let velocity = *pixel_len as f32 / duration;

                // println!("duration={}", duration);
                // println!("velocity={}", velocity);

                let path_type = if *path_type == PathType::PerfectCurve && curve_points.len() > 3 {
                    PathType::Bezier
                } else if curve_points.len() == 2 {
                    PathType::Linear
                } else {
                    *path_type
                };

                let curve = match path_type {
                    PathType::Linear => Curve::linear(curve_points[0], curve_points[1]),
                    PathType::Bezier => Curve::bezier(&curve_points),
                    PathType::Catmull => Curve::catmull(&curve_points),
                    PathType::PerfectCurve => Curve::perfect(&curve_points),
                };

                let mut current_distance = tick_distance;
                let time_add = duration * (tick_distance / (pixel_len * *repeats as f32));

                let target = pixel_len - tick_distance / 8.0;
                let mut ticks = Vec::with_capacity((target / tick_distance) as usize);

                while current_distance < target {
                    let pos = curve.point_at_distance(current_distance);
                    let time = h.start_time + time_add * (ticks.len() + 1) as f32;
                    ticks.push(SliderTick::new(pos, time));
                    current_distance += tick_distance;
                }

                let mut slider_objects = Vec::with_capacity(repeats * (ticks.len() + 1));
                slider_objects.push(SliderTick::new(h.pos, h.start_time));

                if *repeats <= 1 {
                    slider_objects.append(&mut ticks);
                } else {
                    slider_objects.append(&mut ticks.clone());

                    for repeat_id in 1..repeats - 1 {
                        let dist = (repeat_id % 2) as f32 * pixel_len;
                        let time_offset = (duration / *repeats as f32) * repeat_id as f32;
                        let pos = curve.point_at_distance(dist);

                        // Reverse tick / last legacy tick
                        slider_objects.push(SliderTick::new(pos, h.start_time + time_offset));

                        ticks.reverse();
                        slider_objects.extend_from_slice(&ticks); // tick time doesn't need to be adjusted for some reason
                    }

                    // Handling last span separatly so that `ticks` vector isn't cloned again
                    let dist = ((repeats - 1) % 2) as f32 * pixel_len;
                    let time_offset = (duration / *repeats as f32) * (repeats - 1) as f32;
                    let pos = curve.point_at_distance(dist);

                    slider_objects.push(SliderTick::new(pos, h.start_time + time_offset));

                    ticks.reverse();
                    slider_objects.append(&mut ticks);
                }

                // Slider tail
                let span_duration = duration / *repeats as f32;
                let final_span_idx = repeats.saturating_sub(1);
                let final_span_start_time = h.start_time + final_span_idx as f32 * span_duration;
                let final_span_end_time = (h.start_time + duration / 2.0)
                    .max(final_span_start_time + span_duration - LEGACY_LAST_TICK_OFFSET);
                let mut final_progress =
                    (final_span_end_time - final_span_start_time) / span_duration;

                if *repeats & 1 == 0 {
                    final_progress = 1.0 - final_progress;
                }

                // println!(
                //     "final_span_index={} | final_span_start_time={} | \
                //     final_span_end_time={} | final_progress={}",
                //     final_span_idx, final_span_start_time, final_span_end_time, final_progress
                // );

                // println!("len={}", final_progress * *pixel_len as f32);

                let dist_end = (repeats % 2) as f32 * pixel_len;

                let pos = curve.point_at_distance(dist_end);
                slider_objects.push(SliderTick::new(pos, final_span_end_time));

                // println!(
                //     "start_time={} | span_duration={} | vel={} | \
                //     tick_dist={} | dist={} | span_count={} | \
                //     legacy_last_tick_offset={}",
                //     h.start_time,
                //     duration / *repeats as f32,
                //     *pixel_len as f32 / duration,
                //     tick_distance,
                //     *pixel_len,
                //     *repeats,
                //     36
                // );

                // println!("> Slider: {:?}", slider_objects);

                let radius = OBJECT_RADIUS * scale;

                let stack_offset = {
                    let c = stack_height * scale * -6.4;

                    Pos2 { x: c, y: c }
                };

                // println!("radius={} | stack_offset={:?}", radius, stack_offset);

                let pos = h.pos;
                let stacked_pos = pos + stack_offset; // TODO: Simplify for below

                // println!(
                //     "stacked_pos = {:?} + {:?} = {:?}",
                //     pos, stack_offset, stacked_pos
                // );

                let mut cursor_end_pos = stacked_pos;
                let mut cursor_travel_dist = 0.0;
                let approx_follow_circle_radius = radius * 3.0;

                // println!(
                //     "stacked_pos={:?} | approx_follow_circle_radius={}",
                //     stacked_pos, approx_follow_circle_radius
                // );

                // let mut curr_offset = tick_distance;

                for (i, tick) in slider_objects.iter().skip(1).enumerate() {
                    let mut progress = (tick.time - h.start_time) / span_duration;

                    if progress % 2.0 >= 1.0 {
                        progress = 1.0 - progress % 1.0;
                    } else {
                        progress %= 1.0;
                    }

                    let curr_dist = pixel_len * progress;
                    let curr_pos = curve.point_at_distance(curr_dist);

                    let diff = stacked_pos + curr_pos - pos - cursor_end_pos;
                    let mut dist = diff.length();

                    // println!(
                    //     "position at: progress=? | d={} => {:?}",
                    //     curr_offset, tick.pos
                    // );
                    // curr_offset += tick_distance;

                    println!(
                        "[{}] diff = {:?} + {:?} - {:?} = {:?} | dist={}",
                        i,
                        stacked_pos,
                        tick.pos - pos,
                        cursor_end_pos,
                        diff,
                        dist
                    );

                    // println!("{} > {}", dist, approx_follow_circle_radius);

                    if dist > approx_follow_circle_radius {
                        let normalized = diff.normalize();
                        // println!("diff before: {:?}", diff);
                        // println!("diff after: {:?}", normalized);
                        dist -= approx_follow_circle_radius;
                        cursor_end_pos += normalized * dist;

                        // println!("+= {} * {} => {:?}", normalized, dist, cursor_end_pos);

                        cursor_travel_dist += dist;
                        // println!("+= {} => {}", dist, cursor_travel_dist);
                    }
                }

                println!("cursor_travel_dist={}", cursor_travel_dist);

                println!("---");

                Self::Slider {
                    pixel_len: *pixel_len,
                    repeats: *repeats,

                    objects: slider_objects,

                    scale,
                    stack_height,

                    cursor_end_pos,
                    cursor_travel_dist,
                }
            }
            HitObjectKind::Spinner { .. } => Self::Spinner { pos, time },
            HitObjectKind::Hold { .. } => panic!("found Hold object in osu!standard file"),
        }
    }

    #[inline]
    pub(crate) fn time(&self) -> f32 {
        match self {
            Self::Circle { time, .. } => *time,
            Self::Slider { objects, .. } => objects[0].time,
            Self::Spinner { time, .. } => *time,
        }
    }

    #[inline]
    pub(crate) fn radius(&self) -> f32 {
        OBJECT_RADIUS * self.scale()
    }

    #[inline]
    pub(crate) fn travel_dist(&self) -> f32 {
        match self {
            Self::Slider {
                cursor_travel_dist, ..
            } => *cursor_travel_dist,
            _ => 0.0,
        }
    }

    #[inline]
    pub(crate) fn stacked_pos(&self) -> Pos2 {
        self.pos() + self.stack_offset()
    }

    #[inline]
    pub(crate) fn cursor_end_position(&self) -> Pos2 {
        match self {
            Self::Circle { pos, .. } => *pos,
            Self::Slider { cursor_end_pos, .. } => *cursor_end_pos,
            Self::Spinner { pos, .. } => *pos,
        }
    }

    #[inline]
    pub(crate) fn is_spinner(&self) -> bool {
        matches!(self, Self::Spinner { .. })
    }

    #[inline]
    fn scale(&self) -> f32 {
        match self {
            Self::Circle { scale, .. } => *scale,
            Self::Slider { scale, .. } => *scale,
            Self::Spinner { .. } => 1.0,
        }
    }

    #[inline]
    fn stack_height(&self) -> f32 {
        match self {
            Self::Circle { stack_height, .. } => *stack_height,
            Self::Slider { stack_height, .. } => *stack_height,
            Self::Spinner { .. } => 0.0,
        }
    }

    // TODO: Remove pub
    #[inline]
    pub fn pos(&self) -> Pos2 {
        match self {
            Self::Circle { pos, .. } => *pos,
            Self::Slider { objects, .. } => objects[0].pos,
            Self::Spinner { .. } => Pos2::default(),
        }
    }

    #[inline]
    fn stack_offset(&self) -> Pos2 {
        let c = self.stack_height() * self.scale() * -6.4;

        Pos2 { x: c, y: c }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct SliderTick {
    pos: Pos2,
    time: f32,
}

impl SliderTick {
    fn new(pos: Pos2, time: f32) -> Self {
        Self { pos, time }
    }
}

// TODO: Remove
impl std::fmt::Debug for SliderTick {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{pos={:?} | time={}}}", self.pos, self.time)
    }
}
