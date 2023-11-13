#![cfg(feature = "gradual")]

use std::{borrow::Cow, cell::RefCell, rc::Rc, vec::IntoIter};

use crate::{beatmap::BeatmapHitWindows, taiko::rescale, Beatmap, GameMode, Mods};

use super::{
    colours::ColourDifficultyPreprocessor,
    difficulty_object::{MonoIndex, ObjectLists, TaikoDifficultyObject},
    skills::{Peaks, PeaksDifficultyValues, Skill},
    taiko_object::IntoTaikoObjectIter,
    TaikoDifficultyAttributes, DIFFICULTY_MULTIPLIER,
};

/// Gradually calculate the difficulty attributes of an osu!taiko map.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`], the map's next hit object will
/// be processed and the [`TaikoDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`TaikoGradualPerformance`](crate::taiko::TaikoGradualPerformance) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, taiko::TaikoGradualDifficulty};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = TaikoGradualDifficulty::new(&map, mods);
///
/// let attrs1 = iter.next(); // the difficulty of the map after the first hit object
/// let attrs2 = iter.next(); //                           after the second hit object
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "gradual")))]
#[derive(Debug)]
pub struct TaikoGradualDifficulty {
    pub(crate) idx: usize,
    attrs: TaikoDifficultyAttributes,
    diff_objects: IntoIter<Rc<RefCell<TaikoDifficultyObject>>>,
    lists: ObjectLists,
    peaks: Peaks,
    total_hits: usize,
    first_combos: FirstTwoCombos,
    is_convert: bool,
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
    pub fn new(map: &Beatmap, mods: u32) -> Self {
        let map = map.convert_mode(GameMode::Taiko);
        let is_convert = matches!(map, Cow::Owned(_));
        let peaks = Peaks::new();
        let clock_rate = mods.clock_rate();

        let BeatmapHitWindows { od: hit_window, .. } = map
            .attributes()
            .mods(mods)
            .clock_rate(clock_rate)
            .hit_windows();

        let attrs = TaikoDifficultyAttributes {
            hit_window,
            ..Default::default()
        };

        if map.hit_objects.len() < 2 {
            return Self {
                idx: 0,
                diff_objects: Vec::new().into_iter(),
                lists: ObjectLists::with_capacity(0),
                peaks,
                attrs,
                total_hits: 0,
                first_combos: FirstTwoCombos::None,
                is_convert,
            };
        }

        let first_combos = match (
            map.hit_objects[0].is_circle(),
            map.hit_objects[1].is_circle(),
        ) {
            (false, false) => FirstTwoCombos::None,
            (true, false) => FirstTwoCombos::OnlyFirst,
            (false, true) => FirstTwoCombos::OnlySecond,
            (true, true) => FirstTwoCombos::Both,
        };

        let mut total_hits = 0;
        let mut diff_objects = ObjectLists::with_capacity(map.hit_objects.len().saturating_sub(2));

        map.taiko_objects()
            .inspect(|(h, _)| total_hits += h.is_hit as usize)
            .skip(2)
            .zip(map.hit_objects.iter().skip(1))
            .zip(map.hit_objects.iter())
            .enumerate()
            .for_each(|(idx, (((base, base_start_time), last), last_last))| {
                let diff_obj = TaikoDifficultyObject::new(
                    base,
                    base_start_time,
                    last.start_time,
                    last_last.start_time,
                    clock_rate,
                    &diff_objects,
                    idx,
                );

                match &diff_obj.mono_idx {
                    MonoIndex::Centre(_) => diff_objects.centres.push(idx),
                    MonoIndex::Rim(_) => diff_objects.rims.push(idx),
                    MonoIndex::None => {}
                }

                if diff_obj.note_idx.is_some() {
                    diff_objects.notes.push(idx);
                }

                diff_objects.all.push(Rc::new(RefCell::new(diff_obj)));
            });

        ColourDifficultyPreprocessor::process_and_assign(&mut diff_objects);

        Self {
            idx: 0,
            diff_objects: diff_objects.all.clone().into_iter(),
            lists: diff_objects,
            peaks,
            attrs,
            total_hits,
            first_combos,
            is_convert,
        }
    }
}

impl Iterator for TaikoGradualDifficulty {
    type Item = TaikoDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        // The first difficulty object belongs to the third note since each difficulty
        // object requires the current the last, and the second to last note. Hence, if we're still
        // on the first or second object, we don't have a difficulty object yet and just skip
        // processing.
        if self.idx >= 2 {
            loop {
                let curr = self.diff_objects.next()?;
                let borrowed = curr.borrow();
                self.peaks.process(&borrowed, &self.lists);

                if borrowed.base.is_hit {
                    self.attrs.max_combo += 1;

                    break;
                }
            }
        } else if self.lists.all.is_empty() {
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

        let PeaksDifficultyValues {
            mut colour_rating,
            mut rhythm_rating,
            mut stamina_rating,
            mut combined_rating,
        } = self.peaks.clone().difficulty_values();

        colour_rating *= DIFFICULTY_MULTIPLIER;
        rhythm_rating *= DIFFICULTY_MULTIPLIER;
        stamina_rating *= DIFFICULTY_MULTIPLIER;
        combined_rating *= DIFFICULTY_MULTIPLIER;

        let mut star_rating = rescale(combined_rating * 1.4);

        // * TODO: This is temporary measure as we don't detect abuse of multiple-input
        // * playstyles of converts within the current system.
        if self.is_convert {
            star_rating *= 0.925;

            // * For maps with low colour variance and high stamina requirement,
            // * multiple inputs are more likely to be abused.
            if colour_rating < 2.0 && stamina_rating > 8.0 {
                star_rating *= 0.8;
            }
        }

        self.attrs.stamina = stamina_rating;
        self.attrs.colour = colour_rating;
        self.attrs.rhythm = rhythm_rating;
        self.attrs.peak = combined_rating;
        self.attrs.stars = star_rating;

        Some(self.attrs.clone())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let mut take = n.min(self.len().saturating_sub(1));

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
            _ => unreachable!(),
        }

        for _ in 0..take {
            loop {
                let curr = self.diff_objects.next()?;
                let borrowed = curr.borrow();
                self.peaks.process(&borrowed, &self.lists);

                if borrowed.base.is_hit {
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
    #[inline]
    fn len(&self) -> usize {
        self.total_hits - self.idx
    }
}
