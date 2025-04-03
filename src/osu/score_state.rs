use crate::util::{float_ext::FloatExt, hint::unlikely};

/// Aggregation for a score's current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsuScoreState {
    /// Maximum combo that the score has had so far. **Not** the maximum
    /// possible combo of the map so far.
    pub max_combo: u32,
    /// "Large tick" hits.
    ///
    /// The meaning depends on the kind of score:
    /// - if set on osu!stable, this field is irrelevant and can be `0`
    /// - if set on osu!lazer *with* slider accuracy, this field is the amount
    ///   of hit slider ticks and repeats
    /// - if set on osu!lazer *without* slider accuracy, this field is the
    ///   amount of hit slider heads, ticks, and repeats
    ///
    /// Only relevant for osu!lazer.
    pub large_tick_hits: u32,
    /// "Small ticks" hits.
    ///
    /// These are essentially the slider end hits for lazer scores without
    /// slider accuracy.
    ///
    /// Only relevant for osu!lazer.
    pub small_tick_hits: u32,
    /// Amount of successfully hit slider ends.
    ///
    /// Only relevant for osu!lazer.
    pub slider_end_hits: u32,
    /// Amount of current 300s.
    pub n300: u32,
    /// Amount of current 100s.
    pub n100: u32,
    /// Amount of current 50s.
    pub n50: u32,
    /// Amount of current misses.
    pub misses: u32,
}

impl OsuScoreState {
    /// Create a new empty score state.
    pub const fn new() -> Self {
        Self {
            max_combo: 0,
            large_tick_hits: 0,
            small_tick_hits: 0,
            slider_end_hits: 0,
            n300: 0,
            n100: 0,
            n50: 0,
            misses: 0,
        }
    }

    /// Return the total amount of hits by adding everything up.
    pub const fn total_hits(&self) -> u32 {
        self.n300 + self.n100 + self.n50 + self.misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    pub fn accuracy(&self, origin: OsuScoreOrigin) -> f64 {
        let mut numerator = f64::from(6 * self.n300 + 2 * self.n100 + self.n50);
        let mut denominator = f64::from(6 * (self.n300 + self.n100 + self.n50 + self.misses));

        match origin {
            OsuScoreOrigin::Stable => {}
            OsuScoreOrigin::WithSliderAcc {
                max_large_ticks,
                max_slider_ends,
            } => {
                let slider_end_hits = self.slider_end_hits.min(max_slider_ends);
                let large_tick_hits = self.large_tick_hits.min(max_large_ticks);

                numerator += f64::from(3 * slider_end_hits) + 0.6 * f64::from(large_tick_hits);
                denominator += f64::from(3 * max_slider_ends) + 0.6 * f64::from(max_large_ticks);
            }
            OsuScoreOrigin::WithoutSliderAcc {
                max_large_ticks,
                max_small_ticks,
            } => {
                let large_tick_hits = self.large_tick_hits.min(max_large_ticks);
                let small_tick_hits = self.small_tick_hits.min(max_small_ticks);

                numerator += 0.6 * f64::from(large_tick_hits) + 0.2 * f64::from(small_tick_hits);
                denominator += 0.6 * f64::from(max_large_ticks) + 0.2 * f64::from(max_small_ticks);
            }
        }

        if unlikely(denominator.eq(0.0)) {
            0.0
        } else {
            numerator / denominator
        }
    }
}

impl Default for OsuScoreState {
    fn default() -> Self {
        Self::new()
    }
}

/// Type to pass [`OsuScoreState::accuracy`] and specify the origin of a score.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OsuScoreOrigin {
    /// For scores set on osu!stable
    Stable,
    /// For scores set on osu!lazer with slider accuracy
    WithSliderAcc {
        max_large_ticks: u32,
        max_slider_ends: u32,
    },
    /// For scores set on osu!lazer without slider accuracy
    WithoutSliderAcc {
        max_large_ticks: u32,
        max_small_ticks: u32,
    },
}
