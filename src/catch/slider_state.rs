use crate::{Beatmap, ControlPoint, ControlPointIter};

#[derive(Clone, Debug)]
pub(crate) struct SliderState<'p> {
    control_points: ControlPointIter<'p>,
    next: Option<ControlPoint>,
    pub(crate) beat_len: f64,
    pub(crate) slider_velocity: f64,
}

impl<'p> SliderState<'p> {
    #[inline]
    pub(crate) fn new(map: &'p Beatmap) -> Self {
        let mut control_points = ControlPointIter::new(map);

        let (beat_len, slider_velocity) = match control_points.next() {
            Some(ControlPoint::Timing(point)) => (point.beat_len, 1.0),
            Some(ControlPoint::Difficulty(point)) => (1000.0, point.speed_multiplier),
            None => (1000.0, 1.0),
        };

        Self {
            next: control_points.next(),
            control_points,
            beat_len,
            slider_velocity,
        }
    }

    #[inline]
    pub(crate) fn update(&mut self, time: f64) {
        while let Some(next) = self.next.as_ref().filter(|n| time >= n.time()) {
            match next {
                ControlPoint::Timing(point) => {
                    self.beat_len = point.beat_len;
                    self.slider_velocity = 1.0;
                }
                ControlPoint::Difficulty(point) => self.slider_velocity = point.speed_multiplier,
            }

            self.next = self.control_points.next();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parse::{DifficultyPoint, TimingPoint},
        Beatmap,
    };

    use super::SliderState;

    #[test]
    fn catch_slider_state() {
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
        assert!((state.beat_len - 10.0).abs() <= f64::EPSILON);

        state.update(3.0);
        assert!((state.beat_len - 20.0).abs() <= f64::EPSILON);
        assert!((state.slider_velocity - 1.0).abs() <= f64::EPSILON);

        state.update(5.0);
        assert!((state.beat_len - 30.0).abs() <= f64::EPSILON);
        assert!((state.slider_velocity - 45.0).abs() <= f64::EPSILON);
    }
}
