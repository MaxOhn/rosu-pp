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
/// Note that this struct implements [`Iterator`](std::iter::Iterator).
/// On every call of [`Iterator::next`](std::iter::Iterator::next), the map's next hit object will
/// be processed and the [`TaikoDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`TaikoGradualPerformanceAttributes`](crate::taiko::TaikoGradualPerformanceAttributes) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, taiko::TaikoGradualDifficultyAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = TaikoGradualDifficultyAttributes::new(&map, mods);
///
/// let attrs1 = iter.next(); // the difficulty of the map after the first hit object
/// let attrs2 = iter.next(); //                           after the second hit object
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TaikoGradualDifficultyAttributes {
    attrs: TaikoDifficultyAttributes,
    hit_objects: IntoIter<Rc<RefCell<TaikoDifficultyObject>>>,
    lists: ObjectLists,
    peaks: Peaks,
    total_hits: usize,
    is_convert: bool,
    pub(crate) started: bool,
}

impl TaikoGradualDifficultyAttributes {
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

        let mut attrs = TaikoDifficultyAttributes {
            stamina: 0.0,
            rhythm: 0.0,
            colour: 0.0,
            peak: 0.0,
            hit_window,
            stars: 0.0,
            max_combo: 0,
        };

        if map.hit_objects.len() < 2 {
            return Self {
                hit_objects: Vec::new().into_iter(),
                lists: ObjectLists::default(),
                peaks,
                attrs,
                total_hits: 0,
                is_convert,
                started: false,
            };
        }

        attrs.max_combo += map.hit_objects[0].is_circle() as usize;
        attrs.max_combo += map.hit_objects[1].is_circle() as usize;
        let mut total_hits = attrs.max_combo;

        let mut diff_objects = map
            .taiko_objects()
            .skip(2)
            .zip(map.hit_objects.iter().skip(1))
            .zip(map.hit_objects.iter())
            .enumerate()
            .fold(
                ObjectLists::default(),
                |mut lists, (idx, (((base, base_start_time), last), last_last))| {
                    total_hits += base.is_hit as usize;

                    let diff_obj = TaikoDifficultyObject::new(
                        base,
                        base_start_time,
                        last.start_time,
                        last_last.start_time,
                        clock_rate,
                        &lists,
                        idx,
                    );

                    match &diff_obj.mono_idx {
                        MonoIndex::Centre(_) => lists.centres.push(idx),
                        MonoIndex::Rim(_) => lists.rims.push(idx),
                        MonoIndex::None => {}
                    }

                    if diff_obj.note_idx.is_some() {
                        lists.notes.push(idx);
                    }

                    lists.all.push(Rc::new(RefCell::new(diff_obj)));

                    lists
                },
            );

        ColourDifficultyPreprocessor::process_and_assign(&mut diff_objects);

        Self {
            hit_objects: diff_objects.all.clone().into_iter(),
            lists: diff_objects,
            peaks,
            attrs,
            total_hits,
            is_convert,
            started: false,
        }
    }
}

impl Iterator for TaikoGradualDifficultyAttributes {
    type Item = TaikoDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        self.started = true;

        loop {
            let curr = self.hit_objects.next()?;
            let borrowed = curr.borrow();
            self.peaks.process(&borrowed, &self.lists);

            if borrowed.base.is_hit {
                self.attrs.max_combo += 1;

                break;
            }
        }

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
        let skip = n
            .min(self.total_hits - self.attrs.max_combo)
            .saturating_sub(1);

        for _ in 0..skip {
            loop {
                let curr = self.hit_objects.next()?;
                let borrowed = curr.borrow();
                self.peaks.process(&borrowed, &self.lists);

                if borrowed.base.is_hit {
                    self.attrs.max_combo += 1;

                    break;
                }
            }
        }

        self.next()
    }
}

impl ExactSizeIterator for TaikoGradualDifficultyAttributes {
    #[inline]
    fn len(&self) -> usize {
        self.hit_objects.len()
    }
}
