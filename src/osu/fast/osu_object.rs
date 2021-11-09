use crate::parse::{HitObject, Pos2};

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

impl OsuObject {
    pub(crate) fn circle(h: &HitObject, clock_rate: f32) -> Self {
        Self {
            pos: h.pos,
            time: h.start_time / clock_rate,
            kind: OsuObjectKind::Circle,
        }
    }

    #[cfg(feature = "sliders")]
    pub(crate) fn slider(
        h: &HitObject,
        clock_rate: f32,
        radius: f32,
        repeats: usize,
        pixel_len: f32,
        control_points: &[crate::parse::PathControlPoint],
    ) -> Self {
        match control_points.last() {
            Some(point) => {
                let follow_circle_radius = radius * 3.0;
                let span_count = (repeats + 1) as f32;

                let travel_dist =
                    Self::approximate_travel_dist(follow_circle_radius, span_count, point.pos);

                let mut end_pos = h.pos;

                if repeats % 2 == 0 && pixel_len > follow_circle_radius {
                    end_pos += point.pos
                }

                Self {
                    pos: h.pos,
                    time: h.start_time / clock_rate,
                    kind: OsuObjectKind::Slider {
                        end_pos,
                        travel_dist,
                    },
                }
            }
            None => Self::circle(h, clock_rate),
        }
    }

    #[cfg(not(feature = "sliders"))]
    pub(crate) fn slider(
        h: &HitObject,
        clock_rate: f32,
        radius: f32,
        span_count: usize,
        pixel_len: f32,
        last_control_point: Pos2,
    ) -> Self {
        let follow_circle_radius = radius * 3.0;

        let travel_dist = Self::approximate_travel_dist(
            follow_circle_radius,
            span_count as f32,
            point.pos - h.pos,
        );

        let end_pos = if span_count % 2 == 1 && pixel_len > follow_circle_radius {
            last_control_point
        } else {
            h.pos
        };

        Self {
            pos: h.pos,
            time: h.start_time / clock_rate,
            kind: OsuObjectKind::Slider {
                end_pos,
                travel_dist,
            },
        }
    }

    pub(crate) fn spinner(h: &HitObject, clock_rate: f32) -> Self {
        Self {
            pos: h.pos,
            time: h.start_time / clock_rate,
            kind: OsuObjectKind::Spinner,
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
