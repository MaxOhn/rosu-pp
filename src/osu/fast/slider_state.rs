use std::f32::NEG_INFINITY;

use crate::{Beatmap, ControlPoint, ControlPointIter};

pub(crate) struct SliderState<'p> {
    control_points: ControlPointIter<'p>,
    next_time: f32,
    px_per_beat: f32,
    prev_sv: f32,
}

impl<'p> SliderState<'p> {
    #[inline]
    pub(crate) fn new(map: &'p Beatmap) -> Self {
        Self {
            control_points: ControlPointIter::new(map),
            next_time: NEG_INFINITY,
            px_per_beat: 1.0,
            prev_sv: 1.0,
        }
    }

    pub(crate) fn count_ticks(
        &mut self,
        time: f32,
        pixel_len: f32,
        span_count: usize,
        map: &Beatmap,
    ) -> usize {
        while time >= self.next_time {
            self.px_per_beat = map.slider_mult * 100.0 * self.prev_sv;

            match self.control_points.next() {
                Some(ControlPoint::Timing { time, .. }) => {
                    self.next_time = time;
                    self.prev_sv = 1.0;
                }
                Some(ControlPoint::Difficulty {
                    time,
                    slider_velocity,
                }) => {
                    self.next_time = time;
                    self.prev_sv = slider_velocity;
                }
                None => break,
            }
        }

        let spans = span_count as f32;
        let beats = pixel_len * spans / self.px_per_beat;
        let ticks = ((beats - 0.1) / spans * map.tick_rate).ceil() as usize;

        ticks
            .checked_sub(1)
            .map_or(0, |ticks| ticks * span_count + span_count + 1)
    }
}
