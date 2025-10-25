use std::{cmp, mem, slice::Iter};

use rosu_map::section::general::GameMode;

use crate::{
    Beatmap, Difficulty,
    any::difficulty::skills::StrainSkill,
    model::{beatmap::HitWindows, hit_object::HitObject, mode::ConvertError},
    taiko::convert,
    util::sync::RefCount,
};

use super::{
    DifficultyValues, TaikoDifficultyAttributes,
    object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    skills::TaikoSkills,
};

/// Gradually calculate the difficulty attributes of an osu!taiko map.
///
/// Note that this struct implements [`Iterator`]. On every call of
/// [`Iterator::next`], the map's next hit object will be processed and the
/// [`TaikoDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`TaikoGradualPerformance`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::taiko::{Taiko, TaikoGradualDifficulty};
///
/// let map = Beatmap::from_path("./resources/1028484.osu").unwrap();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = TaikoGradualDifficulty::new(difficulty, &map).unwrap();
///
/// // the difficulty of the map after the first hit object
/// let attrs1 = iter.next();
/// // ... after the second hit object
/// let attrs2 = iter.next();
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
///
/// [`TaikoGradualPerformance`]: crate::taiko::TaikoGradualPerformance
pub struct TaikoGradualDifficulty {
    pub(crate) idx: usize,
    pub(crate) difficulty: Difficulty,
    attrs: TaikoDifficultyAttributes,
    diff_objects: TaikoDifficultyObjects,
    diff_objects_iter: Iter<'static, RefCount<TaikoDifficultyObject>>,
    skills: TaikoSkills,
    total_hits: usize,
    first_combos: FirstTwoCombos,
}

#[derive(Copy, Clone, Debug)]
enum FirstTwoCombos {
    None,
    OnlyFirst,
    OnlySecond,
    Both,
}

impl TaikoGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!taiko maps.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Result<Self, ConvertError> {
        let mut map = map.convert_ref(GameMode::Taiko, difficulty.get_mods())?;

        if let Some(seed) = difficulty.get_mods().random_seed() {
            convert::apply_random_to_beatmap(map.to_mut(), seed);
        }

        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let first_combos = match (
            map.hit_objects.first().map(HitObject::is_circle),
            map.hit_objects.get(1).map(HitObject::is_circle),
        ) {
            (None, _) | (Some(false), Some(false) | None) => FirstTwoCombos::None,
            (Some(true), Some(false) | None) => FirstTwoCombos::OnlyFirst,
            (Some(false), Some(true)) => FirstTwoCombos::OnlySecond,
            (Some(true), Some(true)) => FirstTwoCombos::Both,
        };

        let HitWindows {
            od_great,
            od_ok,
            od_meh: _,
            ar: _,
        } = map.attributes().difficulty(&difficulty).hit_windows();

        let mut n_diff_objects = 0;
        let mut max_combo = 0;

        let diff_objects = DifficultyValues::create_difficulty_objects(
            &map,
            take as u32,
            clock_rate,
            &mut max_combo,
            &mut n_diff_objects,
            difficulty.get_mods(),
        );

        let skills = TaikoSkills::new(od_great, map.is_convert);

        let attrs = TaikoDifficultyAttributes {
            great_hit_window: od_great,
            ok_hit_window: od_ok.unwrap_or(0.0),
            is_convert: map.is_convert,
            ..Default::default()
        };

        let total_hits = map.hit_objects.iter().filter(|h| h.is_circle()).count();

        let diff_objects_iter = extend_lifetime(diff_objects.iter());

        Ok(Self {
            idx: 0,
            difficulty,
            diff_objects,
            diff_objects_iter,
            skills,
            attrs,
            total_hits,
            first_combos,
        })
    }
}

fn extend_lifetime(
    iter: Iter<'_, RefCount<TaikoDifficultyObject>>,
) -> Iter<'static, RefCount<TaikoDifficultyObject>> {
    // SAFETY: The underlying data will never be moved.
    unsafe { mem::transmute(iter) }
}

impl Iterator for TaikoGradualDifficulty {
    type Item = TaikoDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        // The first difficulty object belongs to the third note since each
        // difficulty object requires the current, the last, and the second to
        // last note. Hence, if we're still on the first or second object, we
        // don't have a difficulty object yet and just skip processing.
        if self.idx >= 2 {
            loop {
                let curr = self.diff_objects_iter.next()?;
                let borrowed = curr.get();

                self.skills.rhythm.process(&borrowed, &self.diff_objects);
                self.skills.reading.process(&borrowed, &self.diff_objects);
                self.skills.color.process(&borrowed, &self.diff_objects);
                self.skills.stamina.process(&borrowed, &self.diff_objects);
                self.skills
                    .single_color_stamina
                    .process(&borrowed, &self.diff_objects);

                if borrowed.base_hit_type.is_hit() {
                    self.attrs.max_combo += 1;

                    break;
                }
            }
        } else if self.diff_objects.is_empty() {
            return None;
        } else {
            match self.first_combos {
                FirstTwoCombos::OnlyFirst => self.attrs.max_combo = 1,
                FirstTwoCombos::OnlySecond if self.idx == 1 => self.attrs.max_combo = 1,
                FirstTwoCombos::Both if self.idx == 0 => self.attrs.max_combo = 1,
                FirstTwoCombos::Both if self.idx == 1 => self.attrs.max_combo = 2,
                _ => {}
            }
        }

