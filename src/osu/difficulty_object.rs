use crate::{
    osu::osu_object::{NestedObjectKind, OsuObjectKind},
    parse::Pos2,
};

use super::{osu_object::OsuSlider, OsuObject, ScalingFactor};

#[derive(Clone, Debug)]
pub(crate) struct OsuDifficultyObject<'h> {
    pub(crate) start_time: f64,
    pub(crate) delta_time: f64,
    pub(crate) base: &'h OsuObject,
    pub(crate) strain_time: f64,
    pub(crate) dists: Distances,
    pub(crate) idx: usize,
}

impl<'h> OsuDifficultyObject<'h> {
    pub(crate) const MIN_DELTA_TIME: u32 = 25;

    pub(crate) fn new(
        base: &'h OsuObject,
        last: &'h OsuObject,
        clock_rate: f64,
        idx: usize,
        dists: Distances,
    ) -> Self {
        let start_time = base.start_time / clock_rate;
        let delta_time = (base.start_time - last.start_time) / clock_rate;

        // * Capped to 25ms to prevent difficulty calculation breaking from simultaneous objects.
        let strain_time = delta_time.max(Self::MIN_DELTA_TIME as f64);

        Self {
            start_time,
            delta_time,
            base,
            strain_time,
            dists,
            idx,
        }
    }

