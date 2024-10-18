use rosu_map::section::general::GameMode;

use crate::{
    catch::CatchScoreState, mania::ManiaScoreState, osu::OsuScoreState, taiko::TaikoScoreState,
};

/// Aggregation for a score's current state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreState {
    /// Maximum combo that the score has had so far. **Not** the maximum
    /// possible combo of the map so far.
    ///
    /// Note that for osu!catch only fruits and droplets are considered for
    /// combo.
    ///
    /// Irrelevant for osu!mania.
    pub max_combo: u32,
    /// Amount of successfully hit slider ticks and repeats.
    ///
    /// Only relevant for osu!standard in lazer.
    pub slider_tick_hits: u32,
    /// Amount of successfully hit slider ends.
    ///
    /// Only relevant for osu!standard in lazer.
    pub slider_end_hits: u32,
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
    pub misses: u32,
}

impl ScoreState {
    /// Create a new empty score state.
    pub const fn new() -> Self {
        Self {
            max_combo: 0,
            slider_tick_hits: 0,
            slider_end_hits: 0,
            n_geki: 0,
            n_katu: 0,
            n300: 0,
            n100: 0,
            n50: 0,
            misses: 0,
        }
    }

    /// Return the total amount of hits by adding everything up based on the
    /// mode.
    pub fn total_hits(&self, mode: GameMode) -> u32 {
        let mut amount = self.n300 + self.n100 + self.misses;

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
            slider_tick_hits: state.slider_tick_hits,
            slider_end_hits: state.slider_end_hits,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            misses: state.misses,
        }
    }
}

impl From<ScoreState> for TaikoScoreState {
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            n300: state.n300,
            n100: state.n100,
            misses: state.misses,
        }
    }
}

impl From<ScoreState> for CatchScoreState {
    fn from(state: ScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            fruits: state.n300,
            droplets: state.n100,
            tiny_droplets: state.n50,
            tiny_droplet_misses: state.n_katu,
            misses: state.misses,
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
            misses: state.misses,
        }
    }
}

impl From<OsuScoreState> for ScoreState {
    fn from(state: OsuScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            slider_tick_hits: state.slider_tick_hits,
            slider_end_hits: state.slider_end_hits,
            n_geki: 0,
            n_katu: 0,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            misses: state.misses,
        }
    }
}

impl From<TaikoScoreState> for ScoreState {
    fn from(state: TaikoScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            slider_tick_hits: 0,
            slider_end_hits: 0,
            n_geki: 0,
            n_katu: 0,
            n300: state.n300,
            n100: state.n100,
            n50: 0,
            misses: state.misses,
        }
    }
}

impl From<CatchScoreState> for ScoreState {
    fn from(state: CatchScoreState) -> Self {
        Self {
            max_combo: state.max_combo,
            slider_tick_hits: 0,
            slider_end_hits: 0,
            n_geki: 0,
            n_katu: state.tiny_droplet_misses,
            n300: state.fruits,
            n100: state.droplets,
            n50: state.tiny_droplets,
            misses: state.misses,
        }
    }
}

impl From<ManiaScoreState> for ScoreState {
    fn from(state: ManiaScoreState) -> Self {
        Self {
            max_combo: 0,
            slider_tick_hits: 0,
            slider_end_hits: 0,
            n_geki: state.n320,
            n_katu: state.n200,
            n300: state.n300,
            n100: state.n100,
            n50: state.n50,
            misses: state.misses,
        }
    }
}

impl Default for ScoreState {
    fn default() -> Self {
        Self::new()
    }
}
