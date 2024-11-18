/// Aggregation for a score's current state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OsuScoreState {
    /// Maximum combo that the score has had so far. **Not** the maximum
    /// possible combo of the map so far.
    pub max_combo: u32,
    /// "Large tick" hits.
    ///
    /// The meaning depends on the kind of score:
    /// - if set on osu!stable, this field is irrelevant and can be `0`
    /// - if set on osu!lazer *without* `CL`, this field is the amount of hit
    ///   slider ticks and repeats
    /// - if set on osu!lazer *with* `CL`, this field is the amount of hit
    ///   slider heads, ticks, and repeats
    pub large_tick_hits: u32,
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
        let mut numerator = 300 * self.n300 + 100 * self.n100 + 50 * self.n50;
        let mut denominator = 300 * (self.n300 + self.n100 + self.n50 + self.misses);

        match origin {
            OsuScoreOrigin::Stable => {}
            OsuScoreOrigin::LazerWithoutClassic {
                max_large_ticks,
                max_slider_ends,
            } => {
                let slider_end_hits = self.slider_end_hits.min(max_slider_ends);
                let large_tick_hits = self.large_tick_hits.min(max_large_ticks);

                numerator += 150 * slider_end_hits + 30 * large_tick_hits;
                denominator += 150 * max_slider_ends + 30 * max_large_ticks;
            }
            OsuScoreOrigin::LazerWithClassic {
                max_large_ticks,
                max_slider_ends,
            } => {
                let large_tick_hits = self.large_tick_hits.min(max_large_ticks);
                let slider_end_hits = self.slider_end_hits.min(max_slider_ends);

                numerator += 30 * large_tick_hits + 10 * slider_end_hits;
                denominator += 30 * max_large_ticks + 10 * max_slider_ends;
            }
        }

        if denominator == 0 {
            0.0
        } else {
            f64::from(numerator) / f64::from(denominator)
        }
    }
}

impl Default for OsuScoreState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OsuScoreOrigin {
    /// For scores set on osu!stable
    Stable,
    /// For scores set on osu!lazer without the `Classic` mod
    LazerWithoutClassic {
        max_large_ticks: u32,
        max_slider_ends: u32,
    },
    /// For scores set on osu!lazer with the `Classic` mod
    LazerWithClassic {
        max_large_ticks: u32,
        max_slider_ends: u32,
    },
}
