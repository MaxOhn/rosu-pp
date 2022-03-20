use std::{
    iter::{self, Skip, Zip},
    slice::Iter,
};

use crate::{
    mania::{strain::Strain, SECTION_LEN},
    parse::HitObject,
    Beatmap, GameMode, Mods,
};

use super::{DifficultyHitObject, ManiaDifficultyAttributes, STAR_SCALING_FACTOR};

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
    difficulty_objects: ManiaObjectIter<'map>,
    strain: Strain,
    curr_section_end: f64,
    strain_peak_buf: Vec<f64>,
}

impl<'map> ManiaGradualDifficultyAttributes<'map> {
    /// Create a new difficulty attributes iterator for osu!mania maps.
    pub fn new(map: &'map Beatmap, mods: impl Mods) -> Self {
        let rounded_cs = map.cs.round();

        let columns = match map.mode {
            GameMode::MNA => rounded_cs.max(1.0) as u8,
            GameMode::STD => {
                let rounded_od = map.od.round();

                let n_objects = map.n_circles + map.n_sliders + map.n_spinners;
                let slider_or_spinner_ratio = (n_objects - map.n_circles) as f32 / n_objects as f32;

                if slider_or_spinner_ratio < 0.2 {
                    7
                } else if slider_or_spinner_ratio < 0.3 || rounded_cs >= 5.0 {
                    6 + (rounded_od > 5.0) as u8
                } else if slider_or_spinner_ratio > 0.6 {
                    4 + (rounded_od > 4.0) as u8
                } else {
                    (rounded_od as u8 + 1).max(4).min(7)
                }
            }
            other => panic!("can not calculate mania difficulty on a {:?} map", other),
        };

        let clock_rate = mods.speed();
        let strain = Strain::new(columns);
        let columns = columns as f32;
        let difficulty_objects = ManiaObjectIter::new(&map.hit_objects, columns, clock_rate);

        Self {
            idx: 0,
            difficulty_objects,
            strain,
            curr_section_end: 0.0,
            strain_peak_buf: Vec::new(),
        }
    }
}

impl Iterator for ManiaGradualDifficultyAttributes<'_> {
    type Item = ManiaDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        self.idx = self.idx.saturating_add(1);

        if self.idx == 1 {
            return (!self.difficulty_objects.is_empty).then(ManiaDifficultyAttributes::default);
        }

        let h = self.difficulty_objects.next()?;

        if self.idx == 2 {
            self.curr_section_end = (h.start_time / SECTION_LEN).ceil() * SECTION_LEN;
        } else {
            while h.start_time > self.curr_section_end {
                self.strain.save_current_peak();
                self.strain.start_new_section_from(self.curr_section_end);
                self.curr_section_end += SECTION_LEN;
            }
        }

        self.strain.process(&h);

        let missing = self.strain.strain_peaks.len() + 1 - self.strain_peak_buf.len();
        self.strain_peak_buf.extend(iter::repeat(0.0).take(missing));

        self.strain_peak_buf[..self.strain.strain_peaks.len()]
            .copy_from_slice(&self.strain.strain_peaks);

        if let Some(last) = self.strain_peak_buf.last_mut() {
            *last = self.strain.curr_section_peak;
        }

        let stars = Strain::difficulty_value(&mut self.strain_peak_buf) * STAR_SCALING_FACTOR;

        Some(ManiaDifficultyAttributes { stars })
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
        self.difficulty_objects.len() + (self.idx == 0) as usize
    }
}

#[derive(Clone, Debug)]
struct ManiaObjectIter<'map> {
    hit_objects: Zip<Skip<Iter<'map, HitObject>>, Iter<'map, HitObject>>,
    columns: f32,
    clock_rate: f64,
    is_empty: bool,
}

impl<'map> ManiaObjectIter<'map> {
    fn new(hit_objects: &'map [HitObject], columns: f32, clock_rate: f64) -> Self {
        let is_empty = hit_objects.is_empty();
        let hit_objects = hit_objects.iter().skip(1).zip(hit_objects);

        Self {
            hit_objects,
            columns,
            clock_rate,
            is_empty,
        }
    }
}

impl<'map> Iterator for ManiaObjectIter<'map> {
    type Item = DifficultyHitObject<'map>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (base, prev) = self.hit_objects.next()?;
        let obj = DifficultyHitObject::new(base, prev, self.columns, self.clock_rate);

        Some(obj)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hit_objects.size_hint()
    }
}

impl ExactSizeIterator for ManiaObjectIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.hit_objects.len()
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
