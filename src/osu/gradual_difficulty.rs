use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    mem,
};

use crate::{curve::CurveBuffers, Beatmap, Mods};

use super::{
    create_skills,
    difficulty_object::{Distances, OsuDifficultyObject},
    old_stacking,
    osu_object::{ObjectParameters, OsuObject, OsuObjectKind},
    scaling_factor::ScalingFactor,
    skills::{Aim, Flashlight, Skill, Speed},
    stacking, OsuDifficultyAttributes, DIFFICULTY_MULTIPLIER, FADE_IN_DURATION_MULTIPLIER,
    PERFORMANCE_BASE_MULTIPLIER, PREEMPT_MIN,
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
pub struct OsuGradualDifficultyAttributes {
    pub(crate) idx: usize,
    mods: u32,
    attrs: OsuDifficultyAttributes,
    // Unused but `diff_objects`' lifetimes secretly depend on it
    #[allow(unused)]
    hit_objects: Vec<OsuObject>,
    diff_objects: Vec<OsuDifficultyObject<'static>>,
    skills: [Box<dyn Skill>; 4],
    hit_window: f64,
}

impl Debug for OsuGradualDifficultyAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("OsuGradualDifficultyAttributes")
            .field("idx", &self.idx)
            .field("attrs", &self.attrs)
            .field("diff_objects", &self.diff_objects)
            .field("skills", &"<cannot be displayed>")
            .finish()
    }
}

impl OsuGradualDifficultyAttributes {
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

        let mut params = ObjectParameters {
            map,
            attrs: &mut attrs,
            ticks: Vec::new(),
            curve_bufs: CurveBuffers::default(),
        };

        let hit_objects_iter = map
            .hit_objects
            .iter()
            .filter_map(|h| OsuObject::new(h, &mut params));

        let mut hit_objects = Vec::with_capacity(map.hit_objects.len());
        hit_objects.extend(hit_objects_iter);

        attrs.n_circles = 0;
        attrs.n_sliders = 0;
        attrs.n_spinners = 0;
        attrs.max_combo = 0;

        let stack_threshold = time_preempt * map.stack_leniency as f64;

        if map.version >= 6 {
            stacking(&mut hit_objects, stack_threshold);
        } else {
            old_stacking(&mut hit_objects, stack_threshold);
        }

        let mut hit_objects_iter = hit_objects.iter_mut().map(|h| {
            h.post_process(hr, &scaling_factor);

            h
        });

        let skills = create_skills(mods, scaling_factor.radius, time_preempt, time_fade_in);

        let last = match hit_objects_iter.next() {
            Some(prev) => prev,
            None => {
                return Self {
                    idx: 0,
                    mods,
                    attrs,
                    hit_objects: Vec::new(),
                    diff_objects: Vec::new(),
                    skills,
                    hit_window,
                }
            }
        };

        Self::increment_combo(last, &mut attrs);

        let mut last_last = None;

        // Prepare `lazy_travel_dist` and `lazy_end_pos` for `last` manually
        Distances::compute_slider_cursor_pos(last, &scaling_factor);

        let mut last = &*last;
        let mut diff_objects = Vec::with_capacity(map.hit_objects.len().saturating_sub(2));

        for (i, curr) in hit_objects_iter.enumerate() {
            let delta_time = (curr.start_time - last.start_time) / clock_rate;

            // * Capped to 25ms to prevent difficulty calculation breaking from simultaneous objects.
            let strain_time = delta_time.max(OsuDifficultyObject::MIN_DELTA_TIME as f64);

            let dists = Distances::new(
                curr,
                last,
                last_last,
                clock_rate,
                strain_time,
                &scaling_factor,
            );

            let diff_obj = OsuDifficultyObject::new(curr, last, clock_rate, i, dists);
            diff_objects.push(diff_obj);

            last_last = Some(last);
            last = &*curr;
        }

        Self {
            idx: 0,
            mods,
            attrs,
            diff_objects: extend_lifetime(diff_objects),
            hit_objects,
            skills,
            hit_window,
        }
    }

    fn increment_combo(h: &OsuObject, attrs: &mut OsuDifficultyAttributes) {
        attrs.max_combo += 1;

        match &h.kind {
            OsuObjectKind::Circle => attrs.n_circles += 1,
            OsuObjectKind::Slider(slider) => {
                attrs.n_sliders += 1;
                attrs.max_combo += slider.nested_len();
            }
            OsuObjectKind::Spinner { .. } => attrs.n_spinners += 1,
        }
    }
}

fn extend_lifetime(
    diff_objects: Vec<OsuDifficultyObject<'_>>,
) -> Vec<OsuDifficultyObject<'static>> {
    // SAFETY: Owned values of the references will be contained
    // in the same struct and hence live just as long as this vec.
    unsafe { mem::transmute(diff_objects) }
}

impl Iterator for OsuGradualDifficultyAttributes {
    type Item = OsuDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.diff_objects.get(self.idx)?;
        self.idx += 1;

        for skill in self.skills.iter_mut() {
            skill.process(curr, &self.diff_objects, self.hit_window);
        }

        Self::increment_combo(curr.base, &mut self.attrs);

        let [aim, aim_no_sliders, speed, flashlight] = &self.skills;

        let mut aim = aim.as_any().downcast_ref::<Aim>().unwrap().clone();

        let mut aim_no_sliders = aim_no_sliders
            .as_any()
            .downcast_ref::<Aim>()
            .unwrap()
            .clone();

        let mut aim_rating = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
        let aim_rating_no_sliders =
            aim_no_sliders.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

        let mut speed = speed.as_any().downcast_ref::<Speed>().unwrap().clone();
        let speed_notes = speed.relevant_note_count();
        let mut speed_rating = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

        let mut flashlight = flashlight
            .as_any()
            .downcast_ref::<Flashlight>()
            .unwrap()
            .clone();

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

        let mut attrs = self.attrs.clone();
        attrs.aim = aim_rating;
        attrs.speed = speed_rating;
        attrs.flashlight = flashlight_rating;
        attrs.slider_factor = slider_factor;
        attrs.stars = star_rating;
        attrs.speed_note_count = speed_notes;

        Some(attrs)
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

            for skill in self.skills.iter_mut() {
                skill.process(curr, &self.diff_objects, self.hit_window);
            }

            Self::increment_combo(curr.base, &mut self.attrs);
        }

        self.next()
    }
}

impl ExactSizeIterator for OsuGradualDifficultyAttributes {
    #[inline]
    fn len(&self) -> usize {
        self.diff_objects.len() - self.idx
    }
}