    pub(crate) fn opacity_at(
        &self,
        time: f64,
        hidden: bool,
        time_preempt: f64,
        time_fade_in: f64,
    ) -> f64 {
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
            const FADE_OUT_DURATION_MULTIPLIER: f64 = 0.3;
            let fade_out_duration = time_preempt * FADE_OUT_DURATION_MULTIPLIER;

            (((time - fade_in_start_time) / fade_in_duration).clamp(0.0, 1.0))
                .min(1.0 - ((time - fade_out_start_time) / fade_out_duration).clamp(0.0, 1.0))
        } else {
            ((time - fade_in_start_time) / fade_in_duration).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Distances {
    pub(crate) lazy_jump_dist: f64,
    pub(crate) lazy_travel_dist: f32,
    pub(crate) min_jump_dist: f64,
    pub(crate) min_jump_time: f64,
    pub(crate) travel_dist: f64,
    pub(crate) travel_time: f64,
    pub(crate) angle: Option<f64>,
}

impl Distances {
    pub(crate) const NORMALISED_RADIUS: f32 = 50.0;

    const MAXIMUM_SLIDER_RADIUS: f32 = Self::NORMALISED_RADIUS * 2.4;
    const ASSUMED_SLIDER_RADIUS: f32 = Self::NORMALISED_RADIUS * 1.8;

    pub(crate) fn new(
        base: &mut OsuObject,
        last: &OsuObject,
        last_last: Option<&OsuObject>,
        clock_rate: f64,
        strain_time: f64,
        scaling_factor_: &ScalingFactor,
    ) -> Self {
        let mut this =
            if let Some(slider_values) = Self::compute_slider_cursor_pos(base, scaling_factor_) {
                let SliderValues {
                    lazy_travel_dist,
                    slider,
                } = slider_values;

                let repeat_count = slider.repeat_count();

                Self {
                    // * Bonus for repeat sliders until a better per nested object strain system can be achieved.
                    travel_dist: (lazy_travel_dist
                        * (1.0 + repeat_count as f64 / 2.5).powf(1.0 / 2.5) as f32)
                        as f64,
                    travel_time: (base.lazy_travel_time() / clock_rate)
                        .max(OsuDifficultyObject::MIN_DELTA_TIME as f64),
                    lazy_travel_dist,
                    ..Default::default()
                }
            } else {
                Self::default()
            };

        // * We don't need to calculate either angle or distance when
        // * one of the last->curr objects is a spinner
        if base.is_spinner() || last.is_spinner() {
            return this;
        }

        // * We will scale distances by this factor, so we can assume a uniform CircleSize among beatmaps.
        let scaling_factor = scaling_factor_.factor;

        let last_cursor_pos = Self::get_end_cursor_pos(last);

        this.lazy_jump_dist = (base.stacked_pos() * scaling_factor
            - last_cursor_pos * scaling_factor)
            .length() as f64;
        this.min_jump_time = strain_time;
        this.min_jump_dist = this.lazy_jump_dist;

        if let OsuObjectKind::Slider(slider) = &last.kind {
            let last_travel_time = (last.lazy_travel_time() / clock_rate)
                .max(OsuDifficultyObject::MIN_DELTA_TIME as f64);
            this.min_jump_time =
                (strain_time - last_travel_time).max(OsuDifficultyObject::MIN_DELTA_TIME as f64);

            // * There are two types of slider-to-object patterns to consider in order
            // * to better approximate the real movement a player will take to jump between the hitobjects.
            // *
            // * 1. The anti-flow pattern, where players cut the slider short in order to move to the next hitobject.
            // *
            // *     <======o==>  ← slider
            // *            |     ← most natural jump path
            // *            o     ← a follow-up hitcircle
            // *
            // * In this case the most natural jump path is approximated by LazyJumpDistance.
            // *
            // * 2. The flow pattern, where players follow through the slider to its
            // * visual extent into the next hitobject.
            // *
            // *     <======o==>---o
            // *                 ↑
            // *       most natural jump path
            // *
            // * In this case the most natural jump path is better approximated by a new distance
            // * called "tailJumpDistance" - the distance between the slider's tail and the next hitobject.
            // *
            // * Thus, the player is assumed to jump the minimum of these two distances in all cases.

            let stacked_tail_pos =
                slider.tail().map_or_else(|| last.pos(), |tail| tail.pos) + last.stack_offset;

            let tail_jump_dist = (stacked_tail_pos - base.stacked_pos()).length() * scaling_factor;

            this.min_jump_dist = (this.lazy_jump_dist
                - (Self::MAXIMUM_SLIDER_RADIUS - Self::ASSUMED_SLIDER_RADIUS) as f64)
                .min((tail_jump_dist - Self::MAXIMUM_SLIDER_RADIUS) as f64)
                .max(0.0);
        }

        if let Some(last_last) = last_last.filter(|obj| !obj.is_spinner()) {
            let last_last_cursor_pos = Self::get_end_cursor_pos(last_last);

            let v1 = last_last_cursor_pos - last.stacked_pos();
            let v2 = base.stacked_pos() - last_cursor_pos;

            let dot = v1.dot(v2) as f64;
            let det = (v1.x * v2.y - v1.y * v2.x) as f64;

            this.angle = Some(det.atan2(dot).abs());
        }

        this
    }

    pub(crate) fn compute_slider_cursor_pos<'h>(
        hit_object: &'h mut OsuObject,
        scaling_factor_: &ScalingFactor,
    ) -> Option<SliderValues<'h>> {
        let pos = hit_object.pos();

        let slider = if let OsuObjectKind::Slider(slider) = &mut hit_object.kind {
            slider
        } else {
            return None;
        };

        let mut curr_cursor_pos = pos + hit_object.stack_offset;
        let scaling_factor = Self::NORMALISED_RADIUS as f64 / scaling_factor_.radius as f64;

        let mut lazy_travel_dist: f32 = 0.0;

        for (curr_movement_obj, i) in slider.nested_iter().zip(1..) {
            let mut curr_movement =
                (curr_movement_obj.pos + hit_object.stack_offset) - curr_cursor_pos;
            let mut curr_movement_len = scaling_factor * curr_movement.length() as f64;

            // * Amount of movement required so that the cursor position needs to be updated.
            let mut required_movement = Self::ASSUMED_SLIDER_RADIUS as f64;

            if i == slider.nested_len() {
                // * The end of a slider has special aim rules due
                // * to the relaxed time constraint on position.
                // * There is both a lazy end position as well as the actual end slider position.
                // * We assume the player takes the simpler movement.
                // * For sliders that are circular, the lazy end position
                // * may actually be farther away than the sliders true end.
                // * This code is designed to prevent buffing situations
                // * where lazy end is actually a less efficient movement.
                let lazy_movement = slider.lazy_end_pos - curr_cursor_pos;

                if lazy_movement.length() < curr_movement.length() {
                    curr_movement = lazy_movement;
                }

                curr_movement_len = scaling_factor * curr_movement.length() as f64;
            } else if let NestedObjectKind::Repeat = curr_movement_obj.kind {
                // * For a slider repeat, assume a tighter movement threshold to better assess repeat sliders.
                required_movement = Self::NORMALISED_RADIUS as f64;
            }

            if curr_movement_len > required_movement {
                // * this finds the positional delta from the required radius and the current position, and updates the currCursorPosition accordingly, as well as rewarding distance.
                curr_cursor_pos += curr_movement
                    * ((curr_movement_len - required_movement) / curr_movement_len) as f32;
                curr_movement_len *= (curr_movement_len - required_movement) / curr_movement_len;
                lazy_travel_dist += curr_movement_len as f32;
            }
        }

        slider.lazy_end_pos = curr_cursor_pos;

        Some(SliderValues {
            lazy_travel_dist,
            slider,
        })
    }

    fn get_end_cursor_pos(hit_object: &OsuObject) -> Pos2 {
        hit_object.lazy_end_pos()
    }
}

pub(crate) struct SliderValues<'s> {
    lazy_travel_dist: f32,
    slider: &'s OsuSlider,
}
