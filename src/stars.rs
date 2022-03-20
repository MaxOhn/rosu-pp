use crate::{
    Beatmap, DifficultyAttributes, FruitsStars, GameMode, ManiaStars, OsuStars, Strains, TaikoStars,
};

/// Difficulty calculator on maps of any mode.
///
/// # Example
///
/// ```
/// use rosu_pp::{AnyStars, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let difficulty_attrs = AnyStars::new(&map)
///     .mods(8 + 64) // HDDT
///     .calculate();
///
/// println!("Stars: {}", difficulty_attrs.stars());
/// ```
#[derive(Clone, Debug)]
pub enum AnyStars<'map> {
    /// osu!catch difficulty calculator
    Fruits(FruitsStars<'map>),
    /// osu!mania difficulty calculator
    Mania(ManiaStars<'map>),
    /// osu!standard difficulty calculator
    Osu(OsuStars<'map>),
    /// osu!taiko difficulty calculator
    Taiko(TaikoStars<'map>),
}

impl<'map> AnyStars<'map> {
    /// Create a new difficulty calculator for maps of any mode.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        match map.mode {
            GameMode::CTB => Self::Fruits(FruitsStars::new(map)),
            GameMode::MNA => Self::Mania(ManiaStars::new(map)),
            GameMode::STD => Self::Osu(OsuStars::new(map)),
            GameMode::TKO => Self::Taiko(TaikoStars::new(map)),
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(self, mods: u32) -> Self {
        match self {
            Self::Fruits(f) => Self::Fruits(f.mods(mods)),
            Self::Mania(m) => Self::Mania(m.mods(mods)),
            Self::Osu(o) => Self::Osu(o.mods(mods)),
            Self::Taiko(t) => Self::Taiko(t.mods(mods)),
        }
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects, instead of
    /// using [`AnyStars`] multiple times with different `passed_objects`, you should use
    /// [`GradualDifficultyAttributes`](crate::GradualDifficultyAttributes).
    #[inline]
    pub fn passed_objects(self, passed_objects: usize) -> Self {
        match self {
            Self::Fruits(f) => Self::Fruits(f.passed_objects(passed_objects)),
            Self::Mania(m) => Self::Mania(m.passed_objects(passed_objects)),
            Self::Osu(o) => Self::Osu(o.passed_objects(passed_objects)),
            Self::Taiko(t) => Self::Taiko(t.passed_objects(passed_objects)),
        }
    }

    /// Consume the difficulty calculator and calculate
    /// difficulty attributes for the given parameters.
    #[inline]
    pub fn calculate(self) -> DifficultyAttributes {
        match self {
            Self::Fruits(f) => DifficultyAttributes::Fruits(f.calculate()),
            Self::Mania(m) => DifficultyAttributes::Mania(m.calculate()),
            Self::Osu(o) => DifficultyAttributes::Osu(o.calculate()),
            Self::Taiko(t) => DifficultyAttributes::Taiko(t.calculate()),
        }
    }

    /// Consume the difficulty calculator and calculate
    /// skill strains for the given parameters.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> Strains {
        match self {
            Self::Fruits(f) => f.strains(),
            Self::Mania(m) => m.strains(),
            Self::Osu(o) => o.strains(),
            Self::Taiko(t) => t.strains(),
        }
    }
}