        self.idx += 1;

        let mut attrs = self.attrs.clone();
        let is_relax = self.difficulty.get_mods().rx();

        DifficultyValues::eval(&mut attrs, self.skills.clone(), is_relax);

        Some(attrs)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let mut take = cmp::min(n, self.len().saturating_sub(1));

        // The first two notes have no difficulty object but might add to combo
        match (take, self.idx) {
            (_, 2..) | (0, _) => {}
            (1, 0) => {
                take -= 1;
                self.idx += 1;

                match self.first_combos {
                    FirstTwoCombos::None => {}
                    FirstTwoCombos::OnlyFirst => self.attrs.max_combo = 1,
                    FirstTwoCombos::OnlySecond => {}
                    FirstTwoCombos::Both => self.attrs.max_combo = 1,
                }
            }
            (_, 0) => {
                take -= 2;
                self.idx += 2;

                match self.first_combos {
                    FirstTwoCombos::None => {}
                    FirstTwoCombos::OnlyFirst => self.attrs.max_combo = 1,
                    FirstTwoCombos::OnlySecond => self.attrs.max_combo = 1,
                    FirstTwoCombos::Both => self.attrs.max_combo = 2,
                }
            }
            (_, 1) => {
                take -= 1;
                self.idx += 1;

                match self.first_combos {
                    FirstTwoCombos::None => {}
                    FirstTwoCombos::OnlyFirst => self.attrs.max_combo = 1,
                    FirstTwoCombos::OnlySecond => self.attrs.max_combo = 1,
                    FirstTwoCombos::Both => self.attrs.max_combo = 2,
                }
            }
        }

        for _ in 0..take {
            loop {
                let curr = self.diff_objects_iter.next()?;
                let borrowed = curr.get();
                self.skills.rhythm.process(&borrowed, &self.diff_objects);
                self.skills.reading.process(&borrowed, &self.diff_objects);
                self.skills.color.process(&borrowed, &self.diff_objects);
                self.skills.stamina.process(&borrowed, &self.diff_objects);
                self.skills
                    .single_color_stamina
                    .process(&borrowed, &self.diff_objects);

                if borrowed.base_hit_type.is_hit() {
                    self.attrs.max_combo += 1;
                    self.idx += 1;

                    break;
                }
            }
        }

        self.next()
    }
}

impl ExactSizeIterator for TaikoGradualDifficulty {
    fn len(&self) -> usize {
        self.total_hits - self.idx
    }
}

#[cfg(test)]
mod tests {
    use crate::{Beatmap, taiko::Taiko};

    use super::*;

    #[test]
    fn empty() {
        let map = Beatmap::from_bytes(&[]).unwrap();
        let mut gradual = TaikoGradualDifficulty::new(Difficulty::new(), &map).unwrap();
        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let map = Beatmap::from_path("./resources/1028484.osu").unwrap();

        let difficulty = Difficulty::new();

        let mut gradual = TaikoGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_2nd = TaikoGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_3rd = TaikoGradualDifficulty::new(difficulty.clone(), &map).unwrap();

        let hit_objects_len = map.hit_objects.len();

        let n_hits = map.hit_objects.iter().filter(|h| h.is_circle()).count();

        for i in 1.. {
            let Some(next_gradual) = gradual.next() else {
                assert_eq!(i, n_hits + 1);
                assert!(gradual_2nd.last().is_some() || hit_objects_len % 2 == 0);
                assert!(gradual_3rd.last().is_some() || hit_objects_len % 3 == 0);
                break;
            };

            if i % 2 == 0 {
                let next_gradual_2nd = gradual_2nd.nth(1).unwrap();
                assert_eq!(next_gradual, next_gradual_2nd);
            }

            if i % 3 == 0 {
                let next_gradual_3rd = gradual_3rd.nth(2).unwrap();
                assert_eq!(next_gradual, next_gradual_3rd);
            }

            let expected = difficulty
                .clone()
                .passed_objects(i as u32)
                .calculate_for_mode::<Taiko>(&map)
                .unwrap();

            assert_eq!(next_gradual, expected);
        }
    }
}
