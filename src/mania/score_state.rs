/// Aggregation for a score's current state i.e. what are the current hitresults.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManiaScoreState {
    /// Amount of current 320s.
    pub n320: usize,
    /// Amount of current 300s.
    pub n300: usize,
    /// Amount of current 200s.
    pub n200: usize,
    /// Amount of current 100s.
    pub n100: usize,
    /// Amount of current 50s.
    pub n50: usize,
    /// Amount of current misses.
    pub n_misses: usize,
}

impl ManiaScoreState {
    /// Create a new empty score state.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    #[inline]
    pub const fn total_hits(&self) -> usize {
        self.n320 + self.n300 + self.n200 + self.n100 + self.n50 + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    #[inline]
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 6 * (self.n320 + self.n300) + 4 * self.n200 + 2 * self.n100 + self.n50;
        let denominator = 6 * total_hits;

        numerator as f64 / denominator as f64
    }
}
