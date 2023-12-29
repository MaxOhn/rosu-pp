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
    map: Cow<'map, Beatmap>,
    inner: ManiaGradualDifficultyInner,
}

impl<'map> ManiaGradualDifficulty<'map> {
    /// Create a new difficulty attributes iterator for osu!mania maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let map = map.convert_mode(GameMode::Mania);
        let is_convert = matches!(map, Cow::Owned(_));
        let inner = ManiaGradualDifficultyInner::new(map.as_ref(), is_convert, mods);

        Self { map, inner }
    }

    pub(crate) fn idx(&self) -> usize {
        self.inner.idx
    }
}

impl Iterator for ManiaGradualDifficulty<'_> {
    type Item = ManiaDifficultyAttributes;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(&self.map.hit_objects)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n, &self.map.hit_objects)
    }
}

impl ExactSizeIterator for ManiaGradualDifficulty<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Gradually calculate the difficulty attributes of an osu!mania map.
///
/// Check [`ManiaGradualDifficulty`] for more information. This struct does the same
/// but takes ownership of [`Beatmap`] to avoid being bound to a lifetime.
#[cfg_attr(docsrs, doc(cfg(feature = "gradual")))]
#[derive(Clone, Debug)]
pub struct ManiaOwnedGradualDifficulty {
    // Technically only `Beatmap::hit_objects` are required here but storing
    // the full map lets us get away with not storing the map in `ManiaOwnedGradualPerformance`.
    pub(crate) map: Beatmap,
    inner: ManiaGradualDifficultyInner,
}

impl ManiaOwnedGradualDifficulty {
    /// Create a new owned difficulty attributes iterator for osu!mania maps.
    pub fn new(map: Beatmap, mods: u32) -> Self {
        let converted_map = map.convert_mode(GameMode::Mania);
        let is_convert = matches!(converted_map, Cow::Owned(_));
        let inner = ManiaGradualDifficultyInner::new(&converted_map, is_convert, mods);

        let map = match converted_map {
            Cow::Owned(map) => map,
            Cow::Borrowed(_) => map,
        };

        Self { map, inner }
    }

    #[allow(unused)]
    pub(crate) fn idx(&self) -> usize {
        self.inner.idx
    }
}

impl Iterator for ManiaOwnedGradualDifficulty {
    type Item = ManiaDifficultyAttributes;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(&self.map.hit_objects)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n, &self.map.hit_objects)
    }
}

impl ExactSizeIterator for ManiaOwnedGradualDifficulty {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone, Debug)]
struct ManiaGradualDifficultyInner {
    pub(crate) idx: usize,
    hit_window: f64,
    strain: Strain,
    diff_objects: Box<[ManiaDifficultyObject]>,
    curr_combo: usize,
    clock_rate: f64,
    is_convert: bool,
}

impl ManiaGradualDifficultyInner {
    fn new(map: &Beatmap, is_convert: bool, mods: u32) -> Self {
        let total_columns = map.cs.round_even().max(1.0);
        let clock_rate = mods.clock_rate();
        let strain = Strain::new(total_columns as usize);

        let BeatmapHitWindows { od: hit_window, .. } = map
            .attributes()
            .mods(mods)
            .converted(is_convert)
            .clock_rate(clock_rate)
            .hit_windows();

        let mut params = ObjectParameters::new(map);
        let mut curr_combo = 0;
        let mut hit_objects = map.hit_objects.iter();

        let first = match hit_objects.next() {
            Some(h) => {
                let hit_object = ManiaObject::new(h, total_columns, &mut params);

                increment_combo_raw(
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
                    hit_window,
                    strain,
                    diff_objects: Box::from([]),
                    curr_combo: 0,
                    clock_rate,
                    is_convert,
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
            hit_window,
            strain,
            diff_objects: diff_objects.into_boxed_slice(),
            curr_combo,
            clock_rate,
            is_convert,
        }
    }

    fn next(&mut self, hit_objects: &[HitObject]) -> Option<ManiaDifficultyAttributes> {
        // The first difficulty object belongs to the second note since each difficulty
        // object requires the current and the last note. Hence, if we're still on the first
        // object, we don't have a difficulty object yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;
            self.strain.process(curr, &self.diff_objects);

            let h = &hit_objects[self.idx];
            increment_combo(h, curr, &mut self.curr_combo, self.clock_rate);
        } else if hit_objects.is_empty() {
            return None;
        }

        self.idx += 1;

        Some(ManiaDifficultyAttributes {
            stars: self.strain.clone().difficulty_value() * STAR_SCALING_FACTOR,
            hit_window: self.hit_window,
            max_combo: self.curr_combo,
            is_convert: self.is_convert,
            n_objects: self.diff_objects.len() + 1,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize, hit_objects: &[HitObject]) -> Option<ManiaDifficultyAttributes> {
        let skip_iter = self
            .diff_objects
            .iter()
            .zip(hit_objects.iter().skip(1))
            .skip(self.idx.saturating_sub(1));

        let mut take = n.min(self.len().saturating_sub(1));

        // The first note has no difficulty object
        if self.idx == 0 && take > 0 {
            take -= 1;
            self.idx += 1;
        }

        for (curr, h) in skip_iter.take(take) {
            increment_combo(h, curr, &mut self.curr_combo, self.clock_rate);
            self.strain.process(curr, &self.diff_objects);
            self.idx += 1;
        }

        self.next(hit_objects)
    }

    fn len(&self) -> usize {
        self.diff_objects.len() + 1 - self.idx
    }
}

fn increment_combo(
    h: &HitObject,
    diff_obj: &ManiaDifficultyObject,
    curr_combo: &mut usize,
    clock_rate: f64,
) {
    increment_combo_raw(
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
