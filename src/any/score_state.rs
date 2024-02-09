use rosu_map::section::general::GameMode;

use crate::{
    catch::CatchScoreState, mania::ManiaScoreState, osu::OsuScoreState, taiko::TaikoScoreState,
};

/// Aggregation for a score's current state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScoreState {
    /// Maximum combo that the score has had so far. **Not** the maximum
    /// possible combo of the map so far.
    ///
    /// Note that for osu!catch only fruits and droplets are considered for
    /// combo.
    ///
    /// Irrelevant for osu!mania.
    pub max_combo: u32,
    /// Amount of current gekis (n320 for osu!mania).
    pub n_geki: u32,
    /// Amount of current katus (tiny droplet misses for osu!catch / n200 for
    /// osu!mania).
    pub n_katu: u32,
    /// Amount of current 300s (fruits for osu!catch).
    pub n300: u32,
    /// Amount of current 100s (droplets for osu!catch).
    pub n100: u32,
    /// Amount of current 50s (tiny droplets for osu!catch).
    pub n50: u32,
    /// Amount of current misses (fruits + droplets for osu!catch).
    pub n_misses: u32,
}

impl ScoreState {
    /// Create a new empty score state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up based on the
    /// mode.
    pub fn total_hits(&self, mode: GameMode) -> u32 {
        let mut amount = self.n300 + self.n100 + self.n_misses;

        if mode != GameMode::Taiko {
            amount += self.n50;

            if mode != GameMode::Osu {
                amount += self.n_katu;
                amount += u32::from(mode != GameMode::Catch) * self.n_geki;
            }
        }

        amount
    }
}

impl From<ScoreState> for OsuScoreState {
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
