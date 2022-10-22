use std::f64::consts::{FRAC_PI_2, PI};

use crate::osu::difficulty_object::OsuDifficultyObject;

use super::{previous, previous_start_time, OsuStrainSkill, Skill, StrainSkill};

#[derive(Clone, Debug)]
pub(crate) struct Aim {
    curr_strain: f64,
    curr_section_peak: f64,
    curr_section_end: f64,
    pub(crate) strain_peaks: Vec<f64>,
    with_sliders: bool,
}

impl Aim {
    const SKILL_MULTIPLIER: f64 = 23.55;
    const STRAIN_DECAY_BASE: f64 = 0.15;

    pub(crate) fn new(with_sliders: bool) -> Self {
        Self {
            curr_strain: 0.0,
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            strain_peaks: Vec::new(),
            with_sliders,
        }
    }

    fn strain_decay(ms: f64) -> f64 {
        Self::STRAIN_DECAY_BASE.powf(ms / 1000.0)
    }
}

impl Skill for Aim {
    #[inline]
    fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        hit_window: f64,
    ) {
        <Self as StrainSkill>::process(self, curr, diff_objects, hit_window)
    }

    #[inline]
    fn difficulty_value(&mut self) -> f64 {
        <Self as OsuStrainSkill>::difficulty_value(self)
    }
}

impl StrainSkill for Aim {
    #[inline]
    fn strain_peaks_mut(&mut self) -> &mut Vec<f64> {
        &mut self.strain_peaks
    }

    #[inline]
    fn curr_section_peak(&mut self) -> &mut f64 {
        &mut self.curr_section_peak
    }

    #[inline]
    fn curr_section_end(&mut self) -> &mut f64 {
        &mut self.curr_section_end
    }

    #[inline]
    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        _hit_window: f64,
    ) -> f64 {
        self.curr_strain *= Self::strain_decay(curr.delta_time);
        self.curr_strain += AimEvaluator::evaluate_diff_of(curr, diff_objects, self.with_sliders)
            * Self::SKILL_MULTIPLIER;

        self.curr_strain
    }

    #[inline]
    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.curr_strain * Self::strain_decay(time - previous_start_time(diff_objects, curr.idx, 0))
    }

    #[inline]
    fn difficulty_value(&mut self) -> f64 {
        <Self as OsuStrainSkill>::difficulty_value(self)
    }
}

impl OsuStrainSkill for Aim {}

struct AimEvaluator;

impl AimEvaluator {
    const WIDE_ANGLE_MULTIPLIER: f64 = 1.5;
    const ACUTE_ANGLE_MULTIPLIER: f64 = 1.95;
    const SLIDER_MULTIPLIER: f64 = 1.35;
    const VELOCITY_CHANGE_MULTIPLIER: f64 = 0.75;

