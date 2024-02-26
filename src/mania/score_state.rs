/// Aggregation for a score's current state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    pub const fn total_hits(&self) -> u32 {
        self.n320 + self.n300 + self.n200 + self.n100 + self.n50 + self.misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 6 * (self.n320 + self.n300) + 4 * self.n200 + 2 * self.n100 + self.n50;
        let denominator = 6 * total_hits;

        f64::from(numerator) / f64::from(denominator)
    }
}
