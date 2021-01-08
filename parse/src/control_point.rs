use std::cmp::Ordering;

#[derive(PartialEq)]
pub struct TimingPoint {
    pub beat_len: f32,
    pub bpm: f32,
    pub time: f32,
}

impl PartialOrd for TimingPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

#[derive(PartialEq)]
pub struct DifficultyPoint {
    pub time: f32,
    pub speed_multiplier: f32,
}

impl PartialOrd for DifficultyPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
