use super::super::super::DifficultyAttributes;
use super::slider_state::SliderState;

use crate::{curve::Curve, Beatmap, HitObject, HitObjectKind, PathType, Pos2};

const LEGACY_LAST_TICK_OFFSET: f32 = 36.0;

pub(crate) struct OsuObject {
    pub(crate) time: f32,
    pub(crate) pos: Pos2,
    pub(crate) end_pos: Pos2,
    // circle: Some(0.0) | slider: Some(_) | spinner: None
    pub(crate) travel_dist: Option<f32>,
}

impl OsuObject {
    pub(crate) fn new(
        h: &HitObject,
        map: &Beatmap,
        radius: f32,
        ticks: &mut Vec<f32>,
        attributes: &mut DifficultyAttributes,
        slider_state: &mut SliderState,
    ) -> Self {
        attributes.max_combo += 1; // hitcircle, slider head, or spinner

        match &h.kind {
            HitObjectKind::Circle => {
                attributes.n_circles += 1;

                Self {
                    time: h.start_time,
                    pos: h.pos,
                    end_pos: h.pos,
                    travel_dist: Some(0.0),
                }
            }
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                curve_points,
                path_type,
            } => {
                // Key values which are computed here
                let mut end_pos = h.pos;
                let mut travel_dist = 0.0;

                slider_state.update(h.start_time); // Responsible for timing point values
                let approx_follow_circle_radius = radius * 3.0;
                let mut tick_distance = 100.0 * map.sv / map.tick_rate;

                if map.version >= 8 {
                    tick_distance /=
                        (100.0 / slider_state.speed_mult).max(10.0).min(1000.0) / 100.0;
                }

                let duration = *repeats as f32 * slider_state.beat_len * pixel_len
                    / (map.sv * slider_state.speed_mult)
                    / 100.0;
                let span_duration = duration / *repeats as f32;

                // Ensure path type validity
                let path_type = if *path_type == PathType::PerfectCurve && curve_points.len() > 3 {
                    PathType::Bezier
                } else if curve_points.len() == 2 {
                    PathType::Linear
                } else {
                    *path_type
                };

                // Build the curve w.r.t. the curve points
                let curve = match path_type {
                    PathType::Linear => Curve::linear(curve_points[0], curve_points[1]),
                    PathType::Bezier => Curve::bezier(&curve_points),
                    PathType::Catmull => Curve::catmull(&curve_points),
                    PathType::PerfectCurve => Curve::perfect(&curve_points),
                };

                // Called on each slider object except for the head.
                // Increases combo and adjusts `end_pos` and `travel_dist`
                // w.r.t. the object position at the given time on the slider curve.
                let mut compute_vertex = |time: f32| {
                    attributes.max_combo += 1;

                    let mut progress = (time - h.start_time) / span_duration;

                    if progress % 2.0 >= 1.0 {
                        progress = 1.0 - progress % 1.0;
                    } else {
                        progress %= 1.0;
                    }

                    let curr_dist = pixel_len * progress;
                    let curr_pos = curve.point_at_distance(curr_dist);

                    let diff = curr_pos - end_pos;
                    let mut dist = diff.length();

                    if dist > approx_follow_circle_radius {
                        dist -= approx_follow_circle_radius;
                        end_pos += diff.normalize() * dist;
                        travel_dist += dist;
                    }
                };

                let mut current_distance = tick_distance;
                let time_add = duration * (tick_distance / (pixel_len * *repeats as f32));

                let target = pixel_len - tick_distance / 8.0;
                ticks.reserve((target / tick_distance) as usize);

                // Tick of the first span
                if current_distance < target {
                    for tick_idx in 1.. {
                        let time = h.start_time + time_add * tick_idx as f32;
                        compute_vertex(time);
                        ticks.push(time);
                        current_distance += tick_distance;

                        if current_distance >= target {
                            break;
                        }
                    }
                }

                // Other spans
                if *repeats > 1 {
                    for repeat_id in 1..*repeats {
                        let time_offset = (duration / *repeats as f32) * repeat_id as f32;

                        // Reverse tick
                        compute_vertex(h.start_time + time_offset);

                        // Actual ticks
                        if repeat_id & 1 == 1 {
                            ticks.iter().rev().for_each(|&time| compute_vertex(time));
                        } else {
                            ticks.iter().for_each(|&time| compute_vertex(time));
                        }
                    }
                }

                // Slider tail
                let final_span_idx = repeats.saturating_sub(1);
                let final_span_start_time = h.start_time + final_span_idx as f32 * span_duration;
                let final_span_end_time = (h.start_time + duration / 2.0)
                    .max(final_span_start_time + span_duration - LEGACY_LAST_TICK_OFFSET);
                compute_vertex(final_span_end_time);

                ticks.clear();

                Self {
                    time: h.start_time,
                    pos: h.pos,
                    end_pos,
                    travel_dist: Some(travel_dist),
                }
            }
            HitObjectKind::Spinner { .. } => {
                attributes.n_spinners += 1;

                Self {
                    time: h.start_time,
                    pos: h.pos,
                    end_pos: h.pos,
                    travel_dist: None,
                }
            }
            HitObjectKind::Hold { .. } => panic!("found Hold object in osu!standard file"),
        }
    }

    #[inline]
    pub(crate) fn is_spinner(&self) -> bool {
        self.travel_dist.is_none()
    }
}
