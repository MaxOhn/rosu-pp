use std::{cmp, mem, slice::Iter};

use crate::{
    model::{beatmap::HitWindows, hit_object::HitObject},
    taiko::TaikoBeatmap,
    util::sync::RefCount,
    ModeDifficulty,
};

use super::{
    object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    skills::peaks::{Peaks, PeaksSkill},
    DifficultyValues, TaikoDifficultyAttributes,
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
/// use rosu_pp::{Beatmap, ModeDifficulty};
/// use rosu_pp::taiko::{Taiko, TaikoGradualDifficulty};
///
/// let converted = Beatmap::from_path("./resources/1028484.osu")
///     .unwrap()
///     .unchecked_into_converted::<Taiko>();
///
/// let difficulty = ModeDifficulty::new().mods(64); // DT
/// let mut iter = TaikoGradualDifficulty::new(&difficulty, &converted);
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
    pub(crate) mods: u32,
    pub(crate) clock_rate: f64,
    attrs: TaikoDifficultyAttributes,
    diff_objects: TaikoDifficultyObjects,
    diff_objects_iter: Iter<'static, RefCount<TaikoDifficultyObject>>,
    peaks: Peaks,
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
    pub fn new(difficulty: &ModeDifficulty, converted: &TaikoBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let mods = difficulty.get_mods();
        let clock_rate = difficulty.get_clock_rate();

        let first_combos = match (
            converted.map.hit_objects.first().map(HitObject::is_circle),
            converted.map.hit_objects.get(1).map(HitObject::is_circle),
        ) {
            (None, _) | (Some(false), Some(false) | None) => FirstTwoCombos::None,
            (Some(true), Some(false) | None) => FirstTwoCombos::OnlyFirst,
            (Some(false), Some(true)) => FirstTwoCombos::OnlySecond,
            (Some(true), Some(true)) => FirstTwoCombos::Both,
        };

        let HitWindows { od: hit_window, .. } = converted
            .attributes()
            .mods(mods)
            .clock_rate(clock_rate)
            .hit_windows();

        let mut n_diff_objects = 0;
        let mut max_combo = 0;

        let diff_objects = DifficultyValues::create_difficulty_objects(
            converted,
            take as u32,
            clock_rate,
            &mut max_combo,
            &mut n_diff_objects,
        );

        let peaks = Peaks::new();

        let attrs = TaikoDifficultyAttributes {
            hit_window,
            is_convert: converted.map.is_convert,
            ..Default::default()
        };

        let total_hits = converted
            .map
            .hit_objects
            .iter()
            .filter(|h| h.is_circle())
            .count();

        let diff_objects_iter = extend_lifetime(diff_objects.iter());

        Self {
            idx: 0,
            mods,
            clock_rate,
            diff_objects,
            diff_objects_iter,
            peaks,
            attrs,
            total_hits,
            first_combos,
        }
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
                PeaksSkill::new(&mut self.peaks, &self.diff_objects).process(&borrowed);

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

        let color = self.peaks.color_difficulty_value();
        let rhythm = self.peaks.rhythm_difficulty_value();
        let stamina = self.peaks.stamina_difficulty_value();
        let combined = self.peaks.clone().difficulty_value();

        let mut attrs = self.attrs.clone();

        DifficultyValues::eval(&mut attrs, color, rhythm, stamina, combined);

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

        let mut peaks = PeaksSkill::new(&mut self.peaks, &self.diff_objects);

        for _ in 0..take {
            loop {
                let curr = self.diff_objects_iter.next()?;
                let borrowed = curr.get();
                peaks.process(&borrowed);

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

    use crate::Beatmap;

    use super::*;

    #[test]
    fn empty() {
        let converted = Beatmap::from_bytes(&[]).unwrap().unchecked_into_converted();

        let difficulty = ModeDifficulty::new();
        let mut gradual = TaikoGradualDifficulty::new(&difficulty, &converted);

        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let converted = Beatmap::from_path("./resources/1028484.osu")
            .unwrap()
            .unchecked_into_converted();

        let difficulty = ModeDifficulty::new();

        let mut gradual = TaikoGradualDifficulty::new(&difficulty, &converted);
        let mut gradual_2nd = TaikoGradualDifficulty::new(&difficulty, &converted);
        let mut gradual_3rd = TaikoGradualDifficulty::new(&difficulty, &converted);

        let hit_objects_len = converted.map.hit_objects.len();

        let n_hits = converted
            .map
            .hit_objects
            .iter()
            .filter(|h| h.is_circle())
            .count();

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

            let expected = ModeDifficulty::new()
                .passed_objects(i as u32)
                .calculate(&converted);

            assert_eq!(next_gradual, expected);
        }
    }
}
