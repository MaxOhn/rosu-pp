/// Aggregation for a score's current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatchScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    ///
    /// Note that only fruits and droplets are considered for osu!catch combo.
    pub max_combo: u32,
    /// Amount of current fruits (300s).
    pub fruits: u32,
    /// Amount of current droplets (100s).
    pub droplets: u32,
    /// Amount of current tiny droplets (50s).
    pub tiny_droplets: u32,
    /// Amount of current tiny droplet misses (katus).
    pub tiny_droplet_misses: u32,
    /// Amount of current misses (fruits and droplets).
    pub misses: u32,
}

impl CatchScoreState {
    /// Create a new empty score state.
    pub const fn new() -> Self {
        Self {
            max_combo: 0,
            fruits: 0,
            droplets: 0,
            tiny_droplets: 0,
            tiny_droplet_misses: 0,
            misses: 0,
        }
    }

    /// Return the total amount of hits by adding everything up.
    pub const fn total_hits(&self) -> u32 {
        self.fruits + self.droplets + self.tiny_droplets + self.tiny_droplet_misses + self.misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = self.fruits + self.droplets + self.tiny_droplets;
        let denominator = total_hits;

        f64::from(numerator) / f64::from(denominator)
    }
}

impl Default for CatchScoreState {
    fn default() -> Self {
        Self::new()
    }
}
