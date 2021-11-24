use std::{
    cmp::Ordering,
    iter::{self, Enumerate, Skip, Zip},
    slice::Iter,
};

use crate::{
    parse::{HitObject, HitObjectKind},
    taiko::{
        difficulty_object::DifficultyObject, norm, rescale, simple_color_penalty,
        stamina_cheese::StaminaCheeseDetector, COLOR_SKILL_MULTIPLIER, RHYTHM_SKILL_MULTIPLIER,
        SECTION_LEN, STAMINA_SKILL_MULTIPLIER,
    },
    Beatmap, Mods,
};

use super::{skill::Skills, TaikoDifficultyAttributes};

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
pub struct TaikoGradualDifficultyAttributes<'map> {
    pub(crate) idx: usize,
    difficulty_objects: TaikoObjectIter<'map>,
    cheese: Vec<bool>,
    skills: Skills,
    curr_section_end: f64,
    strain_peak_buf: Vec<f64>,
}

impl<'map> TaikoGradualDifficultyAttributes<'map> {
    /// Create a new difficulty attributes iterator for osu!taiko maps.
    pub fn new(map: &'map Beatmap, mods: impl Mods) -> Self {
        // True if the object at that index is stamina cheese
        let cheese = map.find_cheese();

        let skills = Skills::new();
        let clock_rate = mods.speed();
        let difficulty_objects = TaikoObjectIter::new(&map.hit_objects, clock_rate);

        Self {
            idx: 0,
            difficulty_objects,
            cheese,
            skills,
            curr_section_end: 0.0,
            strain_peak_buf: Vec::new(),
        }
    }

    fn locally_combined_difficulty(&mut self, stamina_penalty: f64) -> f64 {
        let iter = self
            .skills
            .color
            .strain_peaks
            .iter()
            .zip(self.skills.rhythm.strain_peaks.iter())
            .zip(self.skills.stamina_right.strain_peaks.iter())
            .zip(self.skills.stamina_left.strain_peaks.iter())
            .map(|(((&color, &rhythm), &stamina_right), &stamina_left)| {
                norm(
                    2.0,
                    color * COLOR_SKILL_MULTIPLIER,
                    rhythm * RHYTHM_SKILL_MULTIPLIER,
                    (stamina_right + stamina_left) * STAMINA_SKILL_MULTIPLIER * stamina_penalty,
                )
            });

        self.strain_peak_buf.clear();
        self.strain_peak_buf.extend(iter);

        let last = norm(
            2.0,
            self.skills.color.curr_section_peak * COLOR_SKILL_MULTIPLIER,
            self.skills.rhythm.curr_section_peak * RHYTHM_SKILL_MULTIPLIER,
            (self.skills.stamina_right.curr_section_peak
                + self.skills.stamina_left.curr_section_peak)
                * STAMINA_SKILL_MULTIPLIER
                * stamina_penalty,
        );

        self.strain_peak_buf.push(last);

        self.strain_peak_buf
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        let mut difficulty = 0.0;
        let mut weight = 1.0;

        for strain in &self.strain_peak_buf {
            difficulty += strain * weight;
            weight *= 0.9;
        }

        difficulty
    }
}

