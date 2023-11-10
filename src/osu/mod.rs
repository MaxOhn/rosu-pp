mod difficulty_object;
mod osu_object;
mod pp;
mod scaling_factor;
mod score_state;
mod skills;

#[cfg(feature = "gradual")]
mod gradual_difficulty;
#[cfg(feature = "gradual")]
mod gradual_performance;

use crate::{curve::CurveBuffers, parse::Pos2, AnyStars, Beatmap, GameMode, Mods};
use std::pin::Pin;

use self::{
    difficulty_object::{Distances, OsuDifficultyObject},
    skills::{Skill, Skills},
};

pub use self::{osu_object::*, pp::*, score_state::OsuScoreState};

#[cfg(feature = "gradual")]
pub use self::{
    gradual_difficulty::OsuGradualDifficulty,
    gradual_performance::{OsuGradualPerformance, OsuOwnedGradualPerformance},
};

pub(crate) use self::scaling_factor::ScalingFactor;

const SECTION_LEN: f64 = 400.0;
const DIFFICULTY_MULTIPLIER: f64 = 0.0675;
// * Change radius to 50 to make 100 the diameter. Easier for mental maths.
const NORMALIZED_RADIUS: f32 = 50.0;
const STACK_DISTANCE: f32 = 3.0;
// * This is being adjusted to keep the final pp value scaled around what it used to be when changing things.
const PERFORMANCE_BASE_MULTIPLIER: f64 = 1.14;
const PREEMPT_MIN: f64 = 450.0;
const FADE_IN_DURATION_MULTIPLIER: f64 = 0.4;
const PLAYFIELD_BASE_SIZE: Pos2 = Pos2 { x: 512.0, y: 384.0 };

/// Difficulty calculator on osu!standard maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{OsuStars, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let difficulty_attrs = OsuStars::new(&map)
///     .mods(8 + 64) // HDDT
///     .calculate();
///
/// println!("Stars: {}", difficulty_attrs.stars);
/// ```
#[derive(Clone, Debug)]
pub struct OsuStars<'map> {
    pub(crate) map: &'map Beatmap,
    pub(crate) mods: u32,
    pub(crate) passed_objects: Option<usize>,
    pub(crate) clock_rate: Option<f64>,
}

