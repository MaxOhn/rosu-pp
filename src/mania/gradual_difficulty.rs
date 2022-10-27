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
/// Note that this struct implements [`Iterator`](std::iter::Iterator).
/// On every call of [`Iterator::next`](std::iter::Iterator::next), the map's next hit object will
/// be processed and the [`ManiaDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`ManiaGradualPerformanceAttributes`](crate::mania::ManiaGradualPerformanceAttributes) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, mania::ManiaGradualDifficultyAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = ManiaGradualDifficultyAttributes::new(&map, mods);
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
pub struct ManiaGradualDifficultyAttributes<'map> {
    pub(crate) idx: usize,
    map: Cow<'map, Beatmap>,
    hit_window: f64,
    strain: Strain,
    diff_objects: Vec<ManiaDifficultyObject>,
    curr_combo: usize,
    clock_rate: f64,
}

impl<'map> ManiaGradualDifficultyAttributes<'map> {
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
        let mut hit_objects = map.hit_objects.iter();

        let first = match hit_objects.next() {
            Some(h) => ManiaObject::new(h, total_columns, &mut params),
            None => {
                return Self {
                    idx: 0,
                    map,
                    hit_window,
                    strain,
                    diff_objects: Vec::new(),
                    curr_combo: 0,
                    clock_rate,
                }
            }
        };

        let curr_combo = params.max_combo;

        let diff_objects_iter = hit_objects.enumerate().scan(first, |last, (i, h)| {
            let base = ManiaObject::new(h, total_columns, &mut params);
            let diff_object = ManiaDifficultyObject::new(&base, &*last, clock_rate, i);
            *last = base;

            Some(diff_object)
        });

        let mut diff_objects = Vec::with_capacity(map.hit_objects.len().saturating_sub(1));
        diff_objects.extend(diff_objects_iter);

        Self {
            idx: 0,
            map,
            hit_window,
            strain,
            diff_objects,
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
        match &h.kind {
            HitObjectKind::Circle => *curr_combo += 1,
            _ => {
                let start_time = diff_obj.start_time * clock_rate;
                let end_time = diff_obj.end_time * clock_rate;
                let duration = end_time - start_time;

                *curr_combo += 1 + (duration / 100.0) as usize;
            }
        }
    }
}

impl Iterator for ManiaGradualDifficultyAttributes<'_> {
    type Item = ManiaDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.diff_objects.get(self.idx)?;
        self.idx += 1;

        if let Some(h) = self.map.hit_objects.get(self.idx) {
            Self::increment_combo(h, curr, &mut self.curr_combo, self.clock_rate);
        }

        self.strain.process(curr, &self.diff_objects);

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
        let skip = n.min(self.len()).saturating_sub(1);

        for _ in 0..skip {
            let curr = self.diff_objects.get(self.idx)?;
            self.idx += 1;

            if let Some(h) = self.map.hit_objects.get(self.idx) {
                Self::increment_combo(h, curr, &mut self.curr_combo, self.clock_rate);
            }

            self.strain.process(curr, &self.diff_objects);
        }

        self.next()
    }
}

impl ExactSizeIterator for ManiaGradualDifficultyAttributes<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.diff_objects.len() - self.idx
    }
}
