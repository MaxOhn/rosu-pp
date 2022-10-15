use crate::{beatmap::BeatmapHitWindows, parse::HitObjectKind, Beatmap, Mods};

use super::{
    difficulty_object::ManiaDifficultyObject,
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
    map: &'map Beatmap,
    hit_window: f64,
    strain: Strain,
    diff_objects: Vec<ManiaDifficultyObject>,
    curr_combo: usize,
}

impl<'map> ManiaGradualDifficultyAttributes<'map> {
    /// Create a new difficulty attributes iterator for osu!mania maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let total_columns = map.cs.round().max(1.0);

        let clock_rate = mods.clock_rate();
        let strain = Strain::new(total_columns as usize);

        let BeatmapHitWindows { od: hit_window, .. } = map
            .attributes()
            .mods(mods)
            // TODO: allow converts in gradual calc
            .converted(false)
            .clock_rate(clock_rate)
            .hit_windows();

        let diff_objects_iter = map
            .hit_objects
            .iter()
            .skip(1)
            .map(ManiaObject::new)
            .enumerate()
            .zip(map.hit_objects.iter().map(ManiaObject::new))
            .map(|((i, base), prev)| {
                ManiaDifficultyObject::new(base, prev, clock_rate, total_columns, i)
            });

        let curr_combo = if let Some(h) = map.hit_objects.first() {
            match &h.kind {
                HitObjectKind::Hold { end_time } => {
                    1 + ((*end_time - h.start_time) / 100.0) as usize
                }
                _ => 1,
            }
        } else {
            0
        };

        let mut diff_objects = Vec::with_capacity(map.hit_objects.len().saturating_sub(1));
        diff_objects.extend(diff_objects_iter);

        Self {
            idx: 0,
            map,
            hit_window,
            strain,
            diff_objects,
            curr_combo,
        }
    }
}

impl Iterator for ManiaGradualDifficultyAttributes<'_> {
    type Item = ManiaDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.diff_objects.get(self.idx)?;
        self.idx += 1;

        if let Some(h) = self.map.hit_objects.get(self.idx) {
            match &h.kind {
                HitObjectKind::Hold { end_time } => {
                    self.curr_combo += 1 + ((*end_time - h.start_time) / 100.0) as usize
                }
                _ => self.curr_combo += 1,
            }
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
}

impl ExactSizeIterator for ManiaGradualDifficultyAttributes<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.diff_objects.len() - self.idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map() {
        let map = Beatmap::default();
        let mut attributes = ManiaGradualDifficultyAttributes::new(&map, 0);
        assert!(attributes.next().is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn iter_end_eq_regular() {
        let map = Beatmap::from_path("./maps/1974394.osu").expect("failed to parse map");
        let mods = 64;
        let regular = crate::ManiaStars::new(&map).mods(mods).calculate();

        let iter_end = ManiaGradualDifficultyAttributes::new(&map, mods)
            .last()
            .expect("empty iter");

        assert_eq!(regular, iter_end);
    }
}
