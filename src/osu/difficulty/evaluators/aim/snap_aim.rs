use crate::{
    any::difficulty::object::IDifficultyObject, 
    osu::difficulty::object::OsuDifficultyObject, 
    util::{
        difficulty::{milliseconds_to_bpm, reverse_lerp, smootherstep, smoothstep}, 
        float_ext::FloatExt
    }
};

pub struct SnapAimEvaluator;

impl SnapAimEvaluator {
    const WIDE_ANGLE_MULTIPLIER: f64 = 9.67;
    const ACUTE_ANGLE_MULTIPLIER: f64 = 2.41;
    const SLIDER_MULTIPLIER: f64 = 1.5;
    const VELOCITY_CHANGE_MULTIPLIER: f64 = 0.9;
    const WIGGLE_MULTIPLIER: f64 = 1.02;

    const WIDE_ANGLE_TIME_SCALE: f64 = 1.45;

    const REPETITION_NOTE_LIMIT: usize = 6;
    const REPETITION_MAX_NERF: f64 = 0.15;
    const REPETITION_MAX_VECTOR_INFLUENCE: f64 = 0.5;

    #[expect(clippy::too_many_lines, reason = "staying in-sync with lazer")]
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        with_slider_travel_dist: bool,
    ) -> f64 {
        let osu_curr_obj = curr;

        let Some(osu_last_obj) = curr
            .previous(0, diff_objects)
            .filter(|last| !(curr.base.is_spinner() || last.base.is_spinner()))
        else {
            return 0.0;
        };

        #[expect(clippy::items_after_statements, reason = "staying in-sync with lazer")]
        const RADIUS: i32 = OsuDifficultyObject::NORMALIZED_RADIUS;
        #[expect(clippy::items_after_statements, reason = "staying in-sync with lazer")]
        const DIAMETER: i32 = OsuDifficultyObject::NORMALIZED_DIAMETER;

        // * Calculate the velocity to the current hitobject, which starts
        // * with a base distance / time assuming the last object is a hitcircle.
        let curr_dist = if with_slider_travel_dist { osu_curr_obj.lazy_jump_dist } else { osu_curr_obj.jump_dist };
        let mut curr_vel = osu_curr_obj.lazy_jump_dist / osu_curr_obj.adjusted_delta_time;

        // * But if the last object is a slider, then we extend the travel
        // * velocity through the slider into the current object.
        if osu_last_obj.base.is_slider() && with_slider_travel_dist {
            let slider_distance = osu_last_obj.lazy_travel_dist + osu_curr_obj.lazy_jump_dist;
            curr_vel = curr_vel.max(slider_distance / osu_curr_obj.adjusted_delta_time);
        }

        let prev_dist = if with_slider_travel_dist { osu_last_obj.lazy_jump_dist } else { osu_last_obj.jump_dist };
        let prev_vel = prev_dist / osu_last_obj.adjusted_delta_time;

        // * Start difficulty with regular velocity.
        let mut snap_diff = curr_vel;

        // * Penalize angle repetition.
        snap_diff *= Self::vector_angle_repetition(osu_curr_obj, osu_last_obj, diff_objects);

        if let (Some(curr_angle), Some(last_angle)) = (osu_curr_obj.angle, osu_last_obj.angle) {
            // * Rewarding angles, take the smaller velocity as base.
            let vel_influence = curr_vel.min(prev_vel);
            let mut acute_angle_bonus = 0.0;

            // * If rhythms are the same.
            if osu_curr_obj.adjusted_delta_time.max(osu_last_obj.adjusted_delta_time) <
                1.25 * osu_curr_obj.adjusted_delta_time.min(osu_last_obj.adjusted_delta_time) {
                acute_angle_bonus = Self::calc_angle_acuteness(curr_angle);
                
                // * Penalize angle repetition. It is important to do it _before_
                // * multiplying by anything because we compare raw acuteness here.
                acute_angle_bonus *= 0.08 
                    + 0.92 
                    * (1.0 - f64::min(
                        acute_angle_bonus,
                        f64::powf(Self::calc_angle_acuteness(last_angle), 3.0)
                    ));

                // * Apply acute angle bonus for BPM above 300 1/2 and distance more than one diameter.
                acute_angle_bonus *= vel_influence 
                    * smootherstep(
                        milliseconds_to_bpm(osu_curr_obj.adjusted_delta_time, Some(2)),
                        300.0, 
                        400.0
                    )
                    * smootherstep(curr_dist, 0.0, f64::from(DIAMETER * 2));
            }

            let mut wide_angle_bonus = Self::calc_angle_wideness(curr_angle);

            // * Penalize angle repetition. It is important to do it _before_
            // * multiplying by anything because we compare raw wideness here.
            wide_angle_bonus *= 0.25 
                + 0.75 
                * (1.0 - f64::min(
                    wide_angle_bonus,
                    f64::powf(Self::calc_angle_wideness(last_angle), 3.0)
                ));
            
            // * Rescaling velocity for the wide angle bonus
            let mut wide_angle_curr_vel = curr_dist / 
                osu_curr_obj.adjusted_delta_time.powf(Self::WIDE_ANGLE_TIME_SCALE);
            let wide_angle_prev_vel = prev_dist /
                osu_last_obj.adjusted_delta_time.powf(Self::WIDE_ANGLE_TIME_SCALE);

            if osu_last_obj.base.is_slider() && with_slider_travel_dist {
                let slider_dist = osu_last_obj.lazy_travel_dist + osu_curr_obj.lazy_jump_dist;
                wide_angle_curr_vel = f64::max(
                    wide_angle_curr_vel,
                    slider_dist / f64::powf(osu_curr_obj.adjusted_delta_time, Self::WIDE_ANGLE_TIME_SCALE)
                );
            }

            wide_angle_bonus *= wide_angle_curr_vel.min(wide_angle_prev_vel);

            if let Some(osu_last_2_obj) = curr.previous(2, diff_objects) {
                // * If objects just go back and forth through a middle point - don't give as much wide bonus.
                // * Use Previous(2) and Previous(0) because angles calculation is done prevprev-prev-curr,
                // * so any object's angle's center point is always the previous object.
                let dist = (osu_last_2_obj.base.stacked_pos() - osu_last_obj.base.stacked_pos()).length();

                if dist < 1.0 {
                    wide_angle_bonus *= 1.0 - 0.55 * (1.0 - f64::from(dist));
                }
            }

            // * Add in acute angle bonus or wide angle bonus, whichever is larger.
            snap_diff += f64::max(
                acute_angle_bonus * Self::ACUTE_ANGLE_MULTIPLIER,
                wide_angle_bonus * Self::WIDE_ANGLE_MULTIPLIER
            );

            // * Apply wiggle bonus for jumps that are [radius, 3*diameter] in distance, with < 110 angle
            // * https://www.desmos.com/calculator/dp0v0nvowc
            let wiggle_bonus = vel_influence
                * smootherstep(curr_dist, f64::from(RADIUS), f64::from(DIAMETER))
                * f64::powf(
                    reverse_lerp(curr_dist, f64::from(DIAMETER * 3), f64::from(DIAMETER)),
                    1.8
                )
                * smootherstep(curr_angle, f64::to_radians(110.0), f64::to_radians(60.0))
                * smootherstep(prev_dist, f64::from(RADIUS), f64::from(DIAMETER))
                * f64::powf(
                    reverse_lerp(prev_dist, f64::from(DIAMETER * 3), f64::from(DIAMETER)),
                    1.8
                )
                * smootherstep(last_angle, f64::to_radians(110.0), f64::to_radians(60.0));
            
            snap_diff += wiggle_bonus * Self::WIGGLE_MULTIPLIER;
        }

        if prev_vel.max(curr_vel).not_eq(0.0) {
            if with_slider_travel_dist {
                // * We want to use just the object jump without slider velocity when awarding differences
                curr_vel = curr_dist / osu_curr_obj.adjusted_delta_time;
            }

            // * Scale with ratio of difference compared to 0.5 * max dist.
            let dist_ratio = smoothstep(
                (prev_vel - curr_vel).abs() / prev_vel.max(curr_vel),
                0.0,
                1.0
            );

            // * Reward for % distance up to 125 / strainTime for overlaps where velocity is still changing.
            let overlap_vel_buff = (f64::from(DIAMETER) * 1.25 / osu_curr_obj.adjusted_delta_time
                .min(osu_last_obj.adjusted_delta_time)
            ).min((prev_vel - curr_vel).abs());

            let mut vel_change_bonus = overlap_vel_buff * dist_ratio;

            // * Penalize for rhythm changes.
            vel_change_bonus *= f64::powf(
                osu_curr_obj.adjusted_delta_time.min(osu_last_obj.adjusted_delta_time)
                    / osu_curr_obj.adjusted_delta_time.max(osu_last_obj.adjusted_delta_time),
                2.0
            );

            snap_diff += vel_change_bonus * Self::VELOCITY_CHANGE_MULTIPLIER;
        }

        // * Reward sliders based on velocity.
        if osu_curr_obj.base.is_slider() && with_slider_travel_dist {
            let slider_bonus = osu_curr_obj.travel_dist / osu_curr_obj.travel_time;
            snap_diff += if slider_bonus.lt(&1.0)
                { slider_bonus }
                else { slider_bonus.powf(0.75) * Self::SLIDER_MULTIPLIER };
        }

        // * Apply high circle size bonus
        snap_diff *= osu_curr_obj.small_circle_bonus;

        snap_diff *= Self::high_bpm_bonus(osu_curr_obj.adjusted_delta_time);

        snap_diff
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - f64::powf(0.03, (ms / 1000.0).powf(0.65)))
    }

    fn vector_angle_repetition<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        prev: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>]
    ) -> f64 {
        let (Some(curr_angle), Some(prev_angle)) = (curr.angle, prev.angle) else {
            return 1.0;
        };

        let mut const_angle_count: f64 = 0.0;
        for i in 0..Self::REPETITION_NOTE_LIMIT {
            let Some(prev_obj) = curr.previous(i, diff_objects) else {
                break
            };

            // * Only consider vectors in the same jump section,
            // * stopping to change rhythm ruins momentum
            if curr.adjusted_delta_time.max(prev_obj.adjusted_delta_time) >
                1.1 * curr.adjusted_delta_time.min(prev_obj.adjusted_delta_time) {
                break
            }

            if let (Some(curr_vec_angle), Some(prev_vec_angle)) = (curr.normalized_vector_angle, prev.normalized_vector_angle) {
                let angle_diff = curr_vec_angle - prev_vec_angle;
                // * Refer to this desmos for tuning, constants need to be precise
                // * so that values stay within the range of 0 and 1.
                // * https://www.desmos.com/calculator/a8jesv5sv2
                const_angle_count += (8.0 * f64::to_radians(11.25).min(angle_diff)).cos();
            }
        }

        let vec_repetition = (0.5 / const_angle_count).min(1.0).powf(2.0);
        let stack_factor = smootherstep(curr.lazy_jump_dist, 0.0, f64::from(OsuDifficultyObject::NORMALIZED_DIAMETER));
        let angle_diff_adjusted = (2.0 * f64::to_radians(45.0).min((curr_angle - prev_angle).abs() * stack_factor)).cos();
        let base_nerf = 1.0 - Self::REPETITION_MAX_NERF * Self::calc_angle_acuteness(prev_angle) * angle_diff_adjusted;

        (base_nerf + (1.0 - base_nerf) * vec_repetition * Self::REPETITION_MAX_VECTOR_INFLUENCE * stack_factor).powf(2.0)
    }

    const fn calc_angle_wideness(angle: f64) -> f64 {
        smoothstep(angle, f64::to_radians(140.0), f64::to_radians(40.0))
    }

    pub const fn calc_angle_acuteness(angle: f64) -> f64 {
        smoothstep(angle, f64::to_radians(140.0), f64::to_radians(40.0))
    }
}