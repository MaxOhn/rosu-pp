use crate::{
    any::difficulty::object::IDifficultyObject, catch::difficulty::object::CatchDifficultyObject,
    util::float_ext::FloatExt,
};

define_skill! {
    pub struct Movement: StrainDecaySkill => [CatchDifficultyObject][CatchDifficultyObject] {
        clock_rate: f64,
    }
}

impl Movement {
    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 0.2;

    const DECAY_WEIGHT: f64 = 0.94;

    const SECTION_LENGTH: f64 = 750.0;

    fn strain_value_of(
        &mut self,
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
    ) -> f64 {
        MovementEvaluator::evaluate_diff_of(curr, diff_objects, self.clock_rate)
    }
}

struct MovementEvaluator;

impl MovementEvaluator {
    const NORMALIZED_HITOBJECT_RADIUS: f32 = 41.0;
    const DIRECTION_CHANGE_BONUS: f64 = 21.0;

    fn evaluate_diff_of(
        curr: &CatchDifficultyObject,
        diff_objects: &[CatchDifficultyObject],
        clock_rate: f64,
    ) -> f64 {
        let catch_last_obj = curr.previous(0, diff_objects);
        let catch_last_last_obj = curr.previous(1, diff_objects);

        let weighted_strain_time = curr.strain_time + 13.0 + (3.0 / clock_rate);

        let mut dist_addition = f64::from(curr.dist_moved.abs()).powf(1.3) / 510.0;
        let sqrt_strain = weighted_strain_time.sqrt();

        let mut edge_dash_bonus: f64 = 0.0;

        let last_strain_time = catch_last_obj.map_or(0.0, |obj| obj.strain_time);

        if curr.dist_moved.abs() > 0.1 {
            let last_dist_moved = catch_last_obj.map_or(0.0, |obj| obj.dist_moved);

            if curr.idx >= 1
                && last_dist_moved.abs() > 0.1
                && curr.dist_moved.signum() != last_dist_moved.signum()
            {
                let bonus_factor = f64::from(curr.dist_moved.abs().min(50.0) / 50.0);
                let anti_flow_factor = f64::from(last_dist_moved.abs().min(70.0) / 70.0).max(0.38);

                dist_addition += Self::DIRECTION_CHANGE_BONUS / (last_strain_time + 16.0).sqrt()
                    * bonus_factor
                    * anti_flow_factor
                    * (1.0 - (weighted_strain_time / 1000.0).powf(3.0)).max(0.0);
            }

            // * Base bonus for every movement, giving some weight to streams.
            dist_addition += 12.5
                * f64::from(f32::abs(curr.dist_moved).min(Self::NORMALIZED_HITOBJECT_RADIUS * 2.0))
                / f64::from(Self::NORMALIZED_HITOBJECT_RADIUS * 6.0)
                / sqrt_strain;
        }

        // * Bonus for edge dashes.
        if curr.last_object.dist_to_hyper_dash <= 20.0 {
            if !curr.last_object.hyper_dash {
                edge_dash_bonus += 5.7;
            }

            // * Edge Dashes are easier at lower ms values
            dist_addition *= 1.0
                + edge_dash_bonus
                    * f64::from((20.0 - curr.last_object.dist_to_hyper_dash) / 20.0)
                    * ((curr.strain_time * clock_rate).min(265.0) / 265.0).powf(1.5);
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
            dist_addition = 0.0;
        }

        dist_addition / weighted_strain_time
    }
}
