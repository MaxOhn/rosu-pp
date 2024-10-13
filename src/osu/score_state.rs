/// Aggregation for a score's current state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OsuScoreState {
    /// Maximum combo that the score has had so far. **Not** the maximum
    /// possible combo of the map so far.
    pub max_combo: u32,
    /// Amount of successfully hit slider ticks and repeat.
    ///
    /// Only relevant for osu!lazer.
    pub slider_tick_hits: u32,
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
            slider_tick_hits: 0,
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
    ///
    /// `max_slider_ticks` and `max_slider_ends` are only relevant for
    /// `osu!lazer` scores. Otherwise, they may be `0`.
    pub fn accuracy(&self, max_slider_ticks: u32, max_slider_ends: u32) -> f64 {
        if self.total_hits() + self.slider_tick_hits + self.slider_end_hits == 0 {
            return 0.0;
        }

        debug_assert!(
            self.slider_end_hits <= max_slider_ends,
            "`self.slider_end_hits` must not be greater than `max_slider_ends`"
        );
        debug_assert!(
            self.slider_tick_hits <= max_slider_ticks,
            "`self.slider_tick_hits` must not be greater than `max_slider_ticks`"
        );

        let numerator = 300 * self.n300
            + 100 * self.n100
            + 50 * self.n50
            + 150 * self.slider_end_hits
            + 30 * self.slider_tick_hits;

        let denominator = 300 * self.n300
            + 300 * self.n100
            + 300 * self.n50
            + 300 * self.misses
            + 150 * max_slider_ends
            + 30 * max_slider_ticks;

        f64::from(numerator) / f64::from(denominator)
    }
}

impl Default for OsuScoreState {
    fn default() -> Self {
        Self::new()
    }
}
