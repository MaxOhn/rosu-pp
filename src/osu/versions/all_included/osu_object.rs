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
    pub(crate) stack_height: f32,
    kind: OsuObjectKind,
}

enum OsuObjectKind {
    Circle,
    Slider {
        end_time: f32,
        end_pos: Pos2,
        lazy_end_pos: Pos2,
        travel_dist: f32,
    },
    Spinner {
        end_time: f32,
    },
}

pub(crate) struct ObjectParameters<'a> {
    pub(crate) map: &'a Beatmap,
    pub(crate) radius: f32,
    pub(crate) scaling_factor: f32,
    pub(crate) max_combo: &'a mut usize,
    pub(crate) ticks: Vec<f32>,
    pub(crate) slider_state: SliderState<'a>,
    pub(crate) curve_bufs: CurveBuffers,
}

impl OsuObject {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(h: &HitObject, hr: bool, params: &mut ObjectParameters<'_>) -> Option<Self> {
        let ObjectParameters {
            map,
            radius,
            scaling_factor,
            max_combo,
            ticks,
            slider_state,
            curve_bufs,
        } = params;

        **max_combo += 1; // hitcircle, slider head, or spinner
        let mut pos = h.pos;

        if hr {
            pos.y = 384.0 - pos.y;
        }

        let obj = match &h.kind {
            HitObjectKind::Circle => Self {
                time: h.start_time,
                pos,
                stack_height: 0.0,
                kind: OsuObjectKind::Circle,
            },
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
            } => {
                // Key values which are computed here
                let mut lazy_end_pos = pos;
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
                let mut compute_vertex = |mut progress: f32| {
                    **max_combo += 1;

                    if progress % 2.0 >= 1.0 {
                        progress = 1.0 - progress % 1.0;
                    } else {
                        progress %= 1.0;
                    }

                    let mut curr_pos = h.pos + curve.position_at(progress);

                    if hr {
                        curr_pos.y = 384.0 - curr_pos.y;
                    }

                    let diff = curr_pos - lazy_end_pos;
                    let mut dist = diff.length();

                    if dist > approx_follow_circle_radius {
                        // * The cursor would be outside the follow circle, we need to move it
                        dist -= approx_follow_circle_radius;
                        lazy_end_pos += diff.normalize() * dist;
                        travel_dist += dist;
                    }
                };

                // * A very lenient maximum length of a slider for ticks to be generated.
                // * This exists for edge cases such as /b/1573664 where the beatmap has
                // * been edited by the user, and should never be reached in normal usage.
                let max_len = 100_000.0;

                let len = curve.dist().min(max_len);
                tick_dist = tick_dist.clamp(0.0, len);
                let min_dist_from_end = velocity * 10.0;

                let mut curr_dist = tick_dist;

                ticks.clear();
                ticks.reserve((len / tick_dist) as usize);

                // Tick of the first span
                while curr_dist < len - min_dist_from_end {
                    let progress = curr_dist / len;

                    compute_vertex(progress);
                    ticks.push(progress);

                    curr_dist += tick_dist;
                }

                // Other spans
                for span_idx in 1..=*repeats {
                    let progress = (span_idx % 2 == 1) as u8 as f32;

                    // Reverse tick
                    compute_vertex(progress);

                    // Actual ticks
                    if span_idx & 1 == 1 {
                        ticks
                            .iter()
                            .rev()
                            .for_each(|&tick_progress| compute_vertex(tick_progress + progress));
                    } else {
                        ticks
                            .iter()
                            .for_each(|&tick_progress| compute_vertex(tick_progress + progress));
                    }
                }

                // Slider tail
                let final_span_start_time = h.start_time + *repeats as f32 * span_duration;
                let final_span_end_time = (h.start_time + duration / 2.0)
                    .max(final_span_start_time + span_duration - LEGACY_LAST_TICK_OFFSET);
                let progress = (*repeats % 2 == 1) as u8 as f32;
                let final_progress =
                    (final_span_end_time - final_span_start_time) / span_duration + progress;

                compute_vertex(final_progress);

                let mut end_pos = h.pos + curve.position_at(progress);
                travel_dist *= *scaling_factor;

                if hr {
                    end_pos.y = 384.0 - end_pos.y;
                }

                Self {
                    time: h.start_time,
                    pos,
                    stack_height: 0.0,
                    kind: OsuObjectKind::Slider {
                        end_time: final_span_end_time,
                        end_pos,
                        lazy_end_pos,
                        travel_dist,
                    },
                }
            }
            HitObjectKind::Spinner { end_time } => Self {
                time: h.start_time,
                pos,
                stack_height: 0.0,
                kind: OsuObjectKind::Spinner {
                    end_time: *end_time,
                },
            },
            HitObjectKind::Hold { .. } => return None,
        };

        Some(obj)
    }

    #[inline]
    pub(crate) fn travel_dist(&self) -> f32 {
        match &self.kind {
            OsuObjectKind::Slider { travel_dist, .. } => *travel_dist,
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => 0.0,
        }
    }

    #[inline]
    pub(crate) fn end_time(&self) -> f32 {
        match &self.kind {
            OsuObjectKind::Circle => self.time,
            OsuObjectKind::Slider { end_time, .. } => *end_time,
            OsuObjectKind::Spinner { end_time } => *end_time,
        }
    }

    #[inline]
    pub(crate) fn end_pos(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider { end_pos, .. } => *end_pos,
        }
    }

    #[inline]
    pub(crate) fn lazy_end_pos(&self, scale_factor: f32) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider { lazy_end_pos, .. } => {
                let stack_offset = scale_factor * self.stack_height;

                *lazy_end_pos + Pos2::new(stack_offset)
            }
        }
    }

    #[inline]
    pub(crate) fn is_circle(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Circle)
    }

    #[inline]
    pub(crate) fn is_slider(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Slider { .. })
    }

    #[inline]
    pub(crate) fn is_spinner(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Spinner { .. })
    }
}
