use crate::{
    parse::{HitObject, HitObjectKind, Pos2},
    Beatmap,
};

use super::slider_state::SliderState;

pub(crate) struct OsuObject {
    pub(crate) pos: Pos2,
    pub(crate) time: f32,
    kind: OsuObjectKind,
}

pub(crate) enum OsuObjectKind {
    Circle,
    Slider { end_pos: Pos2, travel_dist: f32 },
    Spinner,
}

pub(crate) struct ObjectParameters<'a> {
    pub(crate) map: &'a Beatmap,
    pub(crate) radius: f32,
    pub(crate) clock_rate: f32,
    pub(crate) max_combo: usize,
    pub(crate) slider_state: SliderState<'a>,
}

impl OsuObject {
    pub(crate) fn new(h: &HitObject, params: &mut ObjectParameters<'_>) -> Option<Self> {
        let time = h.start_time / params.clock_rate;

        let obj = match &h.kind {
            HitObjectKind::Circle => {
                params.max_combo += 1;

                Self::circle(h.pos, time)
            }
            #[cfg(feature = "sliders")]
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
            } => {
                let span_count = *repeats + 1;

                params.max_combo += params.slider_state.count_ticks(
                    h.start_time,
                    *pixel_len,
                    span_count,
                    params.map,
                );

                match control_points.last() {
                    Some(point) => {
                        let follow_circle_radius = params.radius * 3.0;

                        let travel_dist = Self::approximate_travel_dist(
                            follow_circle_radius,
                            span_count as f32,
                            point.pos,
                        );

                        let mut end_pos = h.pos;

                        if repeats % 2 == 0 && *pixel_len > follow_circle_radius {
                            end_pos += point.pos
                        }

                        Self {
                            pos: h.pos,
                            time,
                            kind: OsuObjectKind::Slider {
                                end_pos,
                                travel_dist,
                            },
                        }
                    }
                    None => Self::circle(h.pos, time),
                }
            }
            #[cfg(not(feature = "sliders"))]
            HitObjectKind::Slider {
                pixel_len,
                span_count,
                last_control_point,
            } => {
                params.max_combo += params.slider_state.count_ticks(
                    h.start_time,
                    *pixel_len,
                    *span_count,
                    params.map,
                );
                let follow_circle_radius = params.radius * 3.0;

                let travel_dist = Self::approximate_travel_dist(
                    follow_circle_radius,
                    *span_count as f32,
                    *last_control_point - h.pos,
                );

                let end_pos = if span_count % 2 == 1 && *pixel_len > follow_circle_radius {
                    *last_control_point
                } else {
                    h.pos
                };

                Self {
                    pos: h.pos,
                    time,
                    kind: OsuObjectKind::Slider {
                        end_pos,
                        travel_dist,
                    },
                }
            }
            HitObjectKind::Spinner { .. } => {
                params.max_combo += 1;

                Self {
                    pos: h.pos,
                    time,
                    kind: OsuObjectKind::Spinner,
                }
            }
            HitObjectKind::Hold { .. } => return None,
        };

        Some(obj)
    }

    fn circle(pos: Pos2, time: f32) -> Self {
        Self {
            pos,
            time,
            kind: OsuObjectKind::Circle,
        }
    }

    pub(crate) fn is_slider(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Slider { .. })
    }

    pub(crate) fn is_spinner(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Spinner)
    }

    pub(crate) fn end_pos(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner => self.pos,
            OsuObjectKind::Slider { end_pos, .. } => *end_pos,
        }
    }

    pub(crate) fn travel_dist(&self) -> f32 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner => 0.0,
            OsuObjectKind::Slider { travel_dist, .. } => *travel_dist,
        }
    }

    // Approximating lower bound for lazy travel distance
    fn approximate_travel_dist(
        follow_circle_radius: f32,
        span_count: f32,
        last_control_point: Pos2,
    ) -> f32 {
        let lazy_end_point_dist = follow_circle_radius * span_count;
        let dist = last_control_point.length();

        (dist * span_count - lazy_end_point_dist).max(0.0)
    }
}
