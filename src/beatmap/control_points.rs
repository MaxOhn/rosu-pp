use std::{cmp::Ordering, iter::Copied, slice::Iter};

use crate::Beatmap;

/// New rhythm speed change.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimingPoint {
    /// The beat length for this timing section
    pub beat_len: f64,
    /// The start time of this timing section
    pub time: f64,
    /// Whether the section between this and the
    /// next timing points is a kiai section
    pub kiai: bool,
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
            kiai: false,
        }
    }
}

/// [`TimingPoint`] that depends on a previous one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DifficultyPoint {
    /// The time at which the control point takes effect.
    pub time: f64,
    /// The slider velocity at this control point.
    pub slider_vel: f64,
    /// Whether the section between this and the
    /// next timing points is a kiai section
    pub kiai: bool,
    /// Legacy BPM multiplier that introduces floating-point errors for rulesets that depend on it.
    pub bpm_mult: f64,
    /// Whether or not slider ticks should be generated at this control point.
    /// This exists for backwards compatibility with maps that abuse NaN
    /// slider velocity behavior on osu!stable (e.g. /b/2628991).
    pub generate_ticks: bool,
}

impl DifficultyPoint {
    /// The default slider velocity for a [`DifficultyPoint`]
    pub const DEFAULT_SLIDER_VEL: f64 = 1.0;
    /// The default BPM multipler for a [`DifficultyPoint`]
    pub const DEFAULT_BPM_MULT: f64 = 1.0;
    /// The default for generating ticks of a [`DifficultyPoint`]
    pub const DEFAULT_GENERATE_TICKS: bool = true;

    /// Create a new [`DifficultyPoint`]
    pub fn new(time: f64, beat_len: f64, speed_multiplier: f64, kiai: bool) -> Self {
        // * Note: In stable, the division occurs on floats, but with compiler optimisations
        // * turned on actually seems to occur on doubles via some .NET black magic (possibly inlining?).
        let bpm_multiplier = if beat_len < 0.0 {
            ((-beat_len) as f32).clamp(10.0, 10_000.0)
        } else {
            1.0
        };

        Self {
            time,
            slider_vel: speed_multiplier.clamp(0.1, 10.0),
            kiai,
            bpm_mult: bpm_multiplier as f64,
            generate_ticks: !beat_len.is_nan(),
        }
    }

    pub(crate) fn is_redundant(&self, existing: &DifficultyPoint) -> bool {
        (self.slider_vel - existing.slider_vel).abs() <= f64::EPSILON
            && self.generate_ticks == existing.generate_ticks
    }
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
            kiai: false,
            slider_vel: Self::DEFAULT_SLIDER_VEL,
            bpm_mult: Self::DEFAULT_BPM_MULT,
            generate_ticks: Self::DEFAULT_GENERATE_TICKS,
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
        let mut map = Beatmap::default();

        map.timing_points.push(TimingPoint {
            time: 1.0,
            beat_len: 10.0,
            kiai: false,
        });

        map.timing_points.push(TimingPoint {
            time: 3.0,
            beat_len: 10.0,
            kiai: false,
        });

        map.timing_points.push(TimingPoint {
            time: 4.0,
            beat_len: 10.0,
            kiai: false,
        });

        map.difficulty_points
            .push(DifficultyPoint::new(2.0, 10.0, 10.0, false));
        map.difficulty_points
            .push(DifficultyPoint::new(5.0, 10.0, 10.0, false));

        let mut iter = ControlPointIter::new(&map);

        assert!(matches!(iter.next(), Some(ControlPoint::Timing(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Difficulty(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Timing(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Timing(_))));
        assert!(matches!(iter.next(), Some(ControlPoint::Difficulty(_))));
        assert!(matches!(iter.next(), None));
    }
}
