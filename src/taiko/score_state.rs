/// Aggregation for a score's current state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TaikoScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    pub max_combo: u32,
    /// Amount of current 300s.
    pub n300: u32,
    /// Amount of current 100s.
    pub n100: u32,
    /// Amount of current misses.
    pub misses: u32,
}

impl TaikoScoreState {
    /// Create a new empty score state.
    pub const fn new() -> Self {
        Self {
            max_combo: 0,
            n300: 0,
            n100: 0,
            misses: 0,
        }
    }

    /// Return the total amount of hits by adding everything up.
    pub const fn total_hits(&self) -> u32 {
        self.n300 + self.n100 + self.misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 2 * self.n300 + self.n100;
        let denominator = 2 * total_hits;

        f64::from(numerator) / f64::from(denominator)
    }
}

impl Default for TaikoScoreState {
    fn default() -> Self {
        Self::new()
    }
}
