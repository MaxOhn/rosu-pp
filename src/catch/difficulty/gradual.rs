use std::cmp;

use rosu_map::section::general::GameMode;

use crate::{
    Beatmap, Difficulty,
    any::difficulty::skills::StrainSkill,
    catch::{
        CatchDifficultyAttributes,
        attributes::{GradualObjectCount, ObjectCountBuilder},
        catcher::Catcher,
        convert::convert_objects,
    },
    model::mode::ConvertError,
};

use super::{
    CatchDifficultySetup, DifficultyValues, object::CatchDifficultyObject,
    skills::movement::Movement,
};

/// Gradually calculate the difficulty attributes of an osu!catch map.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`], the map's next fruit or droplet
/// will be processed and the [`CatchDifficultyAttributes`] will be updated and
/// returned.
///
/// Note that it does not return attributes after a tiny droplet. Only for
/// fruits and droplets.
///
/// If you want to calculate performance attributes, use
/// [`CatchGradualPerformance`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::catch::{Catch, CatchGradualDifficulty};
///
/// let map = Beatmap::from_path("./resources/2118524.osu").unwrap();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = CatchGradualDifficulty::new(difficulty, &map).unwrap();
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
/// [`CatchGradualPerformance`]: crate::catch::CatchGradualPerformance
pub struct CatchGradualDifficulty {
    pub(crate) idx: usize,
    pub(crate) difficulty: Difficulty,
    attrs: CatchDifficultyAttributes,
    /// The delta of object counts after each palpable object
    count: Vec<GradualObjectCount>,
    diff_objects: Box<[CatchDifficultyObject]>,
    movement: Movement,
}

impl CatchGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!catch maps.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Result<Self, ConvertError> {
        let map = map.convert_ref(GameMode::Catch, difficulty.get_mods())?;

        let clock_rate = difficulty.get_clock_rate();

        let CatchDifficultySetup { map_attrs, attrs } =
            CatchDifficultySetup::new(&difficulty, &map);

        let hr_offsets = difficulty.get_hardrock_offsets();
        let reflection = difficulty.get_mods().reflection();
        let mut count = ObjectCountBuilder::new_gradual();

        let palpable_objects = convert_objects(
            &map,
            &mut count,
            reflection,
            hr_offsets,
            map_attrs.cs as f32,
        );

        let mut half_catcher_width = Catcher::calculate_catch_width(map_attrs.cs as f32) * 0.5;
        half_catcher_width *= 1.0 - ((map_attrs.cs as f32 - 5.5).max(0.0) * 0.0625);

        let diff_objects = DifficultyValues::create_difficulty_objects(
            clock_rate,
            half_catcher_width,
            palpable_objects.iter(),
        );

        let count = count.into_gradual();
        let movement = Movement::new(clock_rate);

        Ok(Self {
            idx: 0,
            difficulty,
            attrs,
            count,
            diff_objects,
            movement,
        })
    }
}

impl Iterator for CatchGradualDifficulty {
    type Item = CatchDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        // The first difficulty object belongs to the second palpable object
        // since each difficulty object requires the current and the last note.
        // Hence, if we're still on the first object, we don't have a difficulty
        // object yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;
            self.movement.process(curr, &self.diff_objects);
        } else if self.count.is_empty() {
            return None;
        }

        self.attrs.add_object_count(self.count[self.idx]);
        self.idx += 1;

        let mut attrs = self.attrs.clone();

        let movement = self.movement.cloned_difficulty_value();
        DifficultyValues::eval(&mut attrs, movement);

        Some(attrs)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let skip_iter = self.diff_objects.iter().skip(self.idx.saturating_sub(1));

        let mut take = cmp::min(n, self.len().saturating_sub(1));

        // The first palpable object has no difficulty object
        if self.idx == 0 && take > 0 {
            take -= 1;
            self.attrs.add_object_count(self.count[self.idx]);
            self.idx += 1;
        }

        for curr in skip_iter.take(take) {
            self.movement.process(curr, &self.diff_objects);

            self.attrs.add_object_count(self.count[self.idx]);
            self.idx += 1;
        }

        self.next()
    }
}

impl ExactSizeIterator for CatchGradualDifficulty {
    fn len(&self) -> usize {
        self.diff_objects.len() + 1 - self.idx
    }
}

#[cfg(test)]
mod tests {
    use crate::{Beatmap, catch::Catch};

    use super::*;

    #[test]
    fn empty() {
        let map = Beatmap::from_bytes(&[]).unwrap();
        let mut gradual = CatchGradualDifficulty::new(Difficulty::new(), &map).unwrap();
        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let map = Beatmap::from_path("./resources/2118524.osu").unwrap();

        let difficulty = Difficulty::new();

        let mut gradual = CatchGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_2nd = CatchGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_3rd = CatchGradualDifficulty::new(difficulty.clone(), &map).unwrap();

        for i in 1.. {
            let Some(next_gradual) = gradual.next() else {
                assert_eq!(i, 731);
                assert!(gradual_2nd.last().is_none()); // 730 % 2 == 0
                assert!(gradual_3rd.last().is_some()); // 730 % 3 == 1
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
                .calculate_for_mode::<Catch>(&map)
                .unwrap();

            assert_eq!(next_gradual, expected);
        }
    }
}
