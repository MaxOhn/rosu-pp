use std::cmp;

use crate::{
    any::difficulty::skills::Skill,
    catch::{
        attributes::{GradualObjectCount, ObjectCountBuilder},
        convert::convert_objects,
        CatchBeatmap, CatchDifficultyAttributes,
    },
    Difficulty,
};

use super::{
    object::CatchDifficultyObject, skills::movement::Movement, CatchDifficultySetup,
    DifficultyValues,
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
/// let converted = Beatmap::from_path("./resources/2118524.osu")
///     .unwrap()
///     .unchecked_into_converted::<Catch>();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = CatchGradualDifficulty::new(difficulty, &converted);
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
    pub fn new(difficulty: Difficulty, converted: &CatchBeatmap<'_>) -> Self {
        let clock_rate = difficulty.get_clock_rate();

        let CatchDifficultySetup { map_attrs, attrs } =
            CatchDifficultySetup::new(&difficulty, converted);

        let hr_offsets = difficulty.get_hardrock_offsets();
        let mut count = ObjectCountBuilder::new_gradual();
        let palpable_objects =
            convert_objects(converted, &mut count, hr_offsets, map_attrs.cs as f32);

        let diff_objects = DifficultyValues::create_difficulty_objects(
            &map_attrs,
            clock_rate,
            palpable_objects.iter(),
        );

        let count = count.into_gradual();
        let movement = Movement::new(clock_rate);

        Self {
            idx: 0,
            difficulty,
            attrs,
            count,
            diff_objects,
            movement,
        }
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
            Skill::new(&mut self.movement, &self.diff_objects).process(curr);
        } else if self.count.is_empty() {
            return None;
        }

        self.attrs.add_object_count(self.count[self.idx]);
        self.idx += 1;

        let mut attrs = self.attrs.clone();

        let movement = self.movement.as_difficulty_value();
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

        let mut movement = Skill::new(&mut self.movement, &self.diff_objects);

        for curr in skip_iter.take(take) {
            movement.process(curr);

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
    use crate::Beatmap;

    use super::*;

    #[test]
    fn empty() {
        let converted = Beatmap::from_bytes(&[]).unwrap().unchecked_into_converted();

        let mut gradual = CatchGradualDifficulty::new(Difficulty::new(), &converted);

        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let converted = Beatmap::from_path("./resources/2118524.osu")
            .unwrap()
            .unchecked_into_converted();

        let difficulty = Difficulty::new();

        let mut gradual = CatchGradualDifficulty::new(difficulty.clone(), &converted);
        let mut gradual_2nd = CatchGradualDifficulty::new(difficulty.clone(), &converted);
        let mut gradual_3rd = CatchGradualDifficulty::new(difficulty.clone(), &converted);

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
                .with_mode()
                .calculate(&converted);

            assert_eq!(next_gradual, expected);
        }
    }
}
