use core::f64;

use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::{
        difficulty::{norm, reverse_lerp, smootherstep},
        float_ext::FloatExt,
    },
};

pub struct ReadingEvaluator {
    time_preempt: f64,
    time_fade_in: f64,
}

impl ReadingEvaluator {
    const READING_WINDOW_SIZE: f64 = 3000.0; // * 3 seconds
    const DISTANCE_INFLUENCE_THRESHOLD: f64 =
        (OsuDifficultyObject::NORMALIZED_DIAMETER as f64) * 1.5;

    const DENSITY_MULTIPLIER: f64 = 2.4;
    const DESNITY_DIFFICULTY_BASE: f64 = 2.5;

    const PREEMPT_BALANCING_FACTOR: f64 = 14_0000.0;
    const PREEMPT_STARTING_POINT: f64 = 500.0; // * AR 9.66 in milliseconds

    const HIDDEN_MULTIPLIER: f64 = 0.28;

    const MINIMUM_ANGLE_RELEVANCY_TIME: f64 = 2000.0; // * 2 seconds
    const MAXIMUM_ANGLE_RELEVANCY_TIME: f64 = 200.0;

    pub const fn new(time_preempt: f64, time_fade_in: f64) -> Self {
        Self {
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
        if curr.base.is_spinner() || curr.idx == 0 {
            return 0.0;
        }

        let curr_obj = curr;
        let next_obj = curr.next(0, diff_objects);

        // * Only allow velocity to buff
        let velocity = f64::max(1.0, curr_obj.lazy_jump_dist / curr_obj.adjusted_delta_time);

        let curr_visible_obj_density =
            self.retrieve_current_visible_object_density(curr_obj, diff_objects);
        let past_obj_difficulty_influence =
            self.get_past_obj_difficulty_influence(curr_obj, diff_objects);

        let constant_angle_nerf_factor =
            Self::get_constant_angle_nerf_factor(curr_obj, diff_objects);

        let note_density_difficulty = Self::calc_density_difficulty(
            next_obj,
            velocity,
            constant_angle_nerf_factor,
            past_obj_difficulty_influence,
            curr_visible_obj_density,
        );

        let hidden_difficulty = if hidden {
            self.calc_hidden_difficulty(
                curr_obj,
                diff_objects,
                past_obj_difficulty_influence,
                curr_visible_obj_density,
                velocity,
                constant_angle_nerf_factor,
            )
        } else {
            0.0
        };

        let preempt_difficulty =
            Self::calc_preempt_difficulty(velocity, constant_angle_nerf_factor, self.time_preempt);

        let mut reading_difficulty = norm(
            1.5,
            [
                preempt_difficulty,
                hidden_difficulty,
                note_density_difficulty,
            ],
        );

        // Having less time to process information is harder
        reading_difficulty *= Self::high_bpm_bonus(curr_obj.adjusted_delta_time);

        reading_difficulty
    }

    fn calc_density_difficulty<'a>(
        next_obj: Option<&'a OsuDifficultyObject<'a>>,
        velocity: f64,
        constant_angle_nerf_factor: f64,
        past_obj_difficulty_influence: f64,
        curr_visible_obj_density: f64,
    ) -> f64 {
        // * Consider future densities too because it can make the path the cursor takes less clear
        let mut fut_obj_difficulty_influence = curr_visible_obj_density.sqrt();

        if let Some(next_obj) = next_obj {
            // * Reduce difficulty if movement to next object is small
            fut_obj_difficulty_influence *= smootherstep(
                next_obj.lazy_jump_dist,
                15.0,
                Self::DISTANCE_INFLUENCE_THRESHOLD,
            );
        }

        // * Value higher note densities exponentially
        let mut note_density_difficulty =
            (past_obj_difficulty_influence + fut_obj_difficulty_influence).powf(1.7)
                * 0.4
                * constant_angle_nerf_factor
                * velocity;

        // * Award only denser than average maps.
        note_density_difficulty =
            (note_density_difficulty - Self::DESNITY_DIFFICULTY_BASE).max(0.0);

        // Apply a soft cap to general density reading to account for partial memorization
        note_density_difficulty.powf(0.45) * Self::DENSITY_MULTIPLIER
    }

