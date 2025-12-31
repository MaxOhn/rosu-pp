use rosu_map::section::general::GameMode;

use crate::{
    Beatmap, Difficulty,
    any::DifficultyAttributes,
    catch::{Catch, CatchGradualDifficulty},
    mania::{Mania, ManiaGradualDifficulty},
    model::mode::{ConvertError, IGameMode},
    osu::{Osu, OsuGradualDifficulty},
    taiko::{Taiko, TaikoGradualDifficulty},
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
/// use rosu_pp::{Beatmap, GradualDifficulty, Difficulty};
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
pub enum GradualDifficulty {
    Osu(OsuGradualDifficulty),
    Taiko(TaikoGradualDifficulty),
    Catch(CatchGradualDifficulty),
    Mania(ManiaGradualDifficulty),
}

impl GradualDifficulty {
    /// Create a [`GradualDifficulty`] for a map of any mode.
    #[expect(clippy::missing_panics_doc, reason = "unreachable")]
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Self {
        Self::new_with_mode(difficulty, map, map.mode).expect("no conversion required")
    }

    /// Create a [`GradualDifficulty`] for a [`Beatmap`] on a specific [`GameMode`].
    pub fn new_with_mode(
        difficulty: Difficulty,
        map: &Beatmap,
        mode: GameMode,
    ) -> Result<Self, ConvertError> {
        match mode {
            GameMode::Osu => Osu::gradual_difficulty(difficulty, map).map(Self::Osu),
            GameMode::Taiko => Taiko::gradual_difficulty(difficulty, map).map(Self::Taiko),
            GameMode::Catch => Catch::gradual_difficulty(difficulty, map).map(Self::Catch),
            GameMode::Mania => Mania::gradual_difficulty(difficulty, map).map(Self::Mania),
        }
    }
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
