use rosu_map::util::Pos;

use crate::{
    any::difficulty::object::IDifficultyObject, osu::difficulty::{
        evaluators::aim::snap_aim::SnapAimEvaluator, 
        object::OsuDifficultyObject
    }, 
    util::{
        difficulty::{smootherstep, smoothstep}, 
        float_ext::FloatExt
    }
};

pub struct FlowAimEvaluator;

impl FlowAimEvaluator {
    const VELOCITY_CHANGE_MULTIPLIER: f64 = 0.52;

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

        let curr_dist = if with_slider_travel_dist { osu_curr_obj.lazy_jump_dist } else { osu_curr_obj.jump_dist };
        let prev_dist = if with_slider_travel_dist { osu_last_obj.lazy_jump_dist } else { osu_last_obj.jump_dist };

        let mut curr_vel = curr_dist / osu_curr_obj.adjusted_delta_time;

        if osu_last_obj.base.is_slider() && with_slider_travel_dist {
            // * If the last object is a slider, then we extend the travel velocity through the slider into the current object.
            let slider_dist = osu_last_obj.lazy_travel_dist + osu_curr_obj.lazy_jump_dist;
            curr_vel = curr_vel.max(slider_dist / osu_curr_obj.adjusted_delta_time);
        }

        let prev_vel = prev_dist / osu_last_obj.adjusted_delta_time;

        let mut flow_diff = curr_vel;

        // * Apply high circle size bonus to the base velocity.
        // * We use reduced CS bonus here because the bonus was made for an evaluator with a different d/t scaling.
        flow_diff *= osu_curr_obj.small_circle_bonus.sqrt();

        // * Rhythm changes are harder to flow.
        flow_diff *= 1.0 + f64::min(
            0.25,
            f64::powf(
                (osu_curr_obj.adjusted_delta_time.max(osu_last_obj.adjusted_delta_time) - osu_curr_obj.adjusted_delta_time.min(osu_last_obj.adjusted_delta_time)) / 50.0, 
                4.0
            )
        );

        if let (Some(curr_angle), Some(last_angle)) = (osu_curr_obj.angle, osu_last_obj.angle) {
            let angle_diff = (curr_angle - last_angle).abs();
            let angle_diff_adjusted = (angle_diff / 2.0).sin() * 180.0;
            let angular_vel = angle_diff_adjusted / (osu_curr_obj.adjusted_delta_time * 0.1);

            // * Low angular velocity flow (angles are consistent) is easier to follow than erratic flow.
            flow_diff *= 0.8 + (angular_vel / 270.0).sqrt();
        }

        // * If all three notes are overlapping - don't reward bonuses as you don't have to do additional movement.
        let mut overlapped_notes_weight = 1.0;

        // NOTE: Source does not null check osuLastLastObj
        // instead current.Index > 2 is checked.
        if let Some(osu_last_last_obj) = curr.previous(1, diff_objects) {
            overlapped_notes_weight = 1.0
                - Self::calc_overlap_factor(osu_curr_obj, osu_last_obj)
                * Self::calc_overlap_factor(osu_curr_obj, osu_last_last_obj)
                * Self::calc_overlap_factor(osu_last_obj, osu_last_last_obj);
        }

        if let Some(curr_angle) = osu_curr_obj.angle {
            // * Acute angles are also hard to flow.
            flow_diff += curr_vel
                * SnapAimEvaluator::calc_angle_acuteness(curr_angle)
                * overlapped_notes_weight;
        }

        if prev_vel.max(curr_vel).not_eq(0.0) {
            if with_slider_travel_dist {
                curr_vel = curr_dist / osu_curr_obj.adjusted_delta_time;
            }

            // * Scale with ratio of difference compared to 0.5 * max dist.
            let dist_ratio = smoothstep(
                (prev_vel - curr_vel).abs() / prev_vel.max(curr_vel),
                0.0,
                1.0
            );

            // * Reward for % distance up to 125 / strainTime for overlaps where velocity is still changing.
            let overlap_vel_buff = f64::min(
                f64::from(OsuDifficultyObject::NORMALIZED_DIAMETER) * 1.25 / osu_curr_obj.adjusted_delta_time.min(osu_last_obj.adjusted_delta_time),
                (prev_vel - curr_vel).abs()
            );

            flow_diff += overlap_vel_buff
                * dist_ratio
                * overlapped_notes_weight
                * Self::VELOCITY_CHANGE_MULTIPLIER;
        }

        if osu_curr_obj.base.is_slider() && with_slider_travel_dist {
            // * Include slider velocity to make velocity more consistent with snap.
            flow_diff += osu_curr_obj.travel_dist / osu_curr_obj.travel_time;
        }

        // * Final velocity is being raised to a power because flow difficulty scales harder with 
        // * both high distance and time, and we want to account for that.
        flow_diff = flow_diff.powf(1.45);

        // * Reduce difficulty for low spacing since spacing below radius is always to be flowed
        flow_diff * smootherstep(curr_dist, 0.0, f64::from(OsuDifficultyObject::NORMALIZED_RADIUS))
    }

    fn calc_overlap_factor<'a>(
        first: &'a OsuDifficultyObject<'a>,
        second: &'a OsuDifficultyObject<'a>
    ) -> f64 {
        let dist = Pos::distance(&first.base.stacked_pos(), second.base.stacked_pos());

        f64::clamp(
            1.0 - (f64::from(dist) - first.radius).max(0.0).powf(2.0),
            0.0,
            1.0
        )
    }
}