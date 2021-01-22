use std::cmp::Ordering;

/// New rhythm speed change.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimingPoint {
    pub beat_len: f32,
    pub time: f32,
}

impl PartialOrd for TimingPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

/// [`TimingPoint`](crate::parse::TimingPoint) that depends on a previous one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DifficultyPoint {
    pub time: f32,
    pub speed_multiplier: f32,
}

impl PartialOrd for DifficultyPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
