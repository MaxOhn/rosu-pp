use crate::{Beatmap, DifficultyPoint, TimingPoint};

use std::slice::Iter;

macro_rules! next_tuple {
    ($iter:expr, ($first:ident, $second:ident)) => {
        $iter.next().map(|e| (e.$first, e.$second))
    };
}

pub(crate) struct ControlPointIter<'p> {
    timing_points: Iter<'p, TimingPoint>,
    difficulty_points: Iter<'p, DifficultyPoint>,

    next_timing: Option<(f32, f32)>,
    next_difficulty: Option<(f32, f32)>,
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
    Timing { time: f32, beat_len: f32 },
    Difficulty { time: f32, speed_mult: f32 },
}

impl ControlPoint {
    #[inline]
    pub(crate) fn time(&self) -> f32 {
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
            (_, Some((time, speed_mult))) => {
                self.next_difficulty =
                    next_tuple!(self.difficulty_points, (time, speed_multiplier));

                Some(ControlPoint::Difficulty { time, speed_mult })
            }
            (Some((time, beat_len)), None) => {
                self.next_timing = next_tuple!(self.timing_points, (time, beat_len));

                Some(ControlPoint::Timing { time, beat_len })
            }
            (None, None) => None,
        }
    }
}
