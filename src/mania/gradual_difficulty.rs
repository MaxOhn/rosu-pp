#![cfg(feature = "gradual")]

use std::borrow::Cow;

use crate::{
    beatmap::BeatmapHitWindows,
    parse::{HitObject, HitObjectKind},
    util::FloatExt,
    Beatmap, GameMode, Mods,
};

use super::{
    difficulty_object::ManiaDifficultyObject,
    mania_object::ObjectParameters,
    skills::{Skill, Strain},
    ManiaDifficultyAttributes, ManiaObject, STAR_SCALING_FACTOR,
};

/// Gradually calculate the difficulty attributes of an osu!mania map.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`], the map's next hit object will
/// be processed and the [`ManiaDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`ManiaGradualPerformance`](crate::mania::ManiaGradualPerformance) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, mania::ManiaGradualDifficulty};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = ManiaGradualDifficulty::new(&map, mods);
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
#[derive(Clone, Debug)]
pub struct ManiaGradualDifficulty<'map> {
    pub(crate) idx: usize,
    map: Cow<'map, Beatmap>,
    hit_window: f64,
    strain: Strain,
    diff_objects: Box<[ManiaDifficultyObject]>,
    curr_combo: usize,
    clock_rate: f64,
}

impl<'map> ManiaGradualDifficulty<'map> {
    /// Create a new difficulty attributes iterator for osu!mania maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let map = map.convert_mode(GameMode::Mania);
        let total_columns = map.cs.round_even().max(1.0);
        let clock_rate = mods.clock_rate();
        let strain = Strain::new(total_columns as usize);

        let BeatmapHitWindows { od: hit_window, .. } = map
            .attributes()
            .mods(mods)
            .converted(matches!(map, Cow::Owned(_)))
            .clock_rate(clock_rate)
            .hit_windows();

        let mut params = ObjectParameters::new(map.as_ref());
        let mut curr_combo = 0;
        let mut hit_objects = map.hit_objects.iter();

        let first = match hit_objects.next() {
            Some(h) => {
                let hit_object = ManiaObject::new(h, total_columns, &mut params);

                Self::increment_combo_raw(
                    h,
                    hit_object.start_time,
                    hit_object.end_time,
                    &mut curr_combo,
                );

                hit_object
            }
            None => {
                return Self {
                    idx: 0,
                    map,
                    hit_window,
                    strain,
                    diff_objects: Box::from([]),
                    curr_combo: 0,
                    clock_rate,
                }
            }
        };

        let diff_objects_iter = hit_objects.enumerate().scan(first, |last, (i, h)| {
            let base = ManiaObject::new(h, total_columns, &mut params);
            let diff_object = ManiaDifficultyObject::new(&base, &*last, clock_rate, i);
            *last = base;

            Some(diff_object)
        });

        let mut diff_objects = Vec::with_capacity(map.hit_objects.len() - 1);
        diff_objects.extend(diff_objects_iter);

        debug_assert_eq!(diff_objects.len(), diff_objects.capacity());

        Self {
            idx: 0,
            map,
            hit_window,
            strain,
            diff_objects: diff_objects.into_boxed_slice(),
            curr_combo,
            clock_rate,
        }
    }

    fn increment_combo(
        h: &HitObject,
        diff_obj: &ManiaDifficultyObject,
        curr_combo: &mut usize,
        clock_rate: f64,
    ) {
        Self::increment_combo_raw(
            h,
            diff_obj.start_time * clock_rate,
            diff_obj.end_time * clock_rate,
            curr_combo,
        );
    }

    fn increment_combo_raw(h: &HitObject, start_time: f64, end_time: f64, curr_combo: &mut usize) {
        match h.kind {
            HitObjectKind::Circle => *curr_combo += 1,
            _ => *curr_combo += 1 + ((end_time - start_time) / 100.0) as usize,
        }
    }
}

impl Iterator for ManiaGradualDifficulty<'_> {
    type Item = ManiaDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        // The first difficulty object belongs to the second note since each difficulty
        // object requires the current and the last note. Hence, if we're still on the first
        // object, we don't have a difficulty object yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;
            self.strain.process(curr, &self.diff_objects);

            let h = &self.map.hit_objects[self.idx];
            Self::increment_combo(h, curr, &mut self.curr_combo, self.clock_rate);
        } else if self.map.hit_objects.is_empty() {
            return None;
        }

        self.idx += 1;

        Some(ManiaDifficultyAttributes {
            stars: self.strain.clone().difficulty_value() * STAR_SCALING_FACTOR,
            hit_window: self.hit_window,
            max_combo: self.curr_combo,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let skip_iter = self
            .diff_objects
            .iter()
            .zip(self.map.hit_objects.iter().skip(1))
            .skip(self.idx.saturating_sub(1));

        let mut take = n.min(self.len().saturating_sub(1));

        // The first note has no difficulty object
        if self.idx == 0 && take > 0 {
            take -= 1;
            self.idx += 1;
        }

        for (curr, h) in skip_iter.take(take) {
            Self::increment_combo(h, curr, &mut self.curr_combo, self.clock_rate);
            self.strain.process(curr, &self.diff_objects);
            self.idx += 1;
        }

        self.next()
    }
}

impl ExactSizeIterator for ManiaGradualDifficulty<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.diff_objects.len() + 1 - self.idx
    }
}
