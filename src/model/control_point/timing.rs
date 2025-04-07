/// Timing-related info about this control point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimingPoint {
    pub time: f64,
    pub beat_len: f64,
}

impl TimingPoint {
    pub const DEFAULT_BEAT_LEN: f64 =
        rosu_map::section::timing_points::TimingPoint::DEFAULT_BEAT_LEN;

    pub const DEFAULT_BPM: f64 = 60_000.0 / Self::DEFAULT_BEAT_LEN;

    pub const fn new(time: f64, beat_len: f64) -> Self {
        Self {
            time,
            beat_len: beat_len.clamp(6.0, 60_000.0),
        }
    }

    pub const fn bpm(&self) -> f64 {
        60_000.0 / self.beat_len
    }
}

impl Default for TimingPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            beat_len: Self::DEFAULT_BEAT_LEN,
        }
    }
}

pub fn timing_point_at(points: &[TimingPoint], time: f64) -> Option<&TimingPoint> {
    let i = points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .unwrap_or_else(|i| i.saturating_sub(1));

    points.get(i)
}
