/// Aggregation for a score's current state i.e. what was the
/// maximum combo so far and what are the current hitresults.
///
/// This struct is used for [`CatchGradualPerformanceAttributes`].
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
}
