/// Aggregation for a score's current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManiaScoreState {
    /// Amount of current 320s.
    pub n320: u32,
    /// Amount of current 300s.
    pub n300: u32,
    /// Amount of current 200s.
    pub n200: u32,
    /// Amount of current 100s.
    pub n100: u32,
    /// Amount of current 50s.
    pub n50: u32,
    /// Amount of current misses.
    pub misses: u32,
}

impl ManiaScoreState {
    /// Create a new empty score state.
    pub const fn new() -> Self {
        Self {
            n320: 0,
            n300: 0,
            n200: 0,
            n100: 0,
            n50: 0,
            misses: 0,
        }
    }

    /// Return the total amount of hits by adding everything up.
    pub const fn total_hits(&self) -> u32 {
        self.n320 + self.n300 + self.n200 + self.n100 + self.n50 + self.misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    pub fn accuracy(&self, classic: bool) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let perfect_weight = if classic { 60 } else { 61 };

        let numerator = perfect_weight * self.n320
            + 60 * self.n300
            + 40 * self.n200
            + 20 * self.n100
            + 10 * self.n50;

        let denominator = perfect_weight * total_hits;

        f64::from(numerator) / f64::from(denominator)
    }
}

impl Default for ManiaScoreState {
    fn default() -> Self {
        Self::new()
    }
}