    fn evaluate_diff_of(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        with_sliders: bool,
    ) -> f64 {
        let osu_curr_obj = curr;

        let (osu_last_last_obj, osu_last_obj) = if let Some(tuple) =
            previous(diff_objects, curr.idx, 1)
                .zip(previous(diff_objects, curr.idx, 0))
                .filter(|(_, last)| !(curr.base.is_spinner() || last.base.is_spinner()))
        {
            tuple
        } else {
            return 0.0;
        };

        // * Calculate the velocity to the current hitobject, which starts
        // * with a base distance / time assuming the last object is a hitcircle.
        let mut curr_vel = osu_curr_obj.dists.lazy_jump_dist / osu_curr_obj.strain_time;

        // * But if the last object is a slider, then we extend the travel
        // * velocity through the slider into the current object.
        if osu_last_obj.base.is_slider() && with_sliders {
            // * calculate the slider velocity from slider head to slider end.
            let travel_vel = osu_last_obj.dists.travel_dist / osu_last_obj.dists.travel_time;
            // * calculate the movement velocity from slider end to current object
            let movement_vel = osu_curr_obj.dists.min_jump_dist / osu_curr_obj.dists.min_jump_time;

            // * take the larger total combined velocity.
            curr_vel = curr_vel.max(movement_vel + travel_vel);
        }

        // * As above, do the same for the previous hitobject.
        let mut prev_vel = osu_last_obj.dists.lazy_jump_dist / osu_last_obj.strain_time;

        if osu_last_last_obj.base.is_slider() && with_sliders {
            let travel_vel =
                osu_last_last_obj.dists.travel_dist / osu_last_last_obj.dists.travel_time;
            let movement_vel = osu_last_obj.dists.min_jump_dist / osu_last_obj.dists.min_jump_time;

            prev_vel = prev_vel.max(movement_vel + travel_vel);
        }

        let mut wide_angle_bonus = 0.0;
        let mut acute_angle_bonus = 0.0;
        let mut slider_bonus = 0.0;
        let mut vel_change_bonus = 0.0;

        // * Start strain with regular velocity.
        let mut aim_strain = curr_vel;

        // * If rhythms are the same.
        if osu_curr_obj.strain_time.max(osu_last_obj.strain_time)
            < 1.25 * osu_curr_obj.strain_time.min(osu_last_obj.strain_time)
        {
            if let Some(((curr_angle, last_angle), last_last_angle)) = osu_curr_obj
                .dists
                .angle
                .zip(osu_last_obj.dists.angle)
                .zip(osu_last_last_obj.dists.angle)
            {
                // * Rewarding angles, take the smaller velocity as base.
                let angle_bonus = curr_vel.min(prev_vel);

                wide_angle_bonus = Self::calc_wide_angle_bonus(curr_angle);
                acute_angle_bonus = Self::calc_acute_angle_bonus(curr_angle);

                // * Only buff deltaTime exceeding 300 bpm 1/2.
                if osu_curr_obj.strain_time > 100.0 {
                    acute_angle_bonus = 0.0;
                } else {
                    let base1 =
                        (FRAC_PI_2 * ((100.0 - osu_curr_obj.strain_time) / 25.0).min(1.0)).sin();

                    let base2 = (FRAC_PI_2
                        * ((osu_curr_obj.dists.lazy_jump_dist).clamp(50.0, 100.0) - 50.0)
                        / 50.0)
                        .sin();

                    // * Multiply by previous angle, we don't want to buff unless this is a wiggle type pattern.
                    acute_angle_bonus *= Self::calc_acute_angle_bonus(last_angle)
                    // * The maximum velocity we buff is equal to 125 / strainTime
                        * angle_bonus.min(125.0 / osu_curr_obj.strain_time)
                        // * scale buff from 150 bpm 1/4 to 200 bpm 1/4
                        * base1
                        * base1
                         // * Buff distance exceeding 50 (radius) up to 100 (diameter).
                        * base2
                        * base2;
                }

                // * Penalize wide angles if they're repeated, reducing the penalty as the lastAngle gets more acute.
                wide_angle_bonus *= angle_bonus
                    * (1.0 - wide_angle_bonus.min(Self::calc_wide_angle_bonus(last_angle).powi(3)));
                // * Penalize acute angles if they're repeated, reducing the penalty as the lastLastAngle gets more obtuse.
                acute_angle_bonus *= 0.5
                    + 0.5
                        * (1.0
                            - acute_angle_bonus
                                .min(Self::calc_acute_angle_bonus(last_last_angle).powi(3)));
            }
        }

        if prev_vel.max(curr_vel).abs() > f64::EPSILON {
            // * We want to use the average velocity over the whole object when awarding
            // * differences, not the individual jump and slider path velocities.
            prev_vel = (osu_last_obj.dists.lazy_jump_dist + osu_last_last_obj.dists.travel_dist)
                / osu_last_obj.strain_time;
            curr_vel = (osu_curr_obj.dists.lazy_jump_dist + osu_last_obj.dists.travel_dist)
                / osu_curr_obj.strain_time;

            // * Scale with ratio of difference compared to 0.5 * max dist.
            let dist_ratio_base =
                (FRAC_PI_2 * (prev_vel - curr_vel).abs() / prev_vel.max(curr_vel)).sin();
            let dist_ratio = dist_ratio_base * dist_ratio_base;

            // * Reward for % distance up to 125 / strainTime for overlaps where velocity is still changing.
            let overlap_vel_buff = (125.0 / osu_curr_obj.strain_time.min(osu_last_obj.strain_time))
                .min((prev_vel - curr_vel).abs());

            vel_change_bonus = overlap_vel_buff * dist_ratio;

            // * Penalize for rhythm changes.
            let bonus_base = (osu_curr_obj.strain_time).min(osu_last_obj.strain_time)
                / (osu_curr_obj.strain_time).max(osu_last_obj.strain_time);
            vel_change_bonus *= bonus_base * bonus_base;
        }

        if osu_last_obj.base.is_slider() {
            // * Reward sliders based on velocity.
            slider_bonus = osu_last_obj.dists.travel_dist / osu_last_obj.dists.travel_time
        }

        // * Add in acute angle bonus or wide angle bonus + velocity change bonus, whichever is larger.
        aim_strain += (acute_angle_bonus * Self::ACUTE_ANGLE_MULTIPLIER).max(
            wide_angle_bonus * Self::WIDE_ANGLE_MULTIPLIER
                + vel_change_bonus * Self::VELOCITY_CHANGE_MULTIPLIER,
        );

        // * Add in additional slider velocity bonus.
        if with_sliders {
            aim_strain += slider_bonus * Self::SLIDER_MULTIPLIER;
        }

        aim_strain
    }

    fn calc_wide_angle_bonus(angle: f64) -> f64 {
        let base = (3.0 / 4.0 * ((5.0 / 6.0 * PI).min(angle.max(PI / 6.0)) - PI / 6.0)).sin();

        base * base
    }

    fn calc_acute_angle_bonus(angle: f64) -> f64 {
        1.0 - Self::calc_wide_angle_bonus(angle)
    }
}
