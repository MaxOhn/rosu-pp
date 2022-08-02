use std::{iter, mem, vec::IntoIter};

use crate::{
    curve::CurveBuffers, osu::difficulty_object::DifficultyObject, parse::Pos2, Beatmap, Mods,
};

use super::{
    calculate_star_rating, old_stacking,
    osu_object::{ObjectParameters, OsuObject, OsuObjectKind},
    scaling_factor::ScalingFactor,
    skill::{Skill, Skills},
    slider_state::SliderState,
    stacking, OsuDifficultyAttributes, DIFFICULTY_MULTIPLIER, SECTION_LEN,
};

/// Gradually calculate the difficulty attributes of an osu!standard map.
///
/// Note that this struct implements [`Iterator`](std::iter::Iterator).
/// On every call of [`Iterator::next`](std::iter::Iterator::next), the map's next hit object will
/// be processed and the [`OsuDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`OsuGradualPerformanceAttributes`](crate::osu::OsuGradualPerformanceAttributes) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, osu::OsuGradualDifficultyAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = OsuGradualDifficultyAttributes::new(&map, mods);
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
pub struct OsuGradualDifficultyAttributes {
    pub(crate) idx: usize,
    attributes: OsuDifficultyAttributes,
    clock_rate: f64,
    hit_objects: OsuObjectIter,
    skills: Skills,
    prev_prev: Option<OsuObject>,
    prev: OsuObject,
    curr_section_end: f64,
    strain_peak_buf: Vec<f64>,
}

impl OsuGradualDifficultyAttributes {
    /// Create a new difficulty attributes iterator for osu!standard maps.
    pub fn new(map: &Beatmap, mods: u32) -> Self {
        let map_attributes = map.attributes().mods(mods).build();
        let hit_window = map_attributes.hit_windows.od;
        let time_preempt = map_attributes.hit_windows.ar;
        let hr = mods.hr();
        let scaling_factor = ScalingFactor::new(map_attributes.cs);

        let mut attributes = OsuDifficultyAttributes {
            ar: map_attributes.ar,
            hp: map_attributes.hp,
            od: map_attributes.od,
            ..Default::default()
        };

        let mut params = ObjectParameters {
            map,
            attributes: &mut attributes,
            slider_state: SliderState::new(map),
            ticks: Vec::new(),
            curve_bufs: CurveBuffers::default(),
        };

        let hit_objects_iter = map
            .hit_objects
            .iter()
            .filter_map(|h| OsuObject::new(h, hr, &mut params));

        let mut hit_objects = Vec::with_capacity(map.hit_objects.len());
        hit_objects.extend(hit_objects_iter);

        attributes.n_circles = 0;
        attributes.n_sliders = 0;
        attributes.n_spinners = 0;
        attributes.max_combo = 0;

        let stack_threshold = time_preempt * map.stack_leniency as f64;

        if map.version >= 6 {
            stacking(&mut hit_objects, stack_threshold);
        } else {
            old_stacking(&mut hit_objects, stack_threshold);
        }

        let skills = Skills::new(hit_window, mods.rx(), scaling_factor.radius(), mods.fl());

        let hit_objects = OsuObjectIter {
            hit_objects: hit_objects.into_iter(),
            scaling_factor,
        };

        let prev_prev = None;

        let prev = OsuObject {
            time: 0.0,
            pos: Pos2::zero(),
            stack_height: 0.0,
            kind: OsuObjectKind::Circle,
        };

        Self {
            idx: 0,
            attributes,
            clock_rate: map_attributes.clock_rate,
            hit_objects,
            skills,
            curr_section_end: 0.0,
            prev_prev,
            prev,
            strain_peak_buf: Vec::new(),
        }
    }
}

impl Iterator for OsuGradualDifficultyAttributes {
    type Item = OsuDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.hit_objects.next()?;
        self.attributes.max_combo += 1;

        match &curr.kind {
            OsuObjectKind::Circle => self.attributes.n_circles += 1,
            OsuObjectKind::Slider { nested_objects, .. } => {
                self.attributes.max_combo += nested_objects.len();
                self.attributes.n_sliders += 1
            }
            OsuObjectKind::Spinner { .. } => self.attributes.n_spinners += 1,
        };

        self.idx += 1;

        if self.idx == 1 {
            self.prev = curr;
            self.curr_section_end =
                (self.prev.time / self.clock_rate / SECTION_LEN).ceil() * SECTION_LEN;

            return Some(self.attributes.clone());
        }

        let h = DifficultyObject::new(
            &curr,
            &mut self.prev,
            self.prev_prev.as_ref(),
            &self.hit_objects.scaling_factor,
            self.clock_rate,
        );

