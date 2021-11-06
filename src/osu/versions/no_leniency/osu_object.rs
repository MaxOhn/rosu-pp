use super::super::super::DifficultyAttributes;
use super::slider_state::SliderState;

use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind, Pos2},
    Beatmap,
};

const LEGACY_LAST_TICK_OFFSET: f32 = 36.0;
const BASE_SCORING_DISTANCE: f32 = 100.0;

pub(crate) struct OsuObject {
    pub(crate) time: f32,
    pub(crate) pos: Pos2,
    pub(crate) kind: OsuObjectKind,
}

pub(crate) enum OsuObjectKind {
    Circle,
    Slider {
        end_pos: Pos2,
        lazy_end_pos: Pos2,
        travel_dist: f32,
    },
    Spinner,
}

pub(crate) struct ObjectParameters<'a> {
    pub(crate) map: &'a Beatmap,
    pub(crate) radius: f32,
    pub(crate) scaling_factor: f32,
    pub(crate) attributes: &'a mut DifficultyAttributes,
    pub(crate) ticks: Vec<f32>,
    pub(crate) slider_state: SliderState<'a>,
    pub(crate) curve_bufs: CurveBuffers,
}

impl OsuObject {
    pub(crate) fn new(h: &HitObject, params: &mut ObjectParameters) -> Option<Self> {
        let ObjectParameters {
            map,
            radius,
            scaling_factor,
            attributes,
            ticks,
            slider_state,
            curve_bufs,
        } = params;

        attributes.max_combo += 1; // hitcircle, slider head, or spinner

        let obj = match &h.kind {
            HitObjectKind::Circle => Self {
                time: h.start_time,
                pos: h.pos,
                kind: OsuObjectKind::Circle,
            },
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
            } => {
                // Key values which are computed here
                let mut lazy_end_pos = h.pos;
                let mut travel_dist = 0.0;

                // Responsible for timing point values
                slider_state.update(h.start_time);

                let span_count = (*repeats + 1) as f32;

                let approx_follow_circle_radius = *radius * 3.0;
                let mut tick_dist = 100.0 * map.slider_mult / map.tick_rate;

                if map.version >= 8 {
                    tick_dist /=
                        (100.0 / slider_state.slider_velocity).max(10.0).min(1000.0) / 100.0;
                }

                // Build the curve w.r.t. the curve points
                let curve = Curve::new(control_points, *pixel_len, curve_bufs);

                let velocity =
                    (BASE_SCORING_DISTANCE * map.slider_mult * slider_state.slider_velocity)
                        / slider_state.beat_len;

                let end_time = h.start_time + span_count * curve.dist() / velocity;
                let duration = end_time - h.start_time;
                let span_duration = duration / span_count;

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

                    let curr_pos = curve.position_at(progress);
                    let diff = h.pos + curr_pos - lazy_end_pos;
                    let mut dist = diff.length();

                    if dist > approx_follow_circle_radius {
                        // * The cursor would be outside the follow circle, we need to move it
                        dist -= approx_follow_circle_radius;
                        lazy_end_pos += diff.normalize() * dist;
                        travel_dist += dist;
                    }
                };

                let mut current_distance = tick_dist;
                let time_add = duration * (tick_dist / (pixel_len * span_count));

                let target = pixel_len - tick_dist / 8.0;
                ticks.reserve((target / tick_dist) as usize);

                // Tick of the first span
                if current_distance < target {
                    for tick_idx in 1.. {
                        let time = h.start_time + time_add * tick_idx as f32;
                        compute_vertex(time);
                        ticks.push(time);
                        current_distance += tick_dist;

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

                let end_pos = curve.position_at(1.0); // TODO: what if reversing odd amount?
                travel_dist *= *scaling_factor;

                Self {
                    time: h.start_time,
                    pos: h.pos,
                    kind: OsuObjectKind::Slider {
                        end_pos,
                        lazy_end_pos,
                        travel_dist,
                    },
                }
            }
            HitObjectKind::Spinner { .. } => Self {
                time: h.start_time,
                pos: h.pos,
                kind: OsuObjectKind::Spinner,
            },
            HitObjectKind::Hold { .. } => return None,
        };

        Some(obj)
    }

    #[inline]
    pub(crate) fn end_pos(&self) -> Pos2 {
        match self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner => self.pos,
            OsuObjectKind::Slider { end_pos, .. } => end_pos,
        }
    }

    #[inline]
    pub(crate) fn lazy_end_pos(&self) -> Pos2 {
        match self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner => self.pos,
            OsuObjectKind::Slider { lazy_end_pos, .. } => lazy_end_pos,
        }
    }

    #[inline]
    pub(crate) fn travel_dist(&self) -> f32 {
        match self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner => 0.0,
            OsuObjectKind::Slider { travel_dist, .. } => travel_dist,
        }
    }

    #[inline]
    pub(crate) fn is_slider(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Slider { .. })
    }

    #[inline]
    pub(crate) fn is_spinner(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Spinner)
    }
}
