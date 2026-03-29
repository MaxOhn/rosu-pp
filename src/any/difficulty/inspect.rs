use crate::{
    Difficulty,
    model::{beatmap::BeatmapAttribute, mods::GameMods},
};

/// [`Difficulty`] but all fields are public for inspection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectDifficulty {
    /// Specify mods.
    pub mods: GameMods,
    /// Amount of passed objects for partial plays, e.g. a fail.
    pub passed_objects: Option<u32>,
    /// Adjust the clock rate used in the calculation.
    pub clock_rate: Option<f64>,
    /// Override a beatmap's set AR.
    ///
    /// Only relevant for osu! and osu!catch.
    pub ar: BeatmapAttribute,
    /// Override a beatmap's set CS.
    ///
    /// Only relevant for osu! and osu!catch.
    pub cs: BeatmapAttribute,
    /// Override a beatmap's set HP.
    pub hp: BeatmapAttribute,
    /// Override a beatmap's set OD.
    pub od: BeatmapAttribute,
    /// Adjust patterns as if the HR mod is enabled.
    ///
    /// Only relevant for osu!catch.
    pub hardrock_offsets: Option<bool>,
    /// Whether the calculated attributes belong to an osu!lazer or osu!stable
    /// score.
    ///
    /// Defaults to `true`.
    pub lazer: Option<bool>,
}

impl InspectDifficulty {
    /// Convert `self` into a [`Difficulty`].
    pub fn into_difficulty(self) -> Difficulty {
        let Self {
            mods,
            passed_objects,
            clock_rate,
            ar,
            cs,
            hp,
            od,
            hardrock_offsets,
            lazer,
        } = self;

        let mut difficulty = Difficulty::new().mods(mods);

        if let Some(passed_objects) = passed_objects {
            difficulty = difficulty.passed_objects(passed_objects);
        }

        if let Some(clock_rate) = clock_rate {
            difficulty = difficulty.clock_rate(clock_rate);
        }

        macro_rules! set_attr {
            ( $attr:ident ) => {
                match $attr {
                    BeatmapAttribute::None | BeatmapAttribute::Value(_) => {}
                    BeatmapAttribute::Given(value) => difficulty = difficulty.$attr(value, false),
                    BeatmapAttribute::Fixed(value) => difficulty = difficulty.$attr(value, true),
                };
            };
        }

        set_attr!(ar);
        set_attr!(cs);
        set_attr!(hp);
        set_attr!(od);

        if let Some(hardrock_offsets) = hardrock_offsets {
            difficulty = difficulty.hardrock_offsets(hardrock_offsets);
        }

        if let Some(lazer) = lazer {
            difficulty = difficulty.lazer(lazer);
        }

        difficulty
    }
}

impl From<InspectDifficulty> for Difficulty {
    fn from(difficulty: InspectDifficulty) -> Self {
        difficulty.into_difficulty()
    }
}

impl From<Difficulty> for InspectDifficulty {
    fn from(difficulty: Difficulty) -> Self {
        difficulty.inspect()
    }
}
