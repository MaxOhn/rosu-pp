use std::borrow::Cow;

use rosu_map::section::general::GameMode;

use crate::{
    any::DifficultyAttributes,
    catch::{Catch, CatchBeatmap, CatchGradualDifficulty},
    mania::{Mania, ManiaBeatmap, ManiaGradualDifficulty},
    model::mode::IGameMode,
    osu::{Osu, OsuBeatmap, OsuGradualDifficulty},
    taiko::{Taiko, TaikoBeatmap, TaikoGradualDifficulty},
    Beatmap, Converted, Difficulty,
};

/// Gradually calculate the difficulty attributes on maps of any mode.
///
/// Note that this type implements [`Iterator`]. On every call of
/// [`Iterator::next`], the next object will be processed and the
/// [`DifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use [`GradualPerformance`] instead.
///
/// # Example
///
/// ```
/// use akatsuki_pp::{Beatmap, GradualDifficulty, Difficulty};
///
/// let map = Beatmap::from_path("./resources/2785319.osu").unwrap();
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = GradualDifficulty::new(difficulty, &map);
///
/// // the difficulty of the map after the first object
/// let attrs1 = iter.next();
/// // ... after the second object
/// let attrs2 = iter.next();
///
/// // Remaining objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
///
/// [`GradualPerformance`]: crate::GradualPerformance
// 504 vs 184 bytes is an acceptable difference and the Osu variant (424 bytes)
// is likely the most used one anyway.
#[allow(clippy::large_enum_variant)]
pub enum GradualDifficulty {
    Osu(OsuGradualDifficulty),
    Taiko(TaikoGradualDifficulty),
    Catch(CatchGradualDifficulty),
    Mania(ManiaGradualDifficulty),
}

macro_rules! from_converted {
    ( $fn:ident, $mode:ident, $converted:ident ) => {
        #[doc = concat!("Create a [`GradualDifficulty`] for a [`", stringify!($converted), "`]")]
        pub fn $fn(difficulty: Difficulty, converted: &$converted<'_>) -> Self {
            Self::$mode($mode::gradual_difficulty(difficulty, converted))
        }
    };
}

impl GradualDifficulty {
    /// Create a [`GradualDifficulty`] for a map of any mode.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Self {
        let map = Cow::Borrowed(map);

        match map.mode {
            GameMode::Osu => Self::Osu(Osu::gradual_difficulty(difficulty, &Converted::new(map))),
            GameMode::Taiko => {
                Self::Taiko(Taiko::gradual_difficulty(difficulty, &Converted::new(map)))
            }
            GameMode::Catch => {
                Self::Catch(Catch::gradual_difficulty(difficulty, &Converted::new(map)))
            }
            GameMode::Mania => {
                Self::Mania(Mania::gradual_difficulty(difficulty, &Converted::new(map)))
            }
        }
    }

    from_converted!(from_osu_map, Osu, OsuBeatmap);
    from_converted!(from_taiko_map, Taiko, TaikoBeatmap);
    from_converted!(from_catch_map, Catch, CatchBeatmap);
    from_converted!(from_mania_map, Mania, ManiaBeatmap);
}

impl Iterator for GradualDifficulty {
    type Item = DifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GradualDifficulty::Osu(gradual) => gradual.next().map(DifficultyAttributes::Osu),
            GradualDifficulty::Taiko(gradual) => gradual.next().map(DifficultyAttributes::Taiko),
            GradualDifficulty::Catch(gradual) => gradual.next().map(DifficultyAttributes::Catch),
            GradualDifficulty::Mania(gradual) => gradual.next().map(DifficultyAttributes::Mania),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GradualDifficulty::Osu(gradual) => gradual.size_hint(),
            GradualDifficulty::Taiko(gradual) => gradual.size_hint(),
            GradualDifficulty::Catch(gradual) => gradual.size_hint(),
            GradualDifficulty::Mania(gradual) => gradual.size_hint(),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            GradualDifficulty::Osu(gradual) => gradual.nth(n).map(DifficultyAttributes::Osu),
            GradualDifficulty::Taiko(gradual) => gradual.nth(n).map(DifficultyAttributes::Taiko),
            GradualDifficulty::Catch(gradual) => gradual.nth(n).map(DifficultyAttributes::Catch),
            GradualDifficulty::Mania(gradual) => gradual.nth(n).map(DifficultyAttributes::Mania),
        }
    }
}

impl ExactSizeIterator for GradualDifficulty {
    fn len(&self) -> usize {
        match self {
            GradualDifficulty::Osu(gradual) => gradual.len(),
            GradualDifficulty::Taiko(gradual) => gradual.len(),
            GradualDifficulty::Catch(gradual) => gradual.len(),
            GradualDifficulty::Mania(gradual) => gradual.len(),
        }
    }
}
