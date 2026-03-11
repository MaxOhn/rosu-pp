use std::cmp;

use crate::util::{float_ext::FloatExt, hint::unlikely};

/// osu!standard hitresults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsuHitResults {
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
    /// Amount of 300s.
    pub n300: u32,
    /// Amount of 100s.
    pub n100: u32,
    /// Amount of 50s.
    pub n50: u32,
    /// Amount of misses.
    pub misses: u32,
}

impl OsuHitResults {
    /// Create new empty hitresults.
    pub const fn new() -> Self {
        Self {
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
                let slider_end_hits = cmp::min(self.slider_end_hits, max_slider_ends);
                let large_tick_hits = cmp::min(self.large_tick_hits, max_large_ticks);

                numerator += f64::from(3 * slider_end_hits) + 0.6 * f64::from(large_tick_hits);
                denominator += f64::from(3 * max_slider_ends) + 0.6 * f64::from(max_large_ticks);
            }
            OsuScoreOrigin::WithoutSliderAcc {
                max_large_ticks,
                max_small_ticks,
            } => {
                let large_tick_hits = cmp::min(self.large_tick_hits, max_large_ticks);
                let small_tick_hits = cmp::min(self.small_tick_hits, max_small_ticks);

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

impl Default for OsuHitResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregation for a score's current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsuScoreState {
    /// Maximum combo that the score has had so far. **Not** the maximum
    /// possible combo of the map so far.
    pub max_combo: u32,
    /// Hitresults of a score.
    pub hitresults: OsuHitResults,
    /// Legacy total score.
    ///
    /// Only relevant for osu!stable
    pub legacy_total_score: Option<u32>,
}

impl OsuScoreState {
    /// Create a new empty score state.
    pub const fn new() -> Self {
        Self {
            max_combo: 0,
            hitresults: OsuHitResults::new(),
            legacy_total_score: None,
        }
    }
}

impl Default for OsuScoreState {
    fn default() -> Self {
        Self::new()
    }
}

/// Type to pass [`OsuHitResults::accuracy`] and specify the origin of a score.
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

impl OsuScoreOrigin {
    /// Returns the tick score and tick max for this origin based on the
    /// accuracy formula:
    ///
    /// ```ignore
    /// acc = (300*n300 + 100*n100 + 50*n50 + tick_score) / (300*total_hits + tick_max)
    /// // => returns (tick_score, tick_max)
    /// ```
    pub(crate) fn tick_scores(
        &self,
        large_tick_hits: u32,
        small_tick_hits: u32,
        slider_end_hits: u32,
    ) -> (u32, u32) {
        match self {
            Self::Stable => (0, 0),
            Self::WithSliderAcc {
                max_large_ticks,
                max_slider_ends,
            } => (
                150 * slider_end_hits + 30 * large_tick_hits,
                150 * max_slider_ends + 30 * max_large_ticks,
            ),
            Self::WithoutSliderAcc {
                max_large_ticks,
                max_small_ticks,
            } => (
                30 * large_tick_hits + 10 * small_tick_hits,
                30 * max_large_ticks + 10 * max_small_ticks,
            ),
        }
    }
}
