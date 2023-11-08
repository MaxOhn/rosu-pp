#![cfg(feature = "gradual")]

use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    mem,
    pin::Pin,
};

use crate::{Beatmap, Mods};

use self::osu_objects::OsuObjects;

use super::{
    difficulty_object::{Distances, OsuDifficultyObject},
    osu_object::{OsuObject, OsuObjectKind},
    scaling_factor::ScalingFactor,
    skills::{Skill, Skills},
    OsuDifficultyAttributes, DIFFICULTY_MULTIPLIER, FADE_IN_DURATION_MULTIPLIER,
    PERFORMANCE_BASE_MULTIPLIER, PREEMPT_MIN,
};

/// Gradually calculate the difficulty attributes of an osu!standard map.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`], the map's next hit object will
/// be processed and the [`OsuDifficultyAttributes`] will be updated and returned.
///
/// If you want to calculate performance attributes, use
/// [`OsuGradualPerformance`](crate::osu::OsuGradualPerformance) instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, osu::OsuGradualDifficulty};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let mods = 64; // DT
/// let mut iter = OsuGradualDifficulty::new(&map, mods);
///
/// let attrs1 = iter.next(); // the difficulty of the map after the first hit object
/// let attrs2 = iter.next(); //                           after the second hit object
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
pub struct OsuGradualDifficulty {
    pub(crate) idx: usize,
    mods: u32,
    attrs: OsuDifficultyAttributes,
    // Lifetimes actually depend on `_osu_objects` so this type is self-referential.
    // This field must be treated with great caution, moving `_osu_objects` will immediately
    // invalidate `diff_objects`.
    diff_objects: Vec<OsuDifficultyObject<'static>>,
    osu_objects: OsuObjects,
    skills: Skills,
}

impl Debug for OsuGradualDifficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("OsuGradualDifficultyAttributes")
            .field("idx", &self.idx)
            .field("attrs", &self.attrs)
            .field("diff_objects", &self.diff_objects)
            .field("skills", &self.skills)
            .finish()
    }
}

impl OsuGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!standard maps.
    pub fn new(map: &Beatmap, mods: u32) -> Self {
        let clock_rate = mods.clock_rate();
        let map_attrs = map.attributes().mods(mods).build();
        let scaling_factor = ScalingFactor::new(map_attrs.cs);
        let hr = mods.hr();
        let hit_window = 2.0 * map_attrs.hit_windows.od;
        let time_preempt = (map_attrs.hit_windows.ar * clock_rate) as f32 as f64;

        // * Preempt time can go below 450ms. Normally, this is achieved via the DT mod
        // * which uniformly speeds up all animations game wide regardless of AR.
        // * This uniform speedup is hard to match 1:1, however we can at least make
        // * AR>10 (via mods) feel good by extending the upper linear function above.
        // * Note that this doesn't exactly match the AR>10 visuals as they're
        // * classically known, but it feels good.
        // * This adjustment is necessary for AR>10, otherwise TimePreempt can
        // * become smaller leading to hitcircles not fully fading in.
        let time_fade_in = if mods.hd() {
            time_preempt * FADE_IN_DURATION_MULTIPLIER
        } else {
            400.0 * (time_preempt / PREEMPT_MIN).min(1.0)
        };

        let mut attrs = OsuDifficultyAttributes {
            ar: map_attrs.ar,
            hp: map_attrs.hp,
            od: map_attrs.od,
            ..Default::default()
        };

        let hit_objects = crate::osu::create_osu_objects(
            map,
            &mut attrs,
            &scaling_factor,
            map.hit_objects.len(),
            hr,
            time_preempt,
        );

        let mut osu_objects = OsuObjects::new(hit_objects);

        attrs.n_circles = 0;
        attrs.n_sliders = 0;
        attrs.n_spinners = 0;
        attrs.max_combo = 0;

        let skills = Skills::new(
            mods,
            scaling_factor.radius,
            time_preempt,
            time_fade_in,
            hit_window,
        );

        let mut osu_objects_iter = osu_objects.iter_mut();

        let Some(mut last) = osu_objects_iter.next() else {
            return Self {
                idx: 0,
                mods,
                attrs,
                diff_objects: Vec::new(),
                osu_objects: OsuObjects::new(Vec::new()),
                skills,
            };
        };

        Self::increment_combo(last.as_ref().get_ref(), &mut attrs);

        let mut last_last = None;

        // Prepare `lazy_travel_dist` and `lazy_end_pos` for `last` manually
        let last_pos = last.pos();
        let last_stack_offset = last.stack_offset;

        if let OsuObjectKind::Slider(ref mut slider) = last.kind {
            Distances::compute_slider_travel_dist(
                last_pos,
                last_stack_offset,
                slider,
                &scaling_factor,
            );
        }

        let mut last = last.into_ref();
        let mut diff_objects = Vec::with_capacity(map.hit_objects.len().saturating_sub(2));

        for (i, mut curr) in osu_objects_iter.enumerate() {
            let delta_time = (curr.start_time - last.start_time) / clock_rate;

            // * Capped to 25ms to prevent difficulty calculation breaking from simultaneous objects.
            let strain_time = delta_time.max(OsuDifficultyObject::MIN_DELTA_TIME as f64);

            let dists = Distances::new(
                &mut curr,
                last.get_ref(),
                last_last.map(Pin::get_ref),
                clock_rate,
                strain_time,
                &scaling_factor,
            );

            let curr = curr.into_ref();

            let diff_obj = OsuDifficultyObject::new(curr, last.get_ref(), clock_rate, i, dists);
            diff_objects.push(diff_obj);

            last_last = Some(last);
            last = curr;
        }

        Self {
            idx: 0,
            mods,
            attrs,
            diff_objects: extend_lifetime(diff_objects),
            osu_objects,
            skills,
        }
    }

    fn increment_combo(h: &OsuObject, attrs: &mut OsuDifficultyAttributes) {
        attrs.max_combo += 1;

        match &h.kind {
            OsuObjectKind::Circle => attrs.n_circles += 1,
            OsuObjectKind::Slider(slider) => {
                attrs.n_sliders += 1;
                attrs.max_combo += slider.nested_objects.len();
            }
            OsuObjectKind::Spinner { .. } => attrs.n_spinners += 1,
        }
    }
}

