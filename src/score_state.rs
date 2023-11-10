use crate::{
    catch::CatchScoreState, mania::ManiaScoreState, osu::OsuScoreState, taiko::TaikoScoreState,
    GameMode,
};

/// Aggregation for a score's current state i.e. what is
/// the maximum combo so far, what are the current
/// hitresults and what is the current score.
///
/// This struct is used for [`GradualPerformance`](crate::GradualPerformance).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    ///
    /// Note that for osu!catch only fruits and droplets are considered for combo.
    ///
    /// Irrelevant for osu!mania.
    pub max_combo: usize,
    /// Amount of current gekis (n320 for osu!mania).
    pub n_geki: usize,
    /// Amount of current katus (tiny droplet misses for osu!catch / n200 for osu!mania).
    pub n_katu: usize,
    /// Amount of current 300s (fruits for osu!catch).
    pub n300: usize,
    /// Amount of current 100s (droplets for osu!catch).
    pub n100: usize,
    /// Amount of current 50s (tiny droplets for osu!catch).
    pub n50: usize,
    /// Amount of current misses (fruits + droplets for osu!catch).
    pub n_misses: usize,
}

impl ScoreState {
    /// Create a new empty score state.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up based on the mode.
    #[inline]
    pub fn total_hits(&self, mode: GameMode) -> usize {
        let mut amount = self.n300 + self.n100 + self.n_misses;

        if mode != GameMode::Taiko {
            amount += self.n50;

            if mode != GameMode::Osu {
                amount += self.n_katu;
                amount += (mode != GameMode::Catch) as usize * self.n_geki;
            }
        }

        amount
    }
}

impl From<ScoreState> for OsuScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            n_misses: state.n_misses,
        }
    }
}

impl From<ScoreState> for TaikoScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            n_misses: state.n_misses,
        }
    }
}

impl From<ScoreState> for CatchScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n_fruits: state.n300,
            n_droplets: state.n100,
            n_tiny_droplets: state.n50,
            n_tiny_droplet_misses: state.n_katu,
            n_misses: state.n_misses,
        }
    }
}

impl From<ScoreState> for ManiaScoreState {
    #[inline]
    fn from(state: ScoreState) -> Self {
        Self {
            n320: state.n_geki,
            n300: state.n300,
            n200: state.n_katu,
            n100: state.n100,
            n50: state.n50,
            n_misses: state.n_misses,
        }
    }
}

impl From<OsuScoreState> for ScoreState {
    #[inline]
    fn from(state: OsuScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n_geki: 0,
            n_katu: 0,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            n_misses: state.n_misses,
        }
    }
}

impl From<TaikoScoreState> for ScoreState {
    #[inline]
    fn from(state: TaikoScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n_geki: 0,
            n_katu: 0,
            n300: state.n300,
            n100: state.n100,
            n50: 0,
            n_misses: state.n_misses,
        }
    }
}

impl From<CatchScoreState> for ScoreState {
    #[inline]
    fn from(state: CatchScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n_geki: 0,
            n_katu: state.n_tiny_droplet_misses,
            n300: state.n_fruits,
            n100: state.n_droplets,
            n50: state.n_tiny_droplets,
            n_misses: state.n_misses,
        }
    }
}

impl From<ManiaScoreState> for ScoreState {
    #[inline]
    fn from(state: ManiaScoreState) -> Self {
        Self {
            max_combo: 0,
            n_geki: state.n320,
            n_katu: state.n200,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            n_misses: state.n_misses,
        }
    }
}
