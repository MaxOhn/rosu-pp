use std::{cmp::Ordering, iter::Copied, slice::Iter};

use crate::Beatmap;

/// New rhythm speed change.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimingPoint {
    /// The beat length for this timing section
    pub beat_len: f64,
    /// The start time of this timing section
    pub time: f64,
}

impl PartialOrd for TimingPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Default for TimingPoint {
    fn default() -> Self {
        Self {
            beat_len: 60_000.0 / 60.0,
            time: 0.0,
        }
    }
}

/// [`TimingPoint`] that depends on a previous one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DifficultyPoint {
    /// The start time for the current speed multiplier
    pub time: f64,
    /// The speed multiplier until the next timing point
    pub speed_multiplier: f64,
}

impl PartialOrd for DifficultyPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Default for DifficultyPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed_multiplier: 1.0,
        }
    }
}

/// Control point for a [`Beatmap`].
#[derive(Copy, Clone, Debug)]
pub enum ControlPoint {
    /// A timing point containing the current beat length.
    Timing(TimingPoint),
    /// A difficulty point containing the current speed multiplier.
    Difficulty(DifficultyPoint),
}

impl ControlPoint {
    /// Provides the timestamp of the control point.
    #[inline]
    pub fn time(&self) -> f64 {
        match self {
            Self::Timing(point) => point.time,
            Self::Difficulty(point) => point.time,
        }
    }
}

/// Iterator for a [`Beatmap`]'s timing- and difficulty points sorted by timestamp
#[derive(Clone, Debug)]
pub struct ControlPointIter<'p> {
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

impl<'p> Iterator for ControlPointIter<'p> {
    type Item = ControlPoint;

    #[inline]
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
        beatmap::{ControlPoint, ControlPointIter, DifficultyPoint, TimingPoint},
        Beatmap,
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
