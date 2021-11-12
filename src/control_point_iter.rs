#![cfg(any(feature = "osu", feature = "fruits"))]

use crate::{
    parse::{DifficultyPoint, TimingPoint},
    Beatmap,
};

use std::slice::Iter;

macro_rules! next_tuple {
    ($iter:expr, ($first:ident, $second:ident)) => {
        $iter.next().map(|e| (e.$first, e.$second))
    };
}

pub(crate) struct ControlPointIter<'p> {
    timing_points: Iter<'p, TimingPoint>,
    difficulty_points: Iter<'p, DifficultyPoint>,

    next_timing: Option<(f64, f64)>,
    next_difficulty: Option<(f64, f64)>,
}

impl<'p> ControlPointIter<'p> {
    #[inline]
    pub(crate) fn new(map: &'p Beatmap) -> Self {
        let mut timing_points = map.timing_points.iter();
        let mut difficulty_points = map.difficulty_points.iter();

        Self {
            next_timing: next_tuple!(timing_points, (time, beat_len)),
            next_difficulty: next_tuple!(difficulty_points, (time, speed_multiplier)),

            timing_points,
            difficulty_points,
        }
    }
}

pub(crate) enum ControlPoint {
    Timing { time: f64, beat_len: f64 },
    Difficulty { time: f64, slider_velocity: f64 },
}

#[cfg(any(feature = "osu", feature = "fruits"))]
impl ControlPoint {
    #[inline]
    pub(crate) fn time(&self) -> f64 {
        match self {
            Self::Timing { time, .. } => *time,
            Self::Difficulty { time, .. } => *time,
        }
    }
}

impl<'p> Iterator for ControlPointIter<'p> {
    type Item = ControlPoint;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.next_timing, self.next_difficulty) {
            (Some((time, beat_len)), Some((d, _))) if time <= d => {
                self.next_timing = next_tuple!(self.timing_points, (time, beat_len));

                Some(ControlPoint::Timing { time, beat_len })
            }
            (_, Some((time, slider_velocity))) => {
                self.next_difficulty =
                    next_tuple!(self.difficulty_points, (time, speed_multiplier));

                Some(ControlPoint::Difficulty {
                    time,
                    slider_velocity,
                })
            }
            (Some((time, beat_len)), None) => {
                self.next_timing = next_tuple!(self.timing_points, (time, beat_len));

                Some(ControlPoint::Timing { time, beat_len })
            }
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parse::{DifficultyPoint, TimingPoint},
        Beatmap, ControlPoint, ControlPointIter,
    };

    #[test]
    fn control_point_iter() {
        let map = Beatmap {
            timing_points: vec![
                TimingPoint {
                    time: 1.0,
                    beat_len: 10.0,
                },
                TimingPoint {
                    time: 3.0,
                    beat_len: 10.0,
                },
                TimingPoint {
                    time: 4.0,
                    beat_len: 10.0,
                },
            ],
            difficulty_points: vec![
                DifficultyPoint {
                    time: 2.0,
                    speed_multiplier: 10.0,
                },
                DifficultyPoint {
                    time: 5.0,
                    speed_multiplier: 10.0,
                },
            ],
            ..Default::default()
        };

        let mut iter = ControlPointIter::new(&map);

        assert!(matches!(iter.next(), Some(ControlPoint::Timing { .. })));
        assert!(matches!(iter.next(), Some(ControlPoint::Difficulty { .. })));
        assert!(matches!(iter.next(), Some(ControlPoint::Timing { .. })));
        assert!(matches!(iter.next(), Some(ControlPoint::Timing { .. })));
        assert!(matches!(iter.next(), Some(ControlPoint::Difficulty { .. })));
        assert!(matches!(iter.next(), None));
    }
}
