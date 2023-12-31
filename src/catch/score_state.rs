/// Aggregation for a score's current state i.e. what was the
/// maximum combo so far and what are the current hitresults.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CatchScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    ///
    /// Note that only fruits and droplets are considered for osu!catch combo.
    pub max_combo: usize,
    /// Amount of current fruits (300s).
    pub n_fruits: usize,
    /// Amount of current droplets (100s).
    pub n_droplets: usize,
    /// Amount of current tiny droplets (50s).
    pub n_tiny_droplets: usize,
    /// Amount of current tiny droplet misses (katus).
    pub n_tiny_droplet_misses: usize,
    /// Amount of current misses (fruits and droplets).
    pub n_misses: usize,
}

impl CatchScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    #[inline]
    pub const fn total_hits(&self) -> usize {
        self.n_fruits
            + self.n_droplets
            + self.n_tiny_droplets
            + self.n_tiny_droplet_misses
            + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    #[inline]
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = self.n_fruits + self.n_droplets + self.n_tiny_droplets;
        let denominator = total_hits;

        numerator as f64 / denominator as f64
    }
}
