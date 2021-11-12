use crate::{
    osu::osu_object::{NestedObjectKind, OsuObjectKind},
    parse::Pos2,
};

use super::{OsuObject, ScalingFactor, NORMALIZED_RADIUS};

const MIN_DELTA_TIME: f64 = 25.0;
const MAXIMUM_SLIDER_RADIUS: f32 = NORMALIZED_RADIUS * 2.4;
const ASSUMED_SLIDER_RADIUS: f32 = NORMALIZED_RADIUS * 1.65;

pub(crate) struct DifficultyObject<'h> {
    pub(crate) base: &'h OsuObject,

    pub(crate) delta: f64,
    pub(crate) strain_time: f64,

    pub(crate) angle: Option<f64>,
    pub(crate) jump_dist: f64,

    pub(crate) movement_dist: f64,
    pub(crate) movement_time: f64,

    pub(crate) travel_dist: f64,
    pub(crate) travel_time: f64,
}

impl<'h> DifficultyObject<'h> {
    pub(super) fn new(
        base: &'h OsuObject,
        prev: &mut OsuObject,
        prev_prev: Option<&OsuObject>,
        scaling_factor: &ScalingFactor,
        clock_rate: f64,
    ) -> Self {
        let delta = base.time - prev.time;

        // * Capped to 25ms to prevent difficulty calculation breaking from simultaneous objects
        let strain_time = delta.max(MIN_DELTA_TIME);

        // * We don't need to calculate either angle or distances
        // * when one of the last->curr objects is a spinner
        let (travel_dist, travel_time, movement_dist, movement_time, jump_dist, angle) =
            if base.is_spinner() || prev.is_spinner() {
                (0.0, 0.0, 0.0, 0.0, 0.0, None)
            } else {
                let prev_stack_offset = scaling_factor.stack_offset(prev.stack_height);

                // Important to call `Self::compute_slider_cursor_pos` before using `prev.lazy_end_pos`
                // because the lazy end position is being calculated in that function
                let (travel_dist, travel_time) = Self::compute_slider_cursor_pos(
                    prev,
                    prev_stack_offset,
                    scaling_factor.raw(),
                    clock_rate,
                );

                let prev_cursor_pos = prev.lazy_end_pos(prev_stack_offset);

                let jump_dist =
                    ((base.pos - prev_cursor_pos) * scaling_factor.adjusted()).length() as f64;

                let angle =
                    prev_prev
                        .filter(|prev_prev| !prev_prev.is_spinner())
                        .map(|prev_prev| {
                            let prev_prev_cursor_pos = prev_prev
                                .lazy_end_pos(scaling_factor.stack_offset(prev_prev.stack_height));

                            let v1 = prev_prev_cursor_pos - prev.pos;
                            let v2 = base.pos - prev_cursor_pos;

                            let dot = (v1.dot(v2)) as f64;
                            let det = (v1.x * v2.y - v1.y * v2.x) as f64;

                            det.atan2(dot).abs()
                        });

                let (movement_dist, movement_time) = Self::compute_movement_values(
                    prev,
                    base.pos,
                    jump_dist,
                    strain_time,
                    travel_time,
                    scaling_factor.adjusted(),
                );

                (
                    travel_dist,
                    travel_time,
                    movement_dist,
                    movement_time,
                    jump_dist,
                    angle,
                )
            };

        Self {
            base,
            delta,
            strain_time,
            jump_dist,
            angle,
            movement_dist,
            movement_time,
            travel_dist,
            travel_time,
        }
    }

