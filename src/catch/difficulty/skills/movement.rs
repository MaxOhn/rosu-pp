use crate::{catch::difficulty::object::CatchDifficultyObject, util::float_ext::FloatExt};

define_skill! {
    pub struct Movement: StrainDecaySkill => [CatchDifficultyObject][CatchDifficultyObject] {
        half_catcher_width: f32,
        clock_rate: f64,
        last_player_pos: Option<f32> = None,
        last_dist_moved: f32 = 0.0,
        last_exact_dist_moved: f32 = 0.0,
        last_strain_time: f64 = 0.0,
        is_in_buzz_section: bool = false,
    }
}

impl Movement {
    const ABSOLUTE_PLAYER_POSITIONING_ERROR: f32 = 16.0;
    const NORMALIZED_HITOBJECT_RADIUS: f32 = 41.0;
    const DIRECTION_CHANGE_BONUS: f64 = 21.0;

    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 0.2;

    const DECAY_WEIGHT: f64 = 0.94;

    const SECTION_LENGTH: f64 = 750.0;

    fn strain_value_of(
        &mut self,
        curr: &CatchDifficultyObject,
        _: &[CatchDifficultyObject],
    ) -> f64 {
        let last_player_pos = self.last_player_pos.unwrap_or(curr.last_normalized_pos);

        let term = Self::NORMALIZED_HITOBJECT_RADIUS - Self::ABSOLUTE_PLAYER_POSITIONING_ERROR;
        let mut player_pos =
            last_player_pos.clamp(curr.normalized_pos - term, curr.normalized_pos + term);

        let dist_moved = player_pos - last_player_pos;

        // * For the exact position we consider that the catcher is in the correct position for both objects
        let exact_dist_moved = curr.normalized_pos - last_player_pos;

        let weighted_strain_time = curr.strain_time + 13.0 + (3.0 / self.clock_rate);

        let mut dist_addition = f64::from(dist_moved.abs()).powf(1.3) / 510.0;
        let sqrt_strain = weighted_strain_time.sqrt();

        let mut edge_dash_bonus: f64 = 0.0;

        if dist_moved.abs() > 0.1 {
            if self.last_dist_moved.abs() > 0.1
                && dist_moved.signum() != self.last_dist_moved.signum()
            {
                let bonus_factor = f64::from(dist_moved.abs().min(50.0) / 50.0);
                let anti_flow_factor =
                    f64::from(self.last_dist_moved.abs().min(70.0) / 70.0).max(0.38);

                dist_addition += Self::DIRECTION_CHANGE_BONUS
                    / (self.last_strain_time + 16.0).sqrt()
                    * bonus_factor
                    * anti_flow_factor
                    * (1.0 - (weighted_strain_time / 1000.0).powf(3.0)).max(0.0);
            }

            dist_addition += 12.5
                * f64::from(f32::abs(dist_moved).min(Self::NORMALIZED_HITOBJECT_RADIUS * 2.0))
                / f64::from(Self::NORMALIZED_HITOBJECT_RADIUS * 6.0)
                / sqrt_strain;
        }

        if curr.last_object.dist_to_hyper_dash <= 20.0 {
            if curr.last_object.hyper_dash {
                player_pos = curr.normalized_pos;
            } else {
                edge_dash_bonus += 5.7;
            }

            dist_addition *= 1.0
                + edge_dash_bonus
                    * f64::from((20.0 - curr.last_object.dist_to_hyper_dash) / 20.0)
                    * ((curr.strain_time * self.clock_rate).min(265.0) / 265.0).powf(1.5);
        }

        // * There is an edge case where horizontal back and forth sliders create "buzz" patterns which are repeated "movements" with a distance lower than
        // * the platter's width but high enough to be considered a movement due to the absolute_player_positioning_error and normalized_hitobject_radius offsets
        // * We are detecting this exact scenario. The first back and forth is counted but all subsequent ones are nullified.
        // * To achieve that, we need to store the exact distances (distance ignoring absolute_player_positioning_error and normalized_hitobject_radius)
        if exact_dist_moved.abs() <= self.half_catcher_width * 2.0
            && <f32 as FloatExt>::eq(exact_dist_moved, -self.last_exact_dist_moved)
            && <f64 as FloatExt>::eq(curr.strain_time, self.last_strain_time)
        {
            if self.is_in_buzz_section {
                dist_addition = 0.0;
            } else {
                self.is_in_buzz_section = true;
            }
        } else {
            self.is_in_buzz_section = false;
        }

        self.last_player_pos = Some(player_pos);
        self.last_dist_moved = dist_moved;
        self.last_strain_time = curr.strain_time;
        self.last_exact_dist_moved = exact_dist_moved;

        dist_addition / weighted_strain_time
    }
}
