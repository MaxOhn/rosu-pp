use crate::{
    parse::{DifficultyPoint, TimingPoint},
    Beatmap,
};

use std::{iter::Copied, slice::Iter};

#[derive(Clone, Debug)]
pub(crate) struct ControlPointIter<'p> {
    timing_points: Copied<Iter<'p, TimingPoint>>,
    difficulty_points: Copied<Iter<'p, DifficultyPoint>>,

    next_timing: Option<TimingPoint>,
    next_difficulty: Option<DifficultyPoint>,
}

impl<'p> ControlPointIter<'p> {
    #[inline]
    pub(crate) fn new(map: &'p Beatmap) -> Self {
        let mut timing_points = map.timing_points.iter().copied();
        let mut difficulty_points = map.difficulty_points.iter().copied();

        Self {
            next_timing: timing_points.next(),
            next_difficulty: difficulty_points.next(),

            timing_points,
            difficulty_points,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ControlPoint {
    Timing(TimingPoint),
    Difficulty(DifficultyPoint),
}

impl ControlPoint {
    #[inline]
    pub(crate) fn time(&self) -> f64 {
        match self {
            Self::Timing(point) => point.time,
            Self::Difficulty(point) => point.time,
        }
    }
}

impl<'p> Iterator for ControlPointIter<'p> {
    type Item = ControlPoint;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.next_timing, self.next_difficulty) {
            (Some(timing), Some(difficulty)) if timing.time <= difficulty.time => {
                self.next_timing = self.timing_points.next();

                Some(ControlPoint::Timing(timing))
            }
            (_, Some(point)) => {
                self.next_difficulty = self.difficulty_points.next();

                Some(ControlPoint::Difficulty(point))
            }
            (Some(point), None) => {
                self.next_timing = self.timing_points.next();

                Some(ControlPoint::Timing(point))
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

        assert!(matches!(iter.next(), Some(ControlPoint::Timing(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Difficulty(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Timing(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Timing(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Difficulty(_))));
        assert!(matches!(iter.next(), None));
    }
}
