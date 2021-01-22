use crate::{Beatmap, ControlPoint, ControlPointIter};

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

#[cfg(test)]
mod test {
    use crate::{Beatmap, DifficultyPoint, TimingPoint};

    use super::SliderState;

    #[test]
    fn fruits_slider_state() {
        let map = Beatmap {
            timing_points: vec![
                TimingPoint {
                    time: 1.0,
                    beat_len: 10.0,
                },
                TimingPoint {
                    time: 3.0,
                    beat_len: 20.0,
                },
                TimingPoint {
                    time: 4.0,
                    beat_len: 30.0,
                },
            ],
            difficulty_points: vec![
                DifficultyPoint {
                    time: 2.0,
                    speed_multiplier: 15.0,
                },
                DifficultyPoint {
                    time: 5.0,
                    speed_multiplier: 45.0,
                },
            ],
            ..Default::default()
        };

        let mut state = SliderState::new(&map);

        state.update(2.0);
        assert_eq!(state.beat_len, 10.0);

        state.update(3.0);
        assert_eq!(state.beat_len, 20.0);
        assert_eq!(state.speed_mult, 1.0);

        state.update(5.0);
        assert_eq!(state.beat_len, 30.0);
        assert_eq!(state.speed_mult, 45.0);
    }
}
