use crate::{Difficulty, model::mods::GameMods};

use super::ModsDependent;

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
    pub ar: Option<ModsDependent>,
    /// Override a beatmap's set CS.
    ///
    /// Only relevant for osu! and osu!catch.
    pub cs: Option<ModsDependent>,
    /// Override a beatmap's set HP.
    pub hp: Option<ModsDependent>,
    /// Override a beatmap's set OD.
    pub od: Option<ModsDependent>,
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

        if let Some(ar) = ar {
            difficulty = difficulty.ar(ar.value, ar.with_mods);
        }

        if let Some(cs) = cs {
            difficulty = difficulty.cs(cs.value, cs.with_mods);
        }

        if let Some(hp) = hp {
            difficulty = difficulty.hp(hp.value, hp.with_mods);
        }

        if let Some(od) = od {
            difficulty = difficulty.od(od.value, od.with_mods);
        }

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
