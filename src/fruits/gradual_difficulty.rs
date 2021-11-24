use std::{iter, slice::Iter};

use crate::{
    curve::CurveBuffers,
    fruits::{
        difficulty_object::DifficultyObject, slider_state::SliderState, SECTION_LENGTH,
        STAR_SCALING_FACTOR,
    },
    parse::{HitObject, Pos2},
    Beatmap, Mods,
};

use super::{
    calculate_catch_width,
    catch_object::CatchObject,
    fruit_or_juice::{FruitOrJuice, FruitParams},
    movement::Movement,
    FruitsDifficultyAttributes, ALLOWED_CATCH_RANGE,
};

/// Gradually calculate the difficulty attributes of an osu!ctb map.
///
/// Note that this struct implements [`Iterator`](std::iter::Iterator).
/// On every call of [`Iterator::next`](std::iter::Iterator::next), the map's next fruit or droplet
/// will be processed and the [`FruitsDifficultyAttributes`] will be updated and returned.
///
/// Note that it does not return attributes after a tiny droplet. Only for fruits and droplets.
///
/// If you want to calculate performance attributes, use
/// [`FruitsGradualPerformanceAttributes`](crate::fruits::FruitsGradualPerformanceAttributes) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, fruits::FruitsGradualDifficultyAttributes};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = FruitsGradualDifficultyAttributes::new(&map, mods);
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
pub struct FruitsGradualDifficultyAttributes<'map> {
    pub(crate) idx: usize,
    clock_rate: f64,
    hit_objects: FruitsObjectIter<'map>,
    movement: Movement,
    prev: CatchObject,
    half_catcher_width: f64,
    last_direction: i8,
    last_excess: f64,
    curr_section_end: f64,
    strain_peak_buf: Vec<f64>,
}

impl<'map> FruitsGradualDifficultyAttributes<'map> {
    /// Create a new difficulty attributes iterator for osu!ctb maps.
    pub fn new(map: &'map Beatmap, mods: impl Mods) -> Self {
        let map_attributes = map.attributes().mods(mods);

        let attributes = FruitsDifficultyAttributes {
            ar: map_attributes.ar,
            ..Default::default()
        };

        let hit_objects = FruitsObjectIter::new(map, mods, attributes);

        let half_catcher_width =
            (calculate_catch_width(map_attributes.cs as f32) / 2.0 / ALLOWED_CATCH_RANGE) as f64;
        let last_direction = 0;
        let last_excess = half_catcher_width;

        let movement = Movement::new(map_attributes.cs as f32);
        let prev = CatchObject::new((Pos2::zero(), 0.0));

        Self {
            idx: 0,
            clock_rate: mods.speed(),
            hit_objects,
            movement,
            prev,
            half_catcher_width,
            last_direction,
            last_excess,
            curr_section_end: 0.0,
            strain_peak_buf: Vec::new(),
        }
    }

    fn init_hyper_dash(&mut self, next: &CatchObject) {
        self.prev.init_hyper_dash(
            self.half_catcher_width,
            next,
            &mut self.last_direction,
            &mut self.last_excess,
        );
    }
}

impl Iterator for FruitsGradualDifficultyAttributes<'_> {
    type Item = FruitsDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.hit_objects.next()?;
        self.idx += 1;

        if self.idx == 1 {
            self.prev = curr;

            return Some(self.hit_objects.attributes());
        }

        self.init_hyper_dash(&curr);

        let h = DifficultyObject::new(
            &curr,
            &self.prev,
            self.movement.half_catcher_width,
            self.clock_rate,
        );

        if self.idx == 2 {
            self.curr_section_end =
                (h.base.time / self.clock_rate / SECTION_LENGTH).ceil() * SECTION_LENGTH;
        } else {
            let base_time = h.base.time / self.clock_rate;

            while base_time > self.curr_section_end {
                self.movement.save_current_peak();
                self.movement.start_new_section_from(self.curr_section_end);
                self.curr_section_end += SECTION_LENGTH;
            }
        }

        self.movement.process(&h);
        self.prev = curr;

        let len = self.movement.strain_peaks.len();
        let missing = len + 1 - self.strain_peak_buf.len();
        self.strain_peak_buf.extend(iter::repeat(0.0).take(missing));

        self.strain_peak_buf[..len].copy_from_slice(&self.movement.strain_peaks);

        if let Some(last) = self.strain_peak_buf.last_mut() {
            *last = self.movement.curr_section_peak;
        }

        let mut attributes = self.hit_objects.attributes();
        attributes.stars =
            Movement::difficulty_value(&mut self.strain_peak_buf).sqrt() * STAR_SCALING_FACTOR;

        Some(attributes)
    }
}

#[derive(Clone, Debug)]
struct FruitsObjectIter<'map> {
    last_object: Option<FruitOrJuice>,
    hit_objects: Iter<'map, HitObject>,
    params: FruitParams<'map>,
}

impl<'map> FruitsObjectIter<'map> {
    fn new(map: &'map Beatmap, mods: impl Mods, attributes: FruitsDifficultyAttributes) -> Self {
        let params = FruitParams {
            attributes,
            curve_bufs: CurveBuffers::default(),
            last_pos: None,
            last_time: 0.0,
            map,
            slider_state: SliderState::new(map),
            ticks: Vec::new(),
            with_hr: mods.hr(),
        };

        Self {
            last_object: None,
            hit_objects: map.hit_objects.iter(),
            params,
        }
    }

    fn attributes(&self) -> FruitsDifficultyAttributes {
        self.params.attributes.clone()
    }
}

impl Iterator for FruitsObjectIter<'_> {
    type Item = CatchObject;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(h) = self.last_object.as_mut().and_then(Iterator::next) {
            return Some(h);
        }

        for h in &mut self.hit_objects {
            if let Some(h) = FruitOrJuice::new(h, &mut self.params) {
                return self.last_object.insert(h).next();
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map() {
        let map = Beatmap::default();
        let mut attributes = FruitsGradualDifficultyAttributes::new(&map, 0);
        assert!(attributes.next().is_none());
    }

    #[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
    #[test]
    fn iter_end_eq_regular() {
        let map = Beatmap::from_path("./maps/2118524.osu").expect("failed to parse map");
        let mods = 64;
        let regular = crate::fruits::stars(&map, mods, None);

        let iter_end = FruitsGradualDifficultyAttributes::new(&map, mods)
            .last()
            .expect("empty iter");

        assert_eq!(regular, iter_end);
    }
}
