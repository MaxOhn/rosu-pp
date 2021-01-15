use super::control_point_iter::{ControlPoint, ControlPointIter};
use crate::Beatmap;

pub(crate) struct SliderState<'p> {
    control_points: ControlPointIter<'p>,
    next: Option<ControlPoint>,
    pub(crate) beat_len: f32,
    pub(crate) speed_mult: f32,
}

impl<'p> SliderState<'p> {
    #[inline]
    pub(crate) fn new(map: &'p Beatmap) -> Self {
        let mut control_points = ControlPointIter::new(map);

        let (beat_len, speed_mult) = match control_points.next() {
            Some(ControlPoint::Timing { beat_len, .. }) => (beat_len, 1.0),
            Some(ControlPoint::Difficulty { speed_mult, .. }) => (1000.0, speed_mult),
            None => (1000.0, 1.0),
        };

        Self {
            next: control_points.next(),
            control_points,
            beat_len,
            speed_mult,
        }
    }

    #[inline]
    pub(crate) fn update(&mut self, time: f32) {
        while let Some(next) = self.next.as_ref().filter(|n| time >= n.time()) {
            match next {
                ControlPoint::Timing { beat_len, .. } => {
                    self.beat_len = *beat_len;
                    self.speed_mult = 1.0;
                }
                ControlPoint::Difficulty { speed_mult, .. } => self.speed_mult = *speed_mult,
            }

            self.next = self.control_points.next();
        }
    }
}
