/// Aggregation for a score's current state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatchScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    ///
    /// Note that only fruits and droplets are considered for osu!catch combo.
    pub max_combo: u32,
    /// Amount of current fruits (300s).
    pub n_fruits: u32,
    /// Amount of current droplets (100s).
    pub n_droplets: u32,
    /// Amount of current tiny droplets (50s).
    pub n_tiny_droplets: u32,
    /// Amount of current tiny droplet misses (katus).
    pub n_tiny_droplet_misses: u32,
    /// Amount of current misses (fruits and droplets).
    pub n_misses: u32,
}

impl CatchScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    pub const fn total_hits(&self) -> u32 {
        self.n_fruits
            + self.n_droplets
            + self.n_tiny_droplets
            + self.n_tiny_droplet_misses
            + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = self.n_fruits + self.n_droplets + self.n_tiny_droplets;
        let denominator = total_hits;

        f64::from(numerator) / f64::from(denominator)
    }
}
