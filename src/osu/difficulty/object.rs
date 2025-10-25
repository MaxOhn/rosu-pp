use std::{borrow::Cow, pin::Pin};

use rosu_map::util::Pos;

use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    osu::object::{OsuObject, OsuObjectKind, OsuSlider},
};

use super::{HD_FADE_OUT_DURATION_MULTIPLIER, scaling_factor::ScalingFactor};

pub struct OsuDifficultyObject<'a> {
    pub idx: usize,
    pub base: &'a OsuObject,
    pub start_time: f64,
    pub delta_time: f64,

    pub strain_time: f64,
    pub lazy_jump_dist: f64,
    pub min_jump_dist: f64,
    pub min_jump_time: f64,
    pub travel_dist: f64,
    pub travel_time: f64,
    pub angle: Option<f64>,
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
        last_last_object: Option<&OsuObject>,
        clock_rate: f64,
        idx: usize,
        scaling_factor: &ScalingFactor,
    ) -> Self {
        let delta_time = (hit_object.start_time - last_object.start_time) / clock_rate;
        let start_time = hit_object.start_time / clock_rate;

        let strain_time = delta_time.max(Self::MIN_DELTA_TIME);

        let mut this = Self {
            idx,
            base: hit_object,
            start_time,
            delta_time,
            strain_time,
            lazy_jump_dist: 0.0,
            min_jump_dist: 0.0,
            min_jump_time: 0.0,
            travel_dist: 0.0,
            travel_time: 0.0,
            angle: None,
        };

        this.set_distances(last_object, last_last_object, clock_rate, scaling_factor);

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
        let fade_in_duration = time_fade_in;

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

    pub fn get_doubletapness(&self, next: Option<&Self>, hit_window: f64) -> f64 {
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
        let window_ratio = (curr_delta_time / hit_window).min(1.0).powf(2.0);

        1.0 - (speed_ratio).powf(1.0 - window_ratio)
    }

    fn set_distances(
        &mut self,
        last_object: &OsuObject,
        last_last_object: Option<&OsuObject>,
        clock_rate: f64,
        scaling_factor: &ScalingFactor,
    ) {
        if let OsuObjectKind::Slider(ref slider) = self.base.kind {
            self.travel_dist = f64::from(
                slider.lazy_travel_dist
                    * ((1.0 + slider.repeat_count() as f64 / 2.5).powf(1.0 / 2.5)) as f32,
            );

            self.travel_time = (self.base.lazy_travel_time() / clock_rate)
                .max(OsuDifficultyObject::MIN_DELTA_TIME);
        }

        if self.base.is_spinner() || last_object.is_spinner() {
            return;
        }

        let scaling_factor = scaling_factor.factor;

        let last_cursor_pos = Self::get_end_cursor_pos(last_object);

        self.lazy_jump_dist = f64::from(
            (self.base.stacked_pos() * scaling_factor - last_cursor_pos * scaling_factor).length(),
        );
        self.min_jump_time = self.strain_time;
        self.min_jump_dist = self.lazy_jump_dist;

        if let OsuObjectKind::Slider(ref last_slider) = last_object.kind {
            let last_travel_time = (last_object.lazy_travel_time() / clock_rate)
                .max(OsuDifficultyObject::MIN_DELTA_TIME);
            self.min_jump_time =
                (self.strain_time - last_travel_time).max(OsuDifficultyObject::MIN_DELTA_TIME);

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

        if let Some(last_last_object) = last_last_object.filter(|h| !h.is_spinner()) {
            let last_last_cursor_pos = Self::get_end_cursor_pos(last_last_object);

            let v1 = last_last_cursor_pos - last_object.stacked_pos();
            let v2 = self.base.stacked_pos() - last_cursor_pos;

            let dot = v1.dot(v2);
            let det = v1.x * v2.y - v1.y * v2.x;

            self.angle = Some((f64::from(det).atan2(f64::from(dot))).abs());
        }
    }

    /// The [`Pin<&mut OsuObject>`](std::pin::Pin) denotes that the object will
    /// be mutated but not moved.
    pub fn compute_slider_cursor_pos(
        mut h: Pin<&mut OsuObject>,
        radius: f64,
    ) -> Pin<&mut OsuObject> {
        let pos = h.pos;
        let stack_offset = h.stack_offset;
        let start_time = h.start_time;

        let OsuObjectKind::Slider(ref mut slider) = h.kind else {
            return h;
        };

        let mut nested = Cow::Borrowed(slider.nested_objects.as_slice());
        let duration = slider.end_time - start_time;
        OsuSlider::lazy_travel_time(start_time, duration, &mut nested);
        let nested = nested.as_ref();

        let mut curr_cursor_pos = pos + stack_offset;
        let scaling_factor = f64::from(OsuDifficultyObject::NORMALIZED_RADIUS) / radius;

        for (curr_movement_obj, i) in nested.iter().zip(1..) {
            let mut curr_movement = curr_movement_obj.pos + stack_offset - curr_cursor_pos;
            let mut curr_movement_len = scaling_factor * f64::from(curr_movement.length());
            let mut required_movement = f64::from(OsuDifficultyObject::ASSUMED_SLIDER_RADIUS);

            if i == nested.len() {
                let lazy_movement = slider.lazy_end_pos - curr_cursor_pos;

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
                slider.lazy_travel_dist += curr_movement_len as f32;
            }

            if i == nested.len() {
                slider.lazy_end_pos = curr_cursor_pos;
            }
        }

        h
    }

    const fn get_end_cursor_pos(hit_object: &OsuObject) -> Pos {
        if let OsuObjectKind::Slider(ref slider) = hit_object.kind {
            // We don't have access to the slider's curve at this point so we
            // take the pre-computed value.
            slider.lazy_end_pos
        } else {
            hit_object.stacked_pos()
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