fn extend_lifetime(
    diff_objects: Vec<OsuDifficultyObject<'_>>,
) -> Vec<OsuDifficultyObject<'static>> {
    // SAFETY: Owned values of the references will be contained in the same struct (same lifetime).
    // Also, the only mutable access wraps them in `Pin` to ensure that they won't move.
    unsafe { mem::transmute(diff_objects) }
}

impl Iterator for OsuGradualDifficulty {
    type Item = OsuDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        // The first difficulty object belongs to the second note since each difficulty
        // object requires the current and the last note. Hence, if we're still on the first
        // object, we don't have a difficulty object yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;
            self.skills.process(curr, &self.diff_objects);
            Self::increment_combo(curr.base.get_ref(), &mut self.attrs);
        } else if self.osu_objects.is_empty() {
            return None;
        }

        self.idx += 1;

        let Skills {
            mut aim,
            mut aim_no_sliders,
            mut speed,
            mut flashlight,
        } = self.skills.clone();

        let mut aim_rating = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
        let aim_rating_no_sliders =
            aim_no_sliders.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

        let speed_notes = speed.relevant_note_count();
        let mut speed_rating = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

        let mut flashlight_rating = flashlight.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

        let slider_factor = if aim_rating > 0.0 {
            aim_rating_no_sliders / aim_rating
        } else {
            1.0
        };

        if self.mods.td() {
            aim_rating = aim_rating.powf(0.8);
            flashlight_rating = flashlight_rating.powf(0.8);
        }

        if self.mods.rx() {
            aim_rating *= 0.9;
            speed_rating = 0.0;
            flashlight_rating *= 0.7;
        }

        let base_aim_performance = (5.0 * (aim_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;
        let base_speed_performance =
            (5.0 * (speed_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let base_flashlight_performance = if self.mods.fl() {
            flashlight_rating * flashlight_rating * 25.0
        } else {
            0.0
        };

        let base_performance = ((base_aim_performance).powf(1.1)
            + (base_speed_performance).powf(1.1)
            + (base_flashlight_performance).powf(1.1))
        .powf(1.0 / 1.1);

        let star_rating = if base_performance > 0.00001 {
            PERFORMANCE_BASE_MULTIPLIER.cbrt()
                * 0.027
                * ((100_000.0 / 2.0_f64.powf(1.0 / 1.1) * base_performance).cbrt() + 4.0)
        } else {
            0.0
        };

        let attrs = OsuDifficultyAttributes {
            aim: aim_rating,
            speed: speed_rating,
            flashlight: flashlight_rating,
            slider_factor,
            stars: star_rating,
            speed_note_count: speed_notes,
            ..self.attrs.clone()
        };

        Some(attrs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let skip_iter = self.diff_objects.iter().skip(self.idx.saturating_sub(1));

        let mut take = n.min(self.len().saturating_sub(1));

        // The first note has no difficulty object
        if self.idx == 0 && take > 0 {
            take -= 1;
            self.idx += 1;
        }

        for curr in skip_iter.take(take) {
            self.skills.process(curr, &self.diff_objects);
            Self::increment_combo(curr.base.get_ref(), &mut self.attrs);
            self.idx += 1;
        }

        self.next()
    }
}

impl ExactSizeIterator for OsuGradualDifficulty {
    #[inline]
    fn len(&self) -> usize {
        self.diff_objects.len() + 1 - self.idx
    }
}

mod osu_objects {
    use crate::osu::OsuObject;
    use std::pin::Pin;

    // Wrapper to ensure that the data will not be moved
    pub(super) struct OsuObjects {
        objects: Box<[OsuObject]>,
    }

    impl OsuObjects {
        pub(super) fn new(objects: Vec<OsuObject>) -> Self {
            Self {
                objects: objects.into_boxed_slice(),
            }
        }

        pub(super) fn is_empty(&self) -> bool {
            self.objects.is_empty()
        }

        pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = Pin<&mut OsuObject>> {
            self.objects.iter_mut().map(Pin::new)
        }
    }
}
