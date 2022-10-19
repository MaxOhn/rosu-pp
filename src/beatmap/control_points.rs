use std::cmp::Ordering;

/// New rhythm speed change.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimingPoint {
    /// The beat length for this timing section
    pub beat_len: f64,
    /// The start time of this timing section
    pub time: f64,
}

impl TimingPoint {
    /// Create a new [`TimingPoint`].
    #[inline]
    pub fn new(time: f64, beat_len: f64) -> Self {
        Self { time, beat_len }
    }
}

impl PartialOrd for TimingPoint {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Default for TimingPoint {
    #[inline]
    fn default() -> Self {
        Self::new(0.0, 60_000.0 / 60.0)
    }
}

/// [`TimingPoint`] that depends on a previous one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DifficultyPoint {
    /// The time at which the control point takes effect.
    pub time: f64,
    /// The slider velocity at this control point.
    pub slider_vel: f64,
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

    /// Create a new [`DifficultyPoint`].
    #[inline]
    pub fn new(time: f64, beat_len: f64, speed_multiplier: f64) -> Self {
        // * Note: In stable, the division occurs on floats, but with compiler optimisations
        // * turned on actually seems to occur on doubles via some .NET black magic (possibly inlining?).
        let bpm_multiplier = if beat_len < 0.0 {
            ((-beat_len) as f32).clamp(10.0, 10_000.0) as f64 / 100.0
        } else {
            1.0
        };

        Self {
            time,
            slider_vel: speed_multiplier.clamp(0.1, 10.0),
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
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Default for DifficultyPoint {
    #[inline]
    fn default() -> Self {
        Self {
            time: 0.0,
            slider_vel: Self::DEFAULT_SLIDER_VEL,
            bpm_mult: Self::DEFAULT_BPM_MULT,
            generate_ticks: Self::DEFAULT_GENERATE_TICKS,
        }
    }
}

/// Control point storing effects and their timestamps.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EffectPoint {
    /// The time at which the control point takes effect.
    pub time: f64,
    /// Whether this control point enables Kiai mode.
    pub kiai: bool,
}

impl EffectPoint {
    /// The default slider velocity for a [`DifficultyPoint`]
    pub const DEFAULT_KIAI: bool = false;

    /// Create a new [`EffectPoint`].
    #[inline]
    pub fn new(time: f64, kiai: bool) -> Self {
        Self { time, kiai }
    }
}

impl Default for EffectPoint {
    #[inline]
    fn default() -> Self {
        Self::new(0.0, Self::DEFAULT_KIAI)
    }
}