    fn compute_slider_cursor_pos(
        prev: &mut OsuObject,
        stack_offset: Pos2,
        scaling_factor: f64,
        clock_rate: f64,
    ) -> (f64, f64) {
        match &mut prev.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => (0.0, 0.0),
            OsuObjectKind::Slider {
                lazy_end_pos,
                nested_objects,
                ..
            } => {
                let mut travel_dist = 0.0;
                let pos = prev.pos - stack_offset; // stack offset is ignored everywhere
                let mut curr_cursor_pos = pos;

                let last_idx = nested_objects.len() - 1;

                for (i, nested) in nested_objects.iter_mut().enumerate() {
                    let mut curr_movement = nested.pos - curr_cursor_pos;
                    let mut curr_movement_len = scaling_factor * curr_movement.length() as f64;

                    // * Amount of movement required so that the cursor position needs to be updated.
                    let mut required_movement = ASSUMED_SLIDER_RADIUS as f64;

                    if i == last_idx {
                        // * The end of a slider has special aim rules due
                        // * to the relaxed time constraint on position.
                        // * There is both a lazy end position as well as the actual end slider position.
                        // * We assume the player takes the simpler movement.
                        // * For sliders that are circular, the lazy end position
                        // * may actually be farther away than the sliders true end.
                        // * This code is designed to prevent buffing situations
                        // * where lazy end is actually a less efficient movement.
                        let lazy_movement = *lazy_end_pos - curr_cursor_pos;

                        if lazy_movement.length() < curr_movement.length() {
                            curr_movement = lazy_movement;
                        }

                        curr_movement_len = scaling_factor * curr_movement.length() as f64;
                    } else if let NestedObjectKind::Repeat = nested.kind {
                        // * For a slider repeat, assume a tighter movement
                        // * threshold to better assess repeat sliders.
                        required_movement = NORMALIZED_RADIUS as f64;
                    }

                    if curr_movement_len > required_movement {
                        // * this finds the positional delta from the required
                        // * radius and the current position, and updates the
                        // * currCursorPosition accordingly, as well as rewarding distance.
                        curr_cursor_pos = curr_cursor_pos
                            + curr_movement
                                * ((curr_movement_len - required_movement) / curr_movement_len)
                                    as f32;

                        curr_movement_len *=
                            (curr_movement_len - required_movement) / curr_movement_len;

                        travel_dist += curr_movement_len;
                    }

                    if i == last_idx {
                        *lazy_end_pos = curr_cursor_pos;
                    }
                }

                let repeats = nested_objects
                    .iter()
                    .filter(|nested| matches!(nested.kind, NestedObjectKind::Repeat))
                    .count();

                // * Bonus for repeat sliders until a better per
                // * nested object strain system can be achieved.
                travel_dist *= (1.0 + repeats as f64 / 2.5).powf(1.0 / 2.5);
                let prev_time = prev.time;

                let lazy_travel_time = nested_objects
                    .last()
                    .map_or(0.0, |nested| nested.time / clock_rate - prev_time);

                let travel_time = MIN_DELTA_TIME.max(lazy_travel_time);

                (travel_dist, travel_time)
            }
        }
    }

    fn compute_movement_values(
        prev: &OsuObject,
        base_pos: Pos2,
        jump_dist: f64,
        strain_time: f64,
        travel_time: f64,
        scaling_factor: f32,
    ) -> (f64, f64) {
        match &prev.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => (jump_dist, strain_time),
            OsuObjectKind::Slider { end_pos, .. } => {
                let movement_time = MIN_DELTA_TIME.max(strain_time - travel_time);

                // * Jump distance from the slider tail to the next object,
                // * as opposed to the lazy position of JumpDistance.
                let tail_jump_dist = (*end_pos - base_pos).length() * scaling_factor;

                // * For hitobjects which continue in the direction of the slider,
                // * the player will normally follow through the slider,
                // * such that they're not jumping from the lazy position but
                // * rather from very close to (or the end of) the slider.
                // * In such cases, a leniency is applied by also considering the
                // * jump distance from the tail of the slider,
                // * and taking the minimum jump distance.
                // * Additional distance is removed based on position of jump
                // * relative to slider follow circle radius.
                // * JumpDistance is the leniency distance beyond the assumed_slider_radius.
                // * tailJumpDistance is maximum_slider_radius since
                // * the full distance of radial leniency is still possible.
                let movement_dist = (jump_dist
                    - (MAXIMUM_SLIDER_RADIUS - ASSUMED_SLIDER_RADIUS) as f64)
                    .min((tail_jump_dist - MAXIMUM_SLIDER_RADIUS) as f64)
                    .max(0.0);

                (movement_dist, movement_time)
            }
        }
    }
}
