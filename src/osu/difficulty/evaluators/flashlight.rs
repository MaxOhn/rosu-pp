use std::cmp;

use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::{difficulty::object::OsuDifficultyObject, object::OsuObjectKind},
};

pub struct FlashlightEvaluator {
    scaling_factor: f64,
    time_preempt: f64,
    time_fade_in: f64,
}

impl FlashlightEvaluator {
    const MAX_OPACITY_BONUS: f64 = 0.4;
    const HIDDEN_BONUS: f64 = 0.2;

    const MIN_VELOCITY: f64 = 0.5;
    const SLIDER_MULTIPLIER: f64 = 1.3;

    const MIN_ANGLE_MULTIPLIER: f64 = 0.2;

    pub const fn new(scaling_factor: f64, time_preempt: f64, time_fade_in: f64) -> Self {
        Self {
            scaling_factor,
            time_preempt,
            time_fade_in,
        }
    }

    pub fn evaluate_diff_of<'a>(
        &self,
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        hidden: bool,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let osu_curr = curr;
        let osu_hit_obj = curr.base;

        let mut small_dist_nerf = 1.0;
        let mut cumulative_strain_time = 0.0;

        let mut result = 0.0;

        let mut last_obj = osu_curr;

        let mut angle_repeat_count = 0.0;

        // * This is iterating backwards in time from the current object.
        for i in 0..cmp::min(curr.idx, 10) {
            let Some(curr_obj) = curr.previous(i, diff_objects) else {
                break;
            };

            cumulative_strain_time += last_obj.adjusted_delta_time;

            let curr_hit_obj = curr_obj.base;

            if !curr_obj.base.is_spinner() {
                let jump_dist = f64::from(
                    (osu_hit_obj.stacked_pos() - curr_hit_obj.stacked_end_pos()).length(),
                );

                // * We want to nerf objects that can be easily seen within the Flashlight circle radius.
                if i == 0 {
                    small_dist_nerf = (jump_dist / 75.0).min(1.0);
                }

                // * We also want to nerf stacks so that only the first object of the stack is accounted for.
                let stack_nerf = ((curr_obj.lazy_jump_dist / self.scaling_factor) / 25.0).min(1.0);

                // * Bonus based on how visible the object is.
                let opacity_bonus = 1.0
                    + Self::MAX_OPACITY_BONUS
                        * (1.0
                            - osu_curr.opacity_at(
                                curr_hit_obj.start_time,
                                hidden,
                                self.time_preempt,
                                self.time_fade_in,
                            ));

                result += stack_nerf * opacity_bonus * self.scaling_factor * jump_dist
                    / cumulative_strain_time;

                if let Some((curr_obj_angle, osu_curr_angle)) = curr_obj.angle.zip(osu_curr.angle) {
                    // * Objects further back in time should count less for the nerf.
                    if (curr_obj_angle - osu_curr_angle).abs() < 0.02 {
                        angle_repeat_count += (1.0 - 0.1 * i as f64).max(0.0);
                    }
                }
            }

            last_obj = curr_obj;
        }

        result = (small_dist_nerf * result).powf(2.0);

        // * Additional bonus for Hidden due to there being no approach circles.
        if hidden {
            result *= 1.0 + Self::HIDDEN_BONUS;
        }

        // * Nerf patterns with repeated angles.
        result *= Self::MIN_ANGLE_MULTIPLIER
            + (1.0 - Self::MIN_ANGLE_MULTIPLIER) / (angle_repeat_count + 1.0);

        let mut slider_bonus = 0.0;

        if let OsuObjectKind::Slider(slider) = &osu_curr.base.kind {
            // * Invert the scaling factor to determine the true travel distance independent of circle size.
            let pixel_travel_dist = osu_curr.lazy_travel_dist / self.scaling_factor;

            // * Reward sliders based on velocity.
            slider_bonus = ((pixel_travel_dist / osu_curr.travel_time - Self::MIN_VELOCITY)
                .max(0.0))
            .powf(0.5);

            // * Longer sliders require more memorisation.
            slider_bonus *= pixel_travel_dist;

            // * Nerf sliders with repeats, as less memorisation is required.
            let repeat_count = slider.repeat_count();

            if repeat_count > 0 {
                slider_bonus /= (repeat_count + 1) as f64;
            }
        }

        result += slider_bonus * Self::SLIDER_MULTIPLIER;

        result
    }
}