    fn calc_preempt_difficulty(
        velocity: f64,
        constant_angle_nerf_factor: f64,
        preempt: f64,
    ) -> f64 {
        // * Arbitrary curve for the base value preempt difficulty should have as approach rate increases.
        // * https://www.desmos.com/calculator/c175335a71
        ((Self::PREEMPT_STARTING_POINT - preempt + (preempt - Self::PREEMPT_STARTING_POINT).abs())
            / 2.0)
            .powf(2.5)
            / Self::PREEMPT_BALANCING_FACTOR
            * constant_angle_nerf_factor
            * velocity
    }

    fn calc_hidden_difficulty<'a>(
        &self,
        curr_obj: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        past_obj_difficulty_influence: f64,
        curr_visible_obj_density: f64,
        velocity: f64,
        constant_angle_nerf_factor: f64,
    ) -> f64 {
        // * Higher preempt means that time spent invisible is higher too, we want to reward that
        let preempt_factor = self.time_preempt.powf(2.2) * 0.01;

        // * Account for both past and current densities
        let density_factor =
            (curr_visible_obj_density + past_obj_difficulty_influence).powf(3.3) * 3.0;

        let mut hidden_difficulty =
            (preempt_factor + density_factor) * constant_angle_nerf_factor * velocity * 0.01;

        // * Apply a soft cap to general HD reading to account for partial memorization
        hidden_difficulty = hidden_difficulty.powf(0.4) * Self::HIDDEN_MULTIPLIER;

        if let Some(prev_obj) = curr_obj.previous(0, diff_objects)
            && FloatExt::eq(curr_obj.lazy_jump_dist, 0.0)
            && FloatExt::eq(
                curr_obj.opacity_at(
                    prev_obj.start_time,
                    true,
                    self.time_preempt,
                    self.time_fade_in,
                ),
                0.0,
            )
            && prev_obj.start_time > curr_obj.start_time - self.time_preempt
        {
            // * Perfect stacks are harder the less time between notes
            hidden_difficulty +=
                Self::HIDDEN_MULTIPLIER * 2500.0 / curr_obj.adjusted_delta_time.powf(1.5);
        }

        hidden_difficulty
    }

    fn get_past_obj_difficulty_influence<'a>(
        &self,
        curr_obj: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        diff_objects
            .iter()
            // Note: This achieves the same as retrievePastVisibleObjects
            .filter(|d| {
                d.idx < curr_obj.idx
                    && curr_obj.start_time - d.start_time <= Self::READING_WINDOW_SIZE
                    && d.start_time >= curr_obj.start_time - self.time_preempt
            })
            .fold(0.0, |past_obj_difficulty_influence, loop_obj| {
                let mut loop_difficulty = curr_obj.opacity_at(
                    loop_obj.start_time,
                    false,
                    self.time_preempt,
                    self.time_fade_in,
                );

                // * When aiming an object small distances mean previous objects may be cheesed, so it doesn't matter whether they were arranged confusingly.
                loop_difficulty *= smootherstep(
                    loop_obj.lazy_jump_dist,
                    15.0,
                    Self::DISTANCE_INFLUENCE_THRESHOLD,
                );

                // * Account less for objects close to the max reading window
                let delta_time = curr_obj.start_time - loop_obj.start_time;
                let time_nerf_factor = Self::get_time_nerf_factor(delta_time);

                loop_difficulty *= time_nerf_factor;
                past_obj_difficulty_influence + loop_difficulty
            })
    }

    // * Returns the density of objects visible at the point in time the current object needs to be clicked capped by the reading window.
    fn retrieve_current_visible_object_density<'a>(
        &self,
        curr_obj: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        let mut forwards_idx = 0;
        let mut visible_object_count = 0.0;

        while let Some(hit_obj) = curr_obj.next(forwards_idx, diff_objects).filter(|next| {
            next.start_time - curr_obj.start_time <= Self::READING_WINDOW_SIZE
                // * Object not visible at the time current object needs to be clicked.
                && curr_obj.start_time >= next.start_time - self.time_preempt
        }) {
            let delta_time = hit_obj.start_time - curr_obj.start_time;
            let time_nerf_factor = Self::get_time_nerf_factor(delta_time);

            visible_object_count += hit_obj.opacity_at(
                curr_obj.start_time,
                false,
                self.time_preempt,
                self.time_fade_in,
            ) * time_nerf_factor;

            forwards_idx += 1;
        }

        visible_object_count
    }

    // * Returns a factor of how often the current object's angle has been repeated in a certain time frame.
    // * It does this by checking the difference in angle between current and past objects and sums them based on a range of similarity.
    // * https://www.desmos.com/calculator/eb057a4822
    fn get_constant_angle_nerf_factor<'a>(
        curr_obj: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        let mut constant_angle_count = 0.0;
        let mut backwards_idx = 0;
        let mut curr_time_gap = 0.0;

        let mut loop_obj_prev0 = curr_obj;
        let mut loop_obj_prev1: Option<&OsuDifficultyObject<'a>> = None;
        let mut loop_obj_prev2: Option<&OsuDifficultyObject<'a>> = None;

        while let Some(loop_obj) = curr_obj
            .previous(backwards_idx, diff_objects)
            .filter(|_| curr_time_gap < Self::MINIMUM_ANGLE_RELEVANCY_TIME)
        {
            // * Account less for objects that are close to the time limit.
            let long_interval_factor = 1.0
                - reverse_lerp(
                    loop_obj.adjusted_delta_time,
                    Self::MAXIMUM_ANGLE_RELEVANCY_TIME,
                    Self::MINIMUM_ANGLE_RELEVANCY_TIME,
                );

            if let (Some(loop_obj_angle), Some(curr_obj_angle)) = (loop_obj.angle, curr_obj.angle) {
                let angle_diff = (curr_obj_angle - loop_obj_angle).abs();
                let mut angle_diff_alternating = f64::consts::PI;

                if let (
                    Some(loop_obj_prev0_angle),
                    Some(loop_obj_prev1_angle),
                    Some(loop_obj_prev2_angle),
                ) = (
                    loop_obj_prev0.angle,
                    loop_obj_prev1.and_then(|o| o.angle),
                    loop_obj_prev2.and_then(|o| o.angle),
                ) {
                    angle_diff_alternating = (loop_obj_prev1_angle - loop_obj_angle).abs();
                    angle_diff_alternating += (loop_obj_prev2_angle - loop_obj_prev0_angle).abs();

                    let mut weight = 1.0;

                    // * Be sure that one of the angles is very sharp, when other is wide
                    weight *= reverse_lerp(
                        loop_obj_angle.min(loop_obj_prev0_angle) * 180.0 / f64::consts::PI,
                        20.0,
                        5.0,
                    );
                    weight *= reverse_lerp(
                        loop_obj_angle.max(loop_obj_prev0_angle) * 180.0 / f64::consts::PI,
                        60.0,
                        120.0,
                    );

                    // * Lerp between max angle difference and rescaled alternating difference, with more harsh scaling compared to normal difference
                    angle_diff_alternating =
                        f64::lerp(f64::consts::PI, 0.1 * angle_diff_alternating, weight);
                }

                let stack_factor = smootherstep(
                    loop_obj.lazy_jump_dist,
                    0.0,
                    f64::from(OsuDifficultyObject::NORMALIZED_RADIUS),
                );

                constant_angle_count += (3.0
                    * (f64::to_radians(30.0)
                        .min((angle_diff).min(angle_diff_alternating) * stack_factor)))
                .cos()
                    * long_interval_factor;
            }

            curr_time_gap = curr_obj.start_time - loop_obj.start_time;
            backwards_idx += 1;

            loop_obj_prev2 = loop_obj_prev1;
            loop_obj_prev1 = Some(loop_obj_prev0);
            loop_obj_prev0 = loop_obj;
        }

        (2.0 / constant_angle_count).clamp(0.2, 1.0)
    }

    // * Returns a nerfing factor for when objects are very distant in time, affecting reading less.
    const fn get_time_nerf_factor(delta_time: f64) -> f64 {
        (2.0 - delta_time / (Self::READING_WINDOW_SIZE / 2.0)).clamp(0.0, 1.0)
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - f64::powf(0.8, ms / 1000.0))
    }
}