        let base_time = h.base.time / self.clock_rate;

        if self.idx == 2 {
            while base_time > self.curr_section_end {
                self.skills.start_new_section_from(self.curr_section_end);
                self.curr_section_end += SECTION_LEN;
            }
        } else {
            while base_time > self.curr_section_end {
                self.skills
                    .save_peak_and_start_new_section(self.curr_section_end);
                self.curr_section_end += SECTION_LEN;
            }
        }

        self.skills.process(&h);
        self.prev_prev = Some(mem::replace(&mut self.prev, curr));

        let missing = self.skills.aim().strain_peaks.len() + 1 - self.strain_peak_buf.len();
        self.strain_peak_buf.extend(iter::repeat(0.0).take(missing));

        let aim_rating = {
            let aim = self.skills.aim();
            self.strain_peak_buf[..aim.strain_peaks.len()].copy_from_slice(&aim.strain_peaks);

            if let Some(last) = self.strain_peak_buf.last_mut() {
                *last = aim.curr_section_peak;
            }

            Skill::difficulty_value(&mut self.strain_peak_buf, aim).sqrt() * DIFFICULTY_MULTIPLIER
        };

        let slider_factor = if aim_rating > 0.0 {
            let aim_no_sliders = self.skills.aim_no_sliders();
            self.strain_peak_buf[..aim_no_sliders.strain_peaks.len()]
                .copy_from_slice(&aim_no_sliders.strain_peaks);

            if let Some(last) = self.strain_peak_buf.last_mut() {
                *last = aim_no_sliders.curr_section_peak;
            }

            let aim_rating_no_sliders =
                Skill::difficulty_value(&mut self.strain_peak_buf, aim_no_sliders).sqrt()
                    * DIFFICULTY_MULTIPLIER;

            aim_rating_no_sliders / aim_rating
        } else {
            1.0
        };

        let (speed, flashlight) = self.skills.speed_flashlight();

        let speed_rating = if let Some(speed) = speed {
            self.strain_peak_buf[..speed.strain_peaks.len()].copy_from_slice(&speed.strain_peaks);

            if let Some(last) = self.strain_peak_buf.last_mut() {
                *last = speed.curr_section_peak;
            }

            Skill::difficulty_value(&mut self.strain_peak_buf, speed).sqrt() * DIFFICULTY_MULTIPLIER
        } else {
            0.0
        };

        let flashlight_rating = if let Some(flashlight) = flashlight {
            self.strain_peak_buf[..flashlight.strain_peaks.len()]
                .copy_from_slice(&flashlight.strain_peaks);

            if let Some(last) = self.strain_peak_buf.last_mut() {
                *last = flashlight.curr_section_peak;
            }

            Skill::difficulty_value(&mut self.strain_peak_buf, flashlight).sqrt()
                * DIFFICULTY_MULTIPLIER
        } else {
            0.0
        };

        let star_rating = calculate_star_rating(aim_rating, speed_rating, flashlight_rating);

        self.attributes.aim_strain = aim_rating;
        self.attributes.speed_strain = speed_rating;
        self.attributes.flashlight_rating = flashlight_rating;
        self.attributes.slider_factor = slider_factor;
        self.attributes.stars = star_rating;

        Some(self.attributes.clone())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hit_objects.size_hint()
    }
}

impl ExactSizeIterator for OsuGradualDifficultyAttributes {
    #[inline]
    fn len(&self) -> usize {
        self.hit_objects.len()
    }
}

#[derive(Clone, Debug)]
struct OsuObjectIter {
    hit_objects: IntoIter<OsuObject>,
    scaling_factor: ScalingFactor,
}

impl Iterator for OsuObjectIter {
    type Item = OsuObject;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut h = self.hit_objects.next()?;
        let stack_offset = self.scaling_factor.stack_offset(h.stack_height);
        h.pos += stack_offset;

        Some(h)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hit_objects.size_hint()
    }
}

impl ExactSizeIterator for OsuObjectIter {
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
        let mut attributes = OsuGradualDifficultyAttributes::new(&map, 0);
        assert!(attributes.next().is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn iter_end_eq_regular() {
        let map = Beatmap::from_path("./maps/2785319.osu").expect("failed to parse map");
        let mods = 64;
        let regular = crate::OsuStars::new(&map).mods(mods).calculate();

        let iter_end = OsuGradualDifficultyAttributes::new(&map, mods)
            .last()
            .expect("empty iter");

        assert_eq!(regular, iter_end);
    }
}
