use std::cmp;

use crate::{
    any::difficulty::skills::Skill,
    mania::{object::ObjectParams, ManiaBeatmap},
    model::{beatmap::HitWindows, hit_object::HitObject},
    util::float_ext::FloatExt,
    ModeDifficulty,
};

use super::{
    object::ManiaDifficultyObject, skills::strain::Strain, DifficultyValues,
    ManiaDifficultyAttributes, ManiaObject, STAR_SCALING_FACTOR,
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
/// use rosu_pp::{Beatmap, ModeDifficulty};
/// use rosu_pp::mania::ManiaGradualDifficulty;
///
/// let converted = Beatmap::from_path("./resources/1638954.osu")
///     .unwrap()
///     .unchecked_into_converted();
///
/// let difficulty = ModeDifficulty::new().mods(64); // DT
/// let mut iter = ManiaGradualDifficulty::new(&difficulty, &converted);
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
    pub(crate) mods: u32,
    pub(crate) clock_rate: f64,
    objects_is_circle: Box<[bool]>,
    is_convert: bool,
    strain: Strain,
    diff_objects: Box<[ManiaDifficultyObject]>,
    hit_window: f64,
    curr_combo: u32,
}

impl ManiaGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!mania maps.
    pub fn new(difficulty: &ModeDifficulty, converted: &ManiaBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let mods = difficulty.get_mods();
        let total_columns = converted.cs.round_even().max(1.0);
        let clock_rate = difficulty.get_clock_rate();
        let mut params = ObjectParams::new(converted);

        let HitWindows { od: hit_window, .. } = converted
            .attributes()
            .mods(mods)
            .clock_rate(clock_rate)
            .hit_windows();

        let mania_objects = converted
            .hit_objects
            .iter()
            .map(|h| ManiaObject::new(h, total_columns, &mut params))
            .take(take);

        let diff_objects = DifficultyValues::create_difficulty_objects(clock_rate, mania_objects);

        let strain = Strain::new(total_columns as usize);

        let mut curr_combo = 0;

        let objects_is_circle: Box<[_]> = converted
            .hit_objects
            .iter()
            .map(HitObject::is_circle)
            .collect();

        if let Some(h) = converted.hit_objects.first() {
            let hit_object = ManiaObject::new(h, total_columns, &mut params);

            increment_combo_raw(
                objects_is_circle[0],
                hit_object.start_time,
                hit_object.end_time,
                &mut curr_combo,
            );
        }

        Self {
            idx: 0,
            mods,
            clock_rate,
            objects_is_circle,
            is_convert: converted.is_convert,
            strain,
            diff_objects,
            hit_window,
            curr_combo,
        }
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
            Skill::new(&mut self.strain, &self.diff_objects).process(curr);

            let is_circle = self.objects_is_circle[self.idx];
            increment_combo(is_circle, curr, &mut self.curr_combo, self.clock_rate);
        } else if self.objects_is_circle.is_empty() {
            return None;
        }

        self.idx += 1;

        Some(ManiaDifficultyAttributes {
            stars: self.strain.as_difficulty_value() * STAR_SCALING_FACTOR,
            hit_window: self.hit_window,
            max_combo: self.curr_combo,
            n_objects: self.idx as u32,
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

        let mut strain = Skill::new(&mut self.strain, &self.diff_objects);

        for (curr, is_circle) in skip_iter.take(take) {
            increment_combo(*is_circle, curr, &mut self.curr_combo, self.clock_rate);
            strain.process(curr);
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
    curr_combo: &mut u32,
    clock_rate: f64,
) {
    increment_combo_raw(
        is_circle,
        diff_obj.start_time * clock_rate,
        diff_obj.end_time * clock_rate,
        curr_combo,
    );
}

fn increment_combo_raw(is_circle: bool, start_time: f64, end_time: f64, curr_combo: &mut u32) {
    if is_circle {
        *curr_combo += 1;
    } else {
        *curr_combo += 1 + ((end_time - start_time) / 100.0) as u32;
    }
}

#[cfg(test)]
mod tests {
    use crate::{mania::Mania, Beatmap};

    use super::*;

    #[test]
    fn empty() {
        let converted = Beatmap::from_bytes(&[])
            .unwrap()
            .unchecked_into_converted::<Mania>();

        let difficulty = ModeDifficulty::new();
        let mut gradual = ManiaGradualDifficulty::new(&difficulty, &converted);

        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let converted = Beatmap::from_path("./resources/1638954.osu")
            .unwrap()
            .unchecked_into_converted::<Mania>();

        let difficulty = ModeDifficulty::new();

        let mut gradual = ManiaGradualDifficulty::new(&difficulty, &converted);
        let mut gradual_2nd = ManiaGradualDifficulty::new(&difficulty, &converted);
        let mut gradual_3rd = ManiaGradualDifficulty::new(&difficulty, &converted);

        let hit_objects_len = converted.hit_objects.len();

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

            let expected = ModeDifficulty::new()
                .passed_objects(i as u32)
                .calculate(&converted);

            assert_eq!(next_gradual, expected);
        }
    }
}
