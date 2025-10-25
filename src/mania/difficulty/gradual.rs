use std::cmp;

use rosu_map::section::general::GameMode;

use crate::{
    Beatmap, Difficulty,
    any::difficulty::skills::StrainSkill,
    mania::{convert, object::ObjectParams},
    model::{hit_object::HitObject, mode::ConvertError},
};

use super::{
    DIFFICULTY_MULTIPLIER, DifficultyValues, ManiaDifficultyAttributes, ManiaObject,
    object::ManiaDifficultyObject, skills::strain::Strain,
};

/// Gradually calculate the difficulty attributes of an osu!mania map.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`], the map's next hit object will
/// be processed and the [`ManiaDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`ManiaGradualPerformance`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::mania::ManiaGradualDifficulty;
///
/// let map = Beatmap::from_path("./resources/1638954.osu").unwrap();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = ManiaGradualDifficulty::new(difficulty, &map).unwrap();
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
/// [`ManiaGradualPerformance`]: crate::mania::ManiaGradualPerformance
pub struct ManiaGradualDifficulty {
    pub(crate) idx: usize,
    pub(crate) difficulty: Difficulty,
    objects_is_circle: Box<[bool]>,
    is_convert: bool,
    strain: Strain,
    diff_objects: Box<[ManiaDifficultyObject]>,
    note_state: NoteState,
}

#[derive(Default)]
struct NoteState {
    curr_combo: u32,
    n_hold_notes: u32,
}

impl ManiaGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!mania maps.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Result<Self, ConvertError> {
        let mut map = map.convert_ref(GameMode::Mania, difficulty.get_mods())?;

        if difficulty.get_mods().ho() {
            convert::apply_hold_off_to_beatmap(map.to_mut());
        }

        if difficulty.get_mods().invert() {
            convert::apply_invert_to_beatmap(map.to_mut());
        }

        if let Some(seed) = difficulty.get_mods().random_seed() {
            convert::apply_random_to_beatmap(map.to_mut(), seed);
        }

        let take = difficulty.get_passed_objects();
        let total_columns = map.cs.round_ties_even().max(1.0);
        let clock_rate = difficulty.get_clock_rate();
        let mut params = ObjectParams::new(&map);

        let mania_objects = map
            .hit_objects
            .iter()
            .map(|h| ManiaObject::new(h, total_columns, &mut params))
            .take(take);

        let diff_objects = DifficultyValues::create_difficulty_objects(clock_rate, mania_objects);

        let strain = Strain::new(total_columns as usize);

        let mut note_state = NoteState::default();

        let objects_is_circle: Box<[_]> =
            map.hit_objects.iter().map(HitObject::is_circle).collect();

        if let Some(h) = map.hit_objects.first() {
            let hit_object = ManiaObject::new(h, total_columns, &mut params);

            increment_combo_raw(
                objects_is_circle[0],
                hit_object.start_time,
                hit_object.end_time,
                &mut note_state,
            );
        }

        Ok(Self {
            idx: 0,
            difficulty,
            objects_is_circle,
            is_convert: map.is_convert,
            strain,
            diff_objects,
            note_state,
        })
    }
}

impl Iterator for ManiaGradualDifficulty {
    type Item = ManiaDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        // The first difficulty object belongs to the second note since each
        // difficulty object requires the current and the last note. Hence, if
        // we're still on the first object, we don't have a difficulty object
        // yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;
            self.strain.process(curr, &self.diff_objects);

            let is_circle = self.objects_is_circle[self.idx];
            increment_combo(
                is_circle,
                curr,
                &mut self.note_state,
                self.difficulty.get_clock_rate(),
            );
        } else if self.objects_is_circle.is_empty() {
            return None;
        }

        self.idx += 1;

        Some(ManiaDifficultyAttributes {
            stars: self.strain.cloned_difficulty_value() * DIFFICULTY_MULTIPLIER,
            max_combo: self.note_state.curr_combo,
            n_objects: self.idx as u32,
            n_hold_notes: self.note_state.n_hold_notes,
            is_convert: self.is_convert,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let skip_iter = self
            .diff_objects
            .iter()
            .zip(self.objects_is_circle.iter().skip(1))
            .skip(self.idx.saturating_sub(1));

        let mut take = cmp::min(n, self.len().saturating_sub(1));

        // The first note has no difficulty object
        if self.idx == 0 && take > 0 {
            take -= 1;
            self.idx += 1;
        }

        let clock_rate = self.difficulty.get_clock_rate();

        for (curr, is_circle) in skip_iter.take(take) {
            increment_combo(*is_circle, curr, &mut self.note_state, clock_rate);
            self.strain.process(curr, &self.diff_objects);
            self.idx += 1;
        }

        self.next()
    }
}

impl ExactSizeIterator for ManiaGradualDifficulty {
    fn len(&self) -> usize {
        self.diff_objects.len() + 1 - self.idx
    }
}

fn increment_combo(
    is_circle: bool,
    diff_obj: &ManiaDifficultyObject,
    state: &mut NoteState,
    clock_rate: f64,
) {
    increment_combo_raw(
        is_circle,
        diff_obj.start_time * clock_rate,
        diff_obj.end_time * clock_rate,
        state,
    );
}

fn increment_combo_raw(is_circle: bool, start_time: f64, end_time: f64, state: &mut NoteState) {
    if is_circle {
        state.curr_combo += 1;
    } else {
        state.curr_combo += 1 + ((end_time - start_time) / 100.0) as u32;
        state.n_hold_notes += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::{Beatmap, mania::Mania};

    use super::*;

    #[test]
    fn empty() {
        let map = Beatmap::from_bytes(&[]).unwrap();
        let mut gradual = ManiaGradualDifficulty::new(Difficulty::new(), &map).unwrap();
        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let map = Beatmap::from_path("./resources/1638954.osu").unwrap();

        let difficulty = Difficulty::new();

        let mut gradual = ManiaGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_2nd = ManiaGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_3rd = ManiaGradualDifficulty::new(difficulty.clone(), &map).unwrap();

        let hit_objects_len = map.hit_objects.len();

        for i in 1.. {
            let Some(next_gradual) = gradual.next() else {
                assert_eq!(i, hit_objects_len + 1);
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
                .calculate_for_mode::<Mania>(&map)
                .unwrap();

            assert_eq!(next_gradual, expected);
        }
    }
}