impl Iterator for TaikoGradualDifficultyAttributes<'_> {
    type Item = TaikoDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        self.idx = self.idx.saturating_add(1);

        if self.idx == 1 {
            if self.difficulty_objects.first_object.is_empty() {
                return None;
            }

            self.difficulty_objects.max_combo +=
                self.difficulty_objects.first_object.is_circle() as usize;

            let attributes = TaikoDifficultyAttributes {
                stars: 0.0,
                max_combo: self.difficulty_objects.max_combo,
            };

            return Some(attributes);
        } else if self.idx == 2 {
            if self.difficulty_objects.second_object.is_empty() {
                return None;
            }

            self.difficulty_objects.max_combo +=
                self.difficulty_objects.second_object.is_circle() as usize;

            let attributes = TaikoDifficultyAttributes {
                stars: 0.0,
                max_combo: self.difficulty_objects.max_combo,
            };

            return Some(attributes);
        }

        let h = self.difficulty_objects.next()?;

        if self.idx == 3 {
            self.curr_section_end = (h.start_time / SECTION_LEN).ceil() * SECTION_LEN;
        } else {
            while h.start_time > self.curr_section_end {
                self.skills
                    .save_peak_and_start_new_section(self.curr_section_end);
                self.curr_section_end += SECTION_LEN;
            }
        }

        self.skills.process(&h, &self.cheese);

        let len = self.skills.strain_peaks_len();
        let missing = len + 1 - self.strain_peak_buf.len();
        self.strain_peak_buf.extend(iter::repeat(0.0).take(missing));

        self.skills
            .color
            .copy_strain_peaks(&mut self.strain_peak_buf[..len]);

        if let Some(last) = self.strain_peak_buf.last_mut() {
            *last = self.skills.color.curr_section_peak;
        }

        let color_rating = self
            .skills
            .color
            .difficulty_value(&mut self.strain_peak_buf)
            * COLOR_SKILL_MULTIPLIER;

        self.skills
            .rhythm
            .copy_strain_peaks(&mut self.strain_peak_buf[..len]);

        if let Some(last) = self.strain_peak_buf.last_mut() {
            *last = self.skills.rhythm.curr_section_peak;
        }

        let rhythm_rating = self
            .skills
            .rhythm
            .difficulty_value(&mut self.strain_peak_buf)
            * RHYTHM_SKILL_MULTIPLIER;

        self.skills
            .stamina_right
            .copy_strain_peaks(&mut self.strain_peak_buf[..len]);

        if let Some(last) = self.strain_peak_buf.last_mut() {
            *last = self.skills.stamina_right.curr_section_peak;
        }

        let stamina_right = self
            .skills
            .stamina_right
            .difficulty_value(&mut self.strain_peak_buf);

        self.skills
            .stamina_left
            .copy_strain_peaks(&mut self.strain_peak_buf[..len]);

        if let Some(last) = self.strain_peak_buf.last_mut() {
            *last = self.skills.stamina_left.curr_section_peak;
        }

        let stamina_left = self
            .skills
            .stamina_left
            .difficulty_value(&mut self.strain_peak_buf);

        let mut stamina_rating = (stamina_right + stamina_left) * STAMINA_SKILL_MULTIPLIER;

        let stamina_penalty = simple_color_penalty(stamina_rating, color_rating);
        stamina_rating *= stamina_penalty;

        let combined_rating = self.locally_combined_difficulty(stamina_penalty);
        let separate_rating = norm(1.5, color_rating, rhythm_rating, stamina_rating);

        let stars = rescale(1.4 * separate_rating + 0.5 * combined_rating);

        let attributes = TaikoDifficultyAttributes {
            stars,
            max_combo: self.difficulty_objects.max_combo,
        };

        Some(attributes)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for TaikoGradualDifficultyAttributes<'_> {
    #[inline]
    fn len(&self) -> usize {
        let mut len = self.difficulty_objects.len();

        if self.idx == 0 && !self.difficulty_objects.first_object.is_empty() {
            len += 1 + !self.difficulty_objects.second_object.is_empty() as usize;
        } else if self.idx == 1 && !self.difficulty_objects.second_object.is_empty() {
            len += 1;
        }

        len
    }
}

type InnerIter<'map> = Zip<
    Zip<Skip<Enumerate<Iter<'map, HitObject>>>, Skip<Iter<'map, HitObject>>>,
    Iter<'map, HitObject>,
>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
enum SimpleObject {
    Circle,
    Empty,
    NonCircle,
}

impl From<&HitObject> for SimpleObject {
    fn from(h: &HitObject) -> Self {
        match h.kind {
            HitObjectKind::Circle => Self::Circle,
            _ => Self::NonCircle,
        }
    }
}

impl SimpleObject {
    fn is_empty(self) -> bool {
        self == Self::Empty
    }

    fn is_circle(self) -> bool {
        self == Self::Circle
    }
}

#[derive(Clone, Debug)]
struct TaikoObjectIter<'map> {
    hit_objects: InnerIter<'map>,
    max_combo: usize,
    clock_rate: f64,
    first_object: SimpleObject,
    second_object: SimpleObject,
}

impl<'map> TaikoObjectIter<'map> {
    fn new(hit_objects: &'map [HitObject], clock_rate: f64) -> Self {
        let first_object = hit_objects.get(0).map_or(SimpleObject::Empty, From::from);
        let second_object = hit_objects.get(1).map_or(SimpleObject::Empty, From::from);

        let hit_objects = hit_objects
            .iter()
            .enumerate()
            .skip(2)
            .zip(hit_objects.iter().skip(1))
            .zip(hit_objects.iter());

        Self {
            hit_objects,
            max_combo: 0,
            clock_rate,
            first_object,
            second_object,
        }
    }
}

impl<'map> Iterator for TaikoObjectIter<'map> {
    type Item = DifficultyObject<'map>;

    fn next(&mut self) -> Option<Self::Item> {
        let (((idx, base), prev), prev_prev) = self.hit_objects.next()?;
        self.max_combo += base.is_circle() as usize;

        Some(DifficultyObject::new(
            idx,
            base,
            prev,
            prev_prev,
            self.clock_rate,
        ))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hit_objects.size_hint()
    }
}

impl ExactSizeIterator for TaikoObjectIter<'_> {
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
        let mut attributes = TaikoGradualDifficultyAttributes::new(&map, 0);
        assert!(attributes.next().is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn iter_end_eq_regular() {
        let map = Beatmap::from_path("./maps/1028484.osu").expect("failed to parse map");
        let mods = 64;
        let regular = crate::taiko::stars(&map, mods, None);

        let iter_end = TaikoGradualDifficultyAttributes::new(&map, mods)
            .last()
            .expect("empty iter");

        assert_eq!(regular, iter_end);
    }
}
