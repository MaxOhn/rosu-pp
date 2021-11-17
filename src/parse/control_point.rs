use std::cmp::Ordering;

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

/// [`TimingPoint`](crate::parse::TimingPoint) that depends on a previous one.
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
