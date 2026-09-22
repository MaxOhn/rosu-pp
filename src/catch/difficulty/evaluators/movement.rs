use crate::{
    any::difficulty::object::IDifficultyObject,
    catch::difficulty::object::CatchDifficultyObject,
    util::{difficulty as diff_utils, float_ext::FloatExt},
};

pub struct MovementEvaluator;

impl MovementEvaluator {
    const NORMALIZED_HITOBJECT_RADIUS: f32 = 41.0;
    const DIRECTION_CHANGE_BONUS: f64 = 21.0;

    pub fn evaluate_diff_of(
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
        clock_rate: f64,
    ) -> f64 {
        let catch_last_obj = curr.previous(0, diff_objects);
        let catch_last_last_obj = curr.previous(1, diff_objects);

        let weighted_strain_time = curr.strain_time + 13.0 + (3.0 / clock_rate);

        let mut distance_addition = f64::powf(f64::from(f32::abs(curr.dist_moved)), 1.3) / 510.0;
        let sqrt_strain = f64::sqrt(weighted_strain_time);

        let mut edge_dash_bonus: f64 = 0.0;

        let last_strain_time = catch_last_obj.map_or(0.0, |obj| obj.strain_time);

        if f32::abs(curr.dist_moved) > 0.1 {
            let last_dist_moved = catch_last_obj.map_or(0.0, |obj| obj.dist_moved);

            if curr.idx >= 1
                && last_dist_moved.abs() > 0.1
                && FloatExt::not_eq(f32::signum(curr.dist_moved), f32::signum(last_dist_moved))
            {
                let bonus_factor = f64::from(f32::min(50.0, f32::abs(curr.dist_moved)) / 50.0);
                let anti_flow_factor = f64::max(
                    f64::from(f32::min(70.0, f32::abs(last_dist_moved)) / 70.0),
                    0.38,
                );

                distance_addition += Self::DIRECTION_CHANGE_BONUS
                    / f64::sqrt(last_strain_time + 16.0)
                    * bonus_factor
                    * anti_flow_factor
                    * f64::max(1.0 - diff_utils::pow(weighted_strain_time / 1000.0, 3), 0.0);
            }

            // * Base bonus for every movement, giving some weight to streams.
            distance_addition +=
                12.5 * f64::from(f32::min(
                    f32::abs(curr.dist_moved),
                    Self::NORMALIZED_HITOBJECT_RADIUS * 2.0,
                )) / f64::from(Self::NORMALIZED_HITOBJECT_RADIUS * 6.0)
                    / sqrt_strain;
        }

        // * Linear spacing nerf.
        let mut linear_spacing_count: f64 = 0.0;

        for i in 0..10 {
            let Some(catch_prev_obj) = curr.previous(i, diff_objects) else {
                break;
            };

            // * Only same direction movements matter as they do not take any additional inputs.
            if FloatExt::not_eq(
                f32::signum(curr.dist_moved),
                f32::signum(catch_prev_obj.dist_moved),
            ) || curr.dist_moved == 0.0
                || catch_prev_obj.dist_moved == 0.0
            {
                break;
            }

            let current_spacing = f64::from(curr.dist_moved) / curr.strain_time;
            let prev_spacing = f64::from(catch_prev_obj.dist_moved) / catch_prev_obj.strain_time;

            let relative_difference = f64::abs(current_spacing / prev_spacing - 1.0);

            if relative_difference > 0.05 {
                break;
            }

            linear_spacing_count += 1.0;
        }

        distance_addition *= f64::powf(0.7, linear_spacing_count);

        // * Bonus for edge dashes.
        if curr.last_object.dist_to_hyper_dash <= 20.0 {
            if !curr.last_object.hyper_dash {
                edge_dash_bonus += 5.7;
            }

            // * Edge Dashes are easier at lower ms values
            distance_addition *= 1.0
                + edge_dash_bonus
                    * f64::from((20.0 - curr.last_object.dist_to_hyper_dash) / 20.0)
                    * f64::powf(f64::min(curr.strain_time * clock_rate, 265.0) / 265.0, 1.5);
        }

        let last_exact_dist_moved = catch_last_obj.map_or(0.0, |obj| obj.exact_dist_moved);
        let last_last_exact_dist_moved =
            catch_last_last_obj.map_or(0.0, |obj| obj.exact_dist_moved);

        let last_last_strain_time = catch_last_last_obj.map_or(0.0, |obj| obj.strain_time);

        // * There is an edge case where horizontal back and forth sliders create "buzz" patterns which are repeated "movements" with a distance lower than
        // * the platter's width but high enough to be considered a movement due to the absolute_player_positioning_error and NORMALIZED_HALF_CATCHER_WIDTH offsets
        // * We are detecting this exact scenario. The first back and forth is counted but all subsequent ones are nullified.
        // * To achieve that, we need to store the exact distances (distance ignoring absolute_player_positioning_error and NORMALIZED_HALF_CATCHER_WIDTH)
        if curr.idx >= 2
            && curr.exact_dist_moved.abs()
                <= CatchDifficultyObject::NORMALIZED_HALF_CATCHER_WIDTH * 2.0
            && <f32 as FloatExt>::eq(curr.exact_dist_moved, -last_exact_dist_moved)
            && <f32 as FloatExt>::eq(last_exact_dist_moved, -last_last_exact_dist_moved)
            && <f64 as FloatExt>::eq(curr.strain_time, last_strain_time)
            && <f64 as FloatExt>::eq(last_strain_time, last_last_strain_time)
        {
            distance_addition = 0.0;
        }

        distance_addition / weighted_strain_time
    }
}
