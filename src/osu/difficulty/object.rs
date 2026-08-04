use std::borrow::Cow;

use rosu_map::util::Pos;

use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    osu::object::{OsuObject, OsuObjectKind},
    util::difficulty::reverse_lerp,
};

use super::{HD_FADE_OUT_DURATION_MULTIPLIER, scaling_factor::ScalingFactor};

pub struct OsuDifficultyObject<'a> {
    pub idx: usize,
    pub base: &'a OsuObject,

    /// Start time (clock rate adjusted)
    pub start_time: f64,
    /// End time (clock rate adjusted)
    pub end_time: f64,
    /// Amount of time elapsed between the last and current [`OsuObject`] (clock rate adjusted)
    pub delta_time: f64,
    /// [`delta_time`] capped to [`MIN_DELTA_TIME`]
    pub adjusted_delta_time: f64,
    /// Amount of time elapsed between last and current [`OsuDifficultyObject`] capped to [`MIN_DELTA_TIME`]
    pub last_obj_end_delta_time: f64,

    pub jump_dist: f64,
    pub lazy_jump_dist: f64,
    pub min_jump_dist: f64,
    pub min_jump_time: f64,
    pub travel_dist: f64,
    pub travel_time: f64,
    pub lazy_end_pos: Option<Pos>,
    pub lazy_travel_dist: f64,
    pub lazy_travel_time: f64,

    pub angle: Option<f64>,
    pub normalized_vector_angle: Option<f64>,

    pub small_circle_bonus: f64,
}

impl<'a> OsuDifficultyObject<'a> {
    pub const NORMALIZED_RADIUS: i32 = 50;
    pub const NORMALIZED_DIAMETER: i32 = Self::NORMALIZED_RADIUS * 2;

    pub const MIN_DELTA_TIME: f64 = 25.0;
    const MAX_SLIDER_RADIUS: f32 = Self::NORMALIZED_RADIUS as f32 * 2.4;
    const ASSUMED_SLIDER_RADIUS: f32 = Self::NORMALIZED_RADIUS as f32 * 1.8;

    pub fn new(
        hit_object: &'a OsuObject,
        last_object: &'a OsuObject,
        last_diff_obj: Option<&OsuDifficultyObject>,
        last_last_diff_obj: Option<&OsuDifficultyObject>,
        clock_rate: f64,
        idx: usize,
        scaling_factor: &ScalingFactor,
    ) -> Self {
        let delta_time = (hit_object.start_time - last_object.start_time) / clock_rate;
        let start_time = hit_object.start_time / clock_rate;
        let end_time = hit_object.end_time() / clock_rate;

        // * Capped to 25ms to prevent difficulty calculation breaking from simultaneous objects.
        let adjusted_delta_time = delta_time.max(Self::MIN_DELTA_TIME);
        let last_obj_end_delta_time = if let Some(last) = last_diff_obj {
            (start_time - last.end_time).max(Self::MIN_DELTA_TIME)
        } else {
            adjusted_delta_time
        };

        let small_circle_bonus = (1.0 + (30.0 - scaling_factor.radius) / 70.0).max(1.0);

        let mut this = Self {
            idx,
            base: hit_object,
            start_time,
            end_time,
            delta_time,
            adjusted_delta_time,
            last_obj_end_delta_time,
            jump_dist: 0.0,
            lazy_jump_dist: 0.0,
            min_jump_dist: 0.0,
            min_jump_time: 0.0,
            travel_dist: 0.0,
            travel_time: 0.0,
            lazy_end_pos: None,
            lazy_travel_dist: 0.0,
            lazy_travel_time: 0.0,
            angle: None,
            normalized_vector_angle: None,
            small_circle_bonus,
        };

        this.compute_slider_cursor_pos(scaling_factor.radius);
        this.set_distances(
            last_object,
            last_diff_obj,
            last_last_diff_obj,
            clock_rate,
            scaling_factor,
        );

        this
    }

    pub fn opacity_at(&self, time: f64, hidden: bool, time_preempt: f64, time_fade_in: f64) -> f64 {
        if time > self.base.start_time {
            // * Consider a hitobject as being invisible when its start time is passed.
            // * In reality the hitobject will be visible beyond its start time up until its hittable window has passed,
            // * but this is an approximation and such a case is unlikely to be hit where this function is used.
            return 0.0;
        }

        let fade_in_start_time = self.base.start_time - time_preempt;

        // * Equal to `OsuHitObject.TimeFadeIn` minus any adjustments from the HD mod.
        let fade_in_duration = 400.0 * (time_preempt / OsuObject::PREEMPT_MIN).min(1.0);

        if hidden {
            // * Taken from OsuModHidden.
            let fade_out_start_time = self.base.start_time - time_preempt + time_fade_in;
            let fade_out_duration = time_preempt * HD_FADE_OUT_DURATION_MULTIPLIER;

            (((time - fade_in_start_time) / fade_in_duration).clamp(0.0, 1.0))
                .min(1.0 - ((time - fade_out_start_time) / fade_out_duration).clamp(0.0, 1.0))
        } else {
            ((time - fade_in_start_time) / fade_in_duration).clamp(0.0, 1.0)
        }
    }