impl<'map> OsuStars<'map> {
    /// Create a new difficulty calculator for osu!standard maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map,
            mods: 0,
            passed_objects: None,
            clock_rate: None,
        }
    }

    /// Convert the map into another mode.
    #[inline]
    pub fn mode(self, mode: GameMode) -> AnyStars<'map> {
        match mode {
            GameMode::Osu => AnyStars::Osu(self),
            GameMode::Taiko => AnyStars::Taiko(self.into()),
            GameMode::Catch => AnyStars::Catch(self.into()),
            GameMode::Mania => AnyStars::Mania(self.into()),
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the difficulty after every few objects, instead of
    /// using [`OsuStars`] multiple times with different `passed_objects`, you should use
    /// [`OsuGradualDifficulty`].
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    #[inline]
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Calculate all difficulty related values, including stars.
    #[inline]
    pub fn calculate(self) -> OsuDifficultyAttributes {
        let mods = self.mods;

        let (skills, mut attrs) = calculate_skills(self);

        let Skills {
            mut aim,
            mut aim_no_sliders,
            mut speed,
            mut flashlight,
        } = skills;

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

        if mods.td() {
            aim_rating = aim_rating.powf(0.8);
            flashlight_rating = flashlight_rating.powf(0.8);
        }

        if mods.rx() {
            aim_rating *= 0.9;
            speed_rating = 0.0;
            flashlight_rating *= 0.7;
        }

        let base_aim_performance = (5.0 * (aim_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;
        let base_speed_performance =
            (5.0 * (speed_rating / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        let base_flashlight_performance = if mods.fl() {
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

        attrs.aim = aim_rating;
        attrs.speed = speed_rating;
        attrs.flashlight = flashlight_rating;
        attrs.slider_factor = slider_factor;
        attrs.stars = star_rating;
        attrs.speed_note_count = speed_notes;

        attrs
    }

    /// Calculate the skill strains.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> OsuStrains {
        let (skills, _) = calculate_skills(self);

        let Skills {
            aim,
            aim_no_sliders,
            speed,
            flashlight,
        } = skills;

        OsuStrains {
            section_len: SECTION_LEN,
            aim: aim.strain_peaks.to_vec(),
            aim_no_sliders: aim_no_sliders.strain_peaks.to_vec(),
            speed: speed.strain_peaks.to_vec(),
            flashlight: flashlight.strain_peaks.to_vec(),
        }
    }
}

/// The result of calculating the strains on a osu! map.
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug)]
pub struct OsuStrains {
    /// Time in ms inbetween two strains.
    pub section_len: f64, // TODO: remove field, make it a method
    /// Strain peaks of the aim skill.
    pub aim: Vec<f64>,
    /// Strain peaks of the aim skill without sliders.
    pub aim_no_sliders: Vec<f64>,
    /// Strain peaks of the speed skill.
    pub speed: Vec<f64>,
    /// Strain peaks of the flashlight skill.
    pub flashlight: Vec<f64>,
}

impl OsuStrains {
    /// Returns the number of strain peaks per skill.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.aim.len()
    }
}

fn calculate_skills(params: OsuStars<'_>) -> (Skills, OsuDifficultyAttributes) {
    let OsuStars {
        map,
        mods,
        passed_objects,
        clock_rate,
    } = params;

    let take = passed_objects.unwrap_or(map.hit_objects.len());
    let clock_rate = clock_rate.unwrap_or_else(|| mods.clock_rate());

    let map_attrs = map.attributes().mods(mods).clock_rate(clock_rate).build();
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

    let mut hit_objects =
        create_osu_objects(map, &mut attrs, &scaling_factor, take, hr, time_preempt);
    let mut hit_objects_iter = hit_objects.iter_mut().map(Pin::new);

    let mut skills = Skills::new(
        mods,
        scaling_factor.radius,
        time_preempt,
        time_fade_in,
        hit_window,
    );

    let Some(mut last) = hit_objects_iter.next() else {
        return (skills, attrs);
    };

    let mut last_last = None;

    // Prepare `lazy_travel_dist` and `lazy_end_pos` for `last` manually
    let last_pos = last.pos();
    let last_stack_offset = last.stack_offset;

    if let OsuObjectKind::Slider(ref mut slider) = last.kind {
        Distances::compute_slider_travel_dist(last_pos, last_stack_offset, slider, &scaling_factor);
    }

    let mut last = last.into_ref();
    let mut diff_objects = Vec::with_capacity(hit_objects_iter.len());

    for (i, mut curr) in hit_objects_iter.enumerate() {
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

    for curr in diff_objects.iter() {
        skills.process(curr, &diff_objects);
    }

    (skills, attrs)
}

pub(crate) fn create_osu_objects(
    map: &Beatmap,
    attrs: &mut OsuDifficultyAttributes,
    scaling_factor: &ScalingFactor,
    take: usize,
    hr: bool,
    time_preempt: f64,
) -> Vec<OsuObject> {
    let mut params = ObjectParameters {
        map,
        attrs,
        ticks: Vec::new(),
        curve_bufs: CurveBuffers::default(),
    };

    let mut hit_objects: Vec<_> = map
        .hit_objects
        .iter()
        .take(take)
        .map(|h| OsuObject::new(h, &mut params))
        .collect();

    let stack_threshold = time_preempt * map.stack_leniency as f64;

    if map.version >= 6 {
        stacking(&mut hit_objects, stack_threshold);
    } else {
        old_stacking(&mut hit_objects, stack_threshold);
    }

    hit_objects
        .iter_mut()
        .for_each(|h| h.post_process(hr, scaling_factor));

    hit_objects
}

fn stacking(hit_objects: &mut [OsuObject], stack_threshold: f64) {
    let mut extended_start_idx = 0;

    let Some(extended_end_idx) = hit_objects.len().checked_sub(1) else {
        return;
    };

    // First big `if` in osu!lazer's function can be skipped

    for i in (1..=extended_end_idx).rev() {
        let mut n = i;
        let mut obj_i_idx = i;
        // * We should check every note which has not yet got a stack.
        // * Consider the case we have two interwound stacks and this will make sense.
        // *   o <-1      o <-2
        // *    o <-3      o <-4
        // * We first process starting from 4 and handle 2,
        // * then we come backwards on the i loop iteration until we reach 3 and handle 1.
        // * 2 and 1 will be ignored in the i loop because they already have a stack value.

        if hit_objects[obj_i_idx].stack_height.abs() > 0.0 || hit_objects[obj_i_idx].is_spinner() {
            continue;
        }

        // * If this object is a hitcircle, then we enter this "special" case.
        // * It either ends with a stack of hitcircles only,
        // * or a stack of hitcircles that are underneath a slider.
        // * Any other case is handled by the "is_slider" code below this.
        if hit_objects[obj_i_idx].is_circle() {
            loop {
                n = match n.checked_sub(1) {
                    Some(n) => n,
                    None => break,
                };

                if hit_objects[n].is_spinner() {
                    continue;
                } else if hit_objects[obj_i_idx].start_time - hit_objects[n].end_time()
                    > stack_threshold
                {
                    break; // * We are no longer within stacking range of the previous object.
                }

                // * HitObjects before the specified update range haven't been reset yet
                if n < extended_start_idx {
                    hit_objects[n].stack_height = 0.0;
                    extended_start_idx = n;
                }

                // * This is a special case where hticircles are moved DOWN and RIGHT (negative stacking)
                // * if they are under the *last* slider in a stacked pattern.
                // *    o==o <- slider is at original location
                // *        o <- hitCircle has stack of -1
                // *         o <- hitCircle has stack of -2
                if hit_objects[n].is_slider()
                    && hit_objects[n]
                        .pre_stacked_end_pos()
                        .distance(hit_objects[obj_i_idx].pos())
                        < STACK_DISTANCE
                {
                    let offset =
                        hit_objects[obj_i_idx].stack_height - hit_objects[n].stack_height + 1.0;

                    for j in n + 1..=i {
                        // * For each object which was declared under this slider, we will offset
                        // * it to appear *below* the slider end (rather than above).
                        if hit_objects[n]
                            .pre_stacked_end_pos()
                            .distance(hit_objects[j].pos())
                            < STACK_DISTANCE
                        {
                            hit_objects[j].stack_height -= offset;
                        }
                    }

                    // * We have hit a slider. We should restart calculation using this as the new base.
                    // * Breaking here will mean that the slider still has StackCount of 0,
                    // * so will be handled in the i-outer-loop.
                    break;
                }

                if hit_objects[n].pos().distance(hit_objects[obj_i_idx].pos()) < STACK_DISTANCE {
                    // * Keep processing as if there are no sliders.
                    // * If we come across a slider, this gets cancelled out.
                    // * NOTE: Sliders with start positions stacking
                    // * are a special case that is also handled here.

                    hit_objects[n].stack_height = hit_objects[obj_i_idx].stack_height + 1.0;
                    obj_i_idx = n;
                }
            }
        } else if hit_objects[obj_i_idx].is_slider() {
            // * We have hit the first slider in a possible stack.
            // * From this point on, we ALWAYS stack positive regardless.
            loop {
                n = match n.checked_sub(1) {
                    Some(n) => n,
                    None => break,
                };

                if hit_objects[n].is_spinner() {
                    continue;
                }

                if hit_objects[obj_i_idx].start_time - hit_objects[n].start_time > stack_threshold {
                    break; // * We are no longer within stacking range of the previous object.
                }

                if hit_objects[n]
                    .pre_stacked_end_pos()
                    .distance(hit_objects[obj_i_idx].pos())
                    < STACK_DISTANCE
                {
                    hit_objects[n].stack_height = hit_objects[obj_i_idx].stack_height + 1.0;
                    obj_i_idx = n;
                }
            }
        }
    }
}

fn old_stacking(hit_objects: &mut [OsuObject], stack_threshold: f64) {
    for i in 0..hit_objects.len() {
        if hit_objects[i].stack_height != 0.0 && !hit_objects[i].is_slider() {
            continue;
        }

        let mut start_time = hit_objects[i].end_time();
        let pos2 = hit_objects[i].old_stacking_pos2();

        let mut slider_stack = 0.0;

        for j in i + 1..hit_objects.len() {
            if hit_objects[j].start_time - stack_threshold > start_time {
                break;
            }

            if hit_objects[j].pos().distance(hit_objects[i].pos()) < STACK_DISTANCE {
                hit_objects[i].stack_height += 1.0;
                start_time = hit_objects[j].end_time();
            } else if hit_objects[j].pos().distance(pos2) < STACK_DISTANCE {
                slider_stack += 1.0;
                hit_objects[j].stack_height -= slider_stack;
                start_time = hit_objects[j].end_time();
            }
        }
    }
}

/// The result of a difficulty calculation on an osu!standard map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsuDifficultyAttributes {
    /// The aim portion of the total strain.
    pub aim: f64,
    /// The speed portion of the total strain.
    pub speed: f64,
    /// The flashlight portion of the total strain.
    pub flashlight: f64,
    /// The ratio of the aim strain with and without considering sliders
    pub slider_factor: f64,
    /// The number of clickable objects weighted by difficulty.
    pub speed_note_count: f64,
    /// The approach rate.
    pub ar: f64,
    /// The overall difficulty
    pub od: f64,
    /// The health drain rate.
    pub hp: f64,
    /// The amount of circles.
    pub n_circles: usize,
    /// The amount of sliders.
    pub n_sliders: usize,
    /// The amount of spinners.
    pub n_spinners: usize,
    /// The final star rating
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: usize,
}

impl OsuDifficultyAttributes {
    /// Return the maximum combo.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.max_combo
    }
}

/// The result of a performance calculation on an osu!standard map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsuPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: OsuDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
    /// The accuracy portion of the final pp.
    pub pp_acc: f64,
    /// The aim portion of the final pp.
    pub pp_aim: f64,
    /// The flashlight portion of the final pp.
    pub pp_flashlight: f64,
    /// The speed portion of the final pp.
    pub pp_speed: f64,
    /// Misses including an approximated amount of slider breaks
    pub effective_miss_count: f64,
}

impl OsuPerformanceAttributes {
    /// Return the star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        self.difficulty.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.difficulty.max_combo
    }
}

impl From<OsuPerformanceAttributes> for OsuDifficultyAttributes {
    #[inline]
    fn from(attributes: OsuPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
