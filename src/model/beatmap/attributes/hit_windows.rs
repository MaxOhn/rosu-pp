use crate::model::beatmap::attributes::ext::BeatmapAttributesExt;

/// AR and OD hit windows
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct HitWindows {
    /// Hit window for approach rate i.e. `TimePreempt` in milliseconds.
    ///
    /// Only available for osu!standard and osu!catch.
    pub ar: Option<f64>,
    /// Perfect hit window for overall difficulty i.e. time to hit a "Perfect"
    /// in milliseconds.
    ///
    /// Only available for osu!mania.
    pub od_perfect: Option<f64>,
    /// Great hit window for overall difficulty i.e. time to hit a 300 ("Great")
    /// in milliseconds.
    ///
    /// Only available for osu!standard, osu!taiko, and osu!mania.
    pub od_great: Option<f64>,
    /// Good hit window for overall difficulty i.e. time to hit a "Good" in
    /// milliseconds.
    ///
    /// Only available for osu!mania.
    pub od_good: Option<f64>,
    /// Ok hit window for overall difficulty i.e. time to hit a 100 ("Ok") in
    /// milliseconds.
    ///
    /// Only available for osu!standard, osu!taiko, and osu!mania.
    pub od_ok: Option<f64>,
    /// Meh hit window for overall difficulty i.e. time to hit a 50 ("Meh") in
    /// milliseconds.
    ///
    /// Only available for osu!standard and osu!mania.
    pub od_meh: Option<f64>,
}

pub(super) struct GameModeHitWindows {
    pub min: f64,
    pub mid: f64,
    pub max: f64,
}

impl GameModeHitWindows {
    pub fn difficulty_range(&self, difficulty: f64) -> f64 {
        let Self { min, mid, max } = *self;

        BeatmapAttributesExt::difficulty_range(difficulty, min, mid, max)
    }

    pub fn inverse_difficulty_range(&self, difficulty_value: f64) -> f64 {
        let Self { min, mid, max } = *self;

        BeatmapAttributesExt::inverse_difficulty_range(difficulty_value, min, mid, max)
    }
}

pub mod osu {
    use super::GameModeHitWindows;

    pub const GREAT: GameModeHitWindows = GameModeHitWindows {
        min: 80.0,
        mid: 50.0,
        max: 20.0,
    };

    pub const OK: GameModeHitWindows = GameModeHitWindows {
        min: 140.0,
        mid: 100.0,
        max: 60.0,
    };

    pub const MEH: GameModeHitWindows = GameModeHitWindows {
        min: 200.0,
        mid: 150.0,
        max: 100.0,
    };
}

pub mod taiko {
    use super::GameModeHitWindows;

    pub const GREAT: GameModeHitWindows = GameModeHitWindows {
        min: 50.0,
        mid: 35.0,
        max: 20.0,
    };

    pub const OK: GameModeHitWindows = GameModeHitWindows {
        min: 120.0,
        mid: 80.0,
        max: 50.0,
    };
}

pub mod mania {
    use super::GameModeHitWindows;

    pub const PERFECT: GameModeHitWindows = GameModeHitWindows {
        min: 22.4,
        mid: 19.4,
        max: 13.9,
    };

    pub const GREAT: GameModeHitWindows = GameModeHitWindows {
        min: 64.0,
        mid: 49.0,
        max: 34.0,
    };

    pub const GOOD: GameModeHitWindows = GameModeHitWindows {
        min: 97.0,
        mid: 82.0,
        max: 67.0,
    };

    pub const OK: GameModeHitWindows = GameModeHitWindows {
        min: 127.0,
        mid: 112.0,
        max: 97.0,
    };

    pub const MEH: GameModeHitWindows = GameModeHitWindows {
        min: 151.0,
        mid: 136.0,
        max: 121.0,
    };
}

// Same in both Osu and Catch
pub const AR: GameModeHitWindows = GameModeHitWindows {
    min: 1800.0,
    mid: 1200.0,
    max: 450.0,
};