    pub fn calc_double_tap_feasibility(&self, next: Option<&Self>, hit_window: f64) -> f64 {
        let Some(next) = next else { return 0.0 };

        let hit_window = if self.base.is_spinner() {
            0.0
        } else {
            hit_window
        };

        let curr_delta_time = self.delta_time.max(1.0);
        let next_delta_time = next.delta_time.max(1.0);
        let delta_diff = (next_delta_time - curr_delta_time).abs();

        let speed_ratio = curr_delta_time / curr_delta_time.max(delta_diff);
        let window_ratio = (curr_delta_time / hit_window).min(1.0).powf(5.0);

        // * Can't doubletap if circles don't intersect
        let distance_factor = reverse_lerp(
            self.lazy_jump_dist,
            f64::from(Self::NORMALIZED_DIAMETER),
            f64::from(Self::NORMALIZED_RADIUS),
        )
        .powf(2.0);

        1.0 - speed_ratio.powf(distance_factor * (1.0 - window_ratio))
    }

    fn set_distances(
        &mut self,
        last_object: &OsuObject,
        last_diff_obj: Option<&OsuDifficultyObject>,
        last_last_diff_obj: Option<&OsuDifficultyObject>,
        clock_rate: f64,
        scaling_factor: &ScalingFactor,
    ) {
        if let OsuObjectKind::Slider(ref slider) = self.base.kind {
            // * Bonus for repeat sliders until a better per nested object strain system can be achieved.
            self.travel_dist =
                self.lazy_travel_dist * (slider.repeat_count() as f64).powf(0.3).max(1.0);

            self.travel_time =
                (self.lazy_travel_time / clock_rate).max(OsuDifficultyObject::MIN_DELTA_TIME);
        }

        self.min_jump_time = self.adjusted_delta_time;

        if self.base.is_spinner() || last_object.is_spinner() {
            return;
        }

        let scaling_factor = scaling_factor.factor;

        let mut last_cursor_pos = if let Some(last_diff_obj) = last_diff_obj {
            Self::get_end_cursor_pos(last_diff_obj)
        } else {
            last_object.stacked_pos()
        };

        self.jump_dist = f64::from(
            (last_object.stacked_pos() - self.base.stacked_pos()).length() * scaling_factor,
        );
        self.lazy_jump_dist =
            f64::from((self.base.stacked_pos() - last_cursor_pos).length() * scaling_factor);
        self.min_jump_dist = self.lazy_jump_dist;

        let Some(last_diff_obj) = last_diff_obj else {
            return;
        };

        if let OsuObjectKind::Slider(ref last_slider) = last_object.kind {
            let last_travel_time = (last_diff_obj.lazy_travel_time / clock_rate)
                .max(OsuDifficultyObject::MIN_DELTA_TIME);
            self.min_jump_time = (self.adjusted_delta_time - last_travel_time)
                .max(OsuDifficultyObject::MIN_DELTA_TIME);

            let tail_pos = last_slider.tail().map_or(last_object.pos, |tail| tail.pos);
            let stacked_tail_pos = tail_pos + last_object.stack_offset;

            let tail_jump_dist =
                (stacked_tail_pos - self.base.stacked_pos()).length() * scaling_factor;

            let diff = f64::from(
                OsuDifficultyObject::MAX_SLIDER_RADIUS - OsuDifficultyObject::ASSUMED_SLIDER_RADIUS,
            );

            let min = f64::from(tail_jump_dist - OsuDifficultyObject::MAX_SLIDER_RADIUS);
            self.min_jump_dist = ((self.lazy_jump_dist - diff).min(min)).max(0.0);
        }

        if let Some(last_last_diff_obj) = last_last_diff_obj.filter(|ob| !ob.base.is_spinner()) {
            if last_object.is_slider() && last_diff_obj.travel_dist > 0.0 {
                last_cursor_pos = last_object.stacked_pos();
            }

            let last_last_cursor_pos = Self::get_end_cursor_pos(last_last_diff_obj);

            let angle = self.calculate_angle(last_cursor_pos, last_last_cursor_pos);
            let slider_angle = self.calculate_slider_angle(last_diff_obj, last_last_cursor_pos);
            let v = self.base.stacked_pos() - last_cursor_pos;

            self.normalized_vector_angle = Some(f64::from(v.y).abs().atan2(f64::from(v.x).abs()));
            self.angle = Some(angle.min(slider_angle));
        }
    }

