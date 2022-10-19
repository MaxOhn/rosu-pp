use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind},
    Beatmap,
};

const BASE_SCORING_DISTANCE: f64 = 100.0;

pub(crate) struct ObjectParameters<'a> {
    pub(crate) map: &'a Beatmap,
    pub(crate) max_combo: usize,
    pub(crate) curve_bufs: CurveBuffers,
}

impl<'a> ObjectParameters<'a> {
    pub(crate) fn new(map: &'a Beatmap) -> Self {
        Self {
            map,
            max_combo: 0,
            curve_bufs: CurveBuffers::default(),
        }
    }
}

pub(crate) struct ManiaObject {
    pub(crate) start_time: f64,
    pub(crate) end_time: f64,
    pub(crate) column: usize,
}

impl ManiaObject {
    pub(crate) fn column(x: f32, total_columns: f32) -> usize {
        let x_divisor = 512.0 / total_columns;

        (x / x_divisor).floor().min(total_columns - 1.0) as usize
    }

    pub(crate) fn new(
        h: &HitObject,
        total_columns: f32,
        params: &mut ObjectParameters<'_>,
    ) -> Self {
        let ObjectParameters {
            map,
            max_combo,
            curve_bufs,
        } = params;

        let column = Self::column(h.pos.x, total_columns);
        *max_combo += 1;

        match &h.kind {
            HitObjectKind::Circle => Self {
                start_time: h.start_time,
                end_time: h.start_time,
                column,
            },
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
                ..
            } => {
                let span_count = *repeats as f64 + 1.0;

                let curve = Curve::new(control_points, *pixel_len, curve_bufs);
                let dist = curve.dist();

                let timing_point = map.timing_point_at(h.start_time);
                let difficulty_point = map.difficulty_point_at(h.start_time).unwrap_or_default();

                let scoring_dist =
                    BASE_SCORING_DISTANCE * map.slider_mult * difficulty_point.slider_vel;

                let vel = scoring_dist / timing_point.beat_len;
                let duration = span_count * dist / vel;
                let end_time = h.start_time + duration;

                *max_combo += (duration / 100.0) as usize;

                Self {
                    start_time: h.start_time,
                    end_time,
                    column,
                }
            }
            HitObjectKind::Spinner { end_time } | HitObjectKind::Hold { end_time } => {
                *max_combo += ((*end_time - h.start_time) / 100.0) as usize;

                Self {
                    start_time: h.start_time,
                    end_time: *end_time,
                    column,
                }
            }
        }
    }
}