    pub fn compute_slider_cursor_pos(&mut self, radius: f64) {
        const TAIL_LENIENCY: f64 = -36.0;

        let OsuObjectKind::Slider(ref slider) = self.base.kind else {
            return;
        };

        if self.lazy_end_pos.is_some() {
            return;
        }

        let pos = self.base.pos;
        let stack_offset = self.base.stack_offset;
        let start_time = self.base.start_time;
        let duration = self.base.duration();

        let mut nested_objects = Cow::Borrowed(slider.nested_objects.as_slice());

        let mut tracking_end_time =
            (start_time + duration + TAIL_LENIENCY).max(start_time + duration / 2.0);

        let last_real_tick = nested_objects
            .iter()
            .enumerate()
            .rfind(|(_, nested)| nested.is_tick());

        if let Some((idx, last_real_tick)) =
            last_real_tick.filter(|(_, tick)| tick.start_time > tracking_end_time)
        {
            tracking_end_time = last_real_tick.start_time;

            // * When the last tick falls after the tracking end time, we need to re-sort the nested objects
            // * based on time. This creates a somewhat weird ordering which is counter to how a user would
            // * understand the slider, but allows a zero-diff with known diffcalc output.
            // *
            // * To reiterate, this is definitely not correct from a difficulty calculation perspective
            // * and should be revisited at a later date (likely by replacing this whole code with the commented
            // * version above).
            nested_objects.to_mut()[idx..].rotate_left(1);
        }

        self.lazy_travel_time = tracking_end_time - start_time;

        let nested_objects = nested_objects.as_ref();

        let span_duration = duration / slider.span_count;

        let mut end_time_min = self.lazy_travel_time / span_duration;

        if end_time_min % 2.0 >= 1.0 {
            end_time_min = 1.0 - end_time_min % 1.0;
        } else {
            end_time_min %= 1.0;
        }

        // * temporary lazy end position until a real result can be derived.
        let mut lazy_end_pos = pos + stack_offset + slider.path.position_at(end_time_min);

        let mut curr_cursor_pos = pos + stack_offset;

        // * lazySliderDistance is coded to be sensitive to scaling, this makes the maths easier with the thresholds being used.
        let scaling_factor = f64::from(OsuDifficultyObject::NORMALIZED_RADIUS) / radius;

        for (curr_movement_obj, i) in nested_objects.iter().zip(1..) {
            let mut curr_movement = curr_movement_obj.pos + stack_offset - curr_cursor_pos;
            let mut curr_movement_len = scaling_factor * f64::from(curr_movement.length());
            let mut required_movement = f64::from(OsuDifficultyObject::ASSUMED_SLIDER_RADIUS);

            if i == nested_objects.len() {
                // * The end of a slider has special aim rules due to the relaxed time constraint on position.
                // * There is both a lazy end position as well as the actual end slider position. We assume the player takes the simpler movement.
                // * For sliders that are circular, the lazy end position may actually be farther away than the sliders true end.
                // * This code is designed to prevent buffing situations where lazy end is actually a less efficient movement.
                let lazy_movement = lazy_end_pos - curr_cursor_pos;

                if lazy_movement.length() < curr_movement.length() {
                    curr_movement = lazy_movement;
                }

                curr_movement_len = scaling_factor * f64::from(curr_movement.length());
            } else if curr_movement_obj.is_repeat() {
                required_movement = f64::from(OsuDifficultyObject::NORMALIZED_RADIUS);
            }

            if curr_movement_len > required_movement {
                curr_cursor_pos += curr_movement
                    * ((curr_movement_len - required_movement) / curr_movement_len) as f32;
                curr_movement_len *= (curr_movement_len - required_movement) / curr_movement_len;
                self.lazy_travel_dist += curr_movement_len;
            }

            if i == nested_objects.len() {
                lazy_end_pos = curr_cursor_pos;
            }
        }

        self.lazy_end_pos = Some(lazy_end_pos);
    }

    fn calculate_slider_angle(
        &self,
        last_diff_obj: &OsuDifficultyObject,
        last_last_pos: Pos,
    ) -> f64 {
        let last_pos = Self::get_end_cursor_pos(last_diff_obj);

        let last_last_pos = if let OsuObjectKind::Slider(ref last_slider) = last_diff_obj.base.kind
            && last_diff_obj.travel_dist > 0.0
        {
            let m = last_slider.nested_objects.len();

            if m >= 2 {
                last_slider.nested_objects[m - 2].pos + last_diff_obj.base.stack_offset
            } else {
                // C#'s NestedHitObjects[^2] would resolve to the head circle here —
                // which isn't present in `nested_objects`, so fall back to the base object.
                last_diff_obj.base.stacked_pos()
            }
        } else {
            last_last_pos
        };

        self.calculate_angle(last_pos, last_last_pos)
    }

    fn calculate_angle(&self, last_pos: Pos, last_last_pos: Pos) -> f64 {
        let v1 = last_last_pos - last_pos;
        let v2 = self.base.stacked_pos() - last_pos;
        let dot = v1.dot(v2);
        let det = v1.x * v2.y - v1.y * v2.x;

        f64::from(det).atan2(f64::from(dot)).abs()
    }

    const fn get_end_cursor_pos(hit_object: &OsuDifficultyObject) -> Pos {
        if let Some(lazy_end_pos) = hit_object.lazy_end_pos {
            lazy_end_pos
        } else {
            hit_object.base.stacked_pos()
        }
    }
}

impl IDifficultyObject for OsuDifficultyObject<'_> {
    type DifficultyObjects = [Self];

    fn idx(&self) -> usize {
        self.idx
    }
}

impl HasStartTime for OsuDifficultyObject<'_> {
    fn start_time(&self) -> f64 {
        self.start_time
    }
}
