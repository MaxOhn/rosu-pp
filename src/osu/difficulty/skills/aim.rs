use std::f64::consts::{FRAC_PI_2, PI};

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill},
    },
    osu::difficulty::object::OsuDifficultyObject,
    util::{float_ext::FloatExt, strains_vec::StrainsVec},
};

use super::strain::{DifficultyValue, OsuStrainSkill, UsedOsuStrainSkills};

const SKILL_MULTIPLIER: f64 = 25.18;
const STRAIN_DECAY_BASE: f64 = 0.15;

#[derive(Clone)]
pub struct Aim {
    with_sliders: bool,
    curr_strain: f64,
    inner: OsuStrainSkill,
}

impl Aim {
    pub fn new(with_sliders: bool) -> Self {
        Self {
            with_sliders,
            curr_strain: 0.0,
            inner: OsuStrainSkill::default(),
        }
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks().strains()
    }

    pub fn difficulty_value(self) -> UsedOsuStrainSkills<DifficultyValue> {
        Self::static_difficulty_value(self.inner)
    }

    /// Use [`difficulty_value`] instead whenever possible because
    /// [`as_difficulty_value`] clones internally.
    pub fn as_difficulty_value(&self) -> UsedOsuStrainSkills<DifficultyValue> {
        Self::static_difficulty_value(self.inner.clone())
    }

    fn static_difficulty_value(skill: OsuStrainSkill) -> UsedOsuStrainSkills<DifficultyValue> {
        skill.difficulty_value(
            OsuStrainSkill::REDUCED_SECTION_COUNT,
            OsuStrainSkill::REDUCED_STRAIN_BASELINE,
            OsuStrainSkill::DECAY_WEIGHT,
        )
    }
}

impl ISkill for Aim {
    type DifficultyObjects<'a> = [OsuDifficultyObject<'a>];
}

impl<'a> Skill<'a, Aim> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        let prev_start_time = curr
            .previous(0, self.diff_objects)
            .map_or(0.0, |prev| prev.start_time);

        self.inner.curr_strain * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    fn curr_section_peak(&self) -> f64 {
        self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_end(&self) -> f64 {
        self.inner.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &'a OsuDifficultyObject<'a>) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / OsuStrainSkill::SECTION_LEN).ceil()
                * OsuStrainSkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += OsuStrainSkill::SECTION_LEN;
        }

        let strain_value_at = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }

    fn strain_value_at(&mut self, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        self.inner.curr_strain *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        self.inner.curr_strain +=
            AimEvaluator::evaluate_diff_of(curr, self.diff_objects, self.inner.with_sliders)
                * SKILL_MULTIPLIER;
        self.inner.inner.object_strains.push(self.inner.curr_strain);

        self.inner.curr_strain
    }
}

struct AimEvaluator;

impl AimEvaluator {
    const WIDE_ANGLE_MULTIPLIER: f64 = 1.5;
    const ACUTE_ANGLE_MULTIPLIER: f64 = 1.95;
    const SLIDER_MULTIPLIER: f64 = 1.35;
    const VELOCITY_CHANGE_MULTIPLIER: f64 = 0.75;

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        with_slider_travel_dist: bool,
    ) -> f64 {
        let osu_curr_obj = curr;

        let Some((osu_last_last_obj, osu_last_obj)) = curr
            .previous(1, diff_objects)
            .zip(curr.previous(0, diff_objects))
            .filter(|(_, last)| !(curr.base.is_spinner() || last.base.is_spinner()))
        else {
            return 0.0;
        };

        // * Calculate the velocity to the current hitobject, which starts
        // * with a base distance / time assuming the last object is a hitcircle.
        let mut curr_vel = osu_curr_obj.lazy_jump_dist / osu_curr_obj.strain_time;

        // * But if the last object is a slider, then we extend the travel
        // * velocity through the slider into the current object.
        if osu_last_obj.base.is_slider() && with_slider_travel_dist {
            // * calculate the slider velocity from slider head to slider end.
            let travel_vel = osu_last_obj.travel_dist / osu_last_obj.travel_time;
            // * calculate the movement velocity from slider end to current object
            let movement_vel = osu_curr_obj.min_jump_dist / osu_curr_obj.min_jump_time;

            // * take the larger total combined velocity.
            curr_vel = curr_vel.max(movement_vel + travel_vel);
        }

        // * As above, do the same for the previous hitobject.
        let mut prev_vel = osu_last_obj.lazy_jump_dist / osu_last_obj.strain_time;

        if osu_last_last_obj.base.is_slider() && with_slider_travel_dist {
            let travel_vel = osu_last_last_obj.travel_dist / osu_last_last_obj.travel_time;
            let movement_vel = osu_last_obj.min_jump_dist / osu_last_obj.min_jump_time;

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
                .angle
                .zip(osu_last_obj.angle)
                .zip(osu_last_last_obj.angle)
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
                        * ((osu_curr_obj.lazy_jump_dist).clamp(50.0, 100.0) - 50.0)
                        / 50.0)
                        .sin();

                    // * Multiply by previous angle, we don't want to buff unless this is a wiggle type pattern.
                    acute_angle_bonus *= Self::calc_acute_angle_bonus(last_angle)
                    // * The maximum velocity we buff is equal to 125 / strainTime
                        * angle_bonus.min(125.0 / osu_curr_obj.strain_time)
                        // * scale buff from 150 bpm 1/4 to 200 bpm 1/4
                        * base1.powf(2.0)
                         // * Buff distance exceeding 50 (radius) up to 100 (diameter).
                        * base2.powf(2.0);
                }

                // * Penalize wide angles if they're repeated, reducing the penalty as the lastAngle gets more acute.
                wide_angle_bonus *= angle_bonus
                    * (1.0
                        - wide_angle_bonus.min(Self::calc_wide_angle_bonus(last_angle).powf(3.0)));
                // * Penalize acute angles if they're repeated, reducing the penalty as the lastLastAngle gets more obtuse.
                acute_angle_bonus *= 0.5
                    + 0.5
                        * (1.0
                            - acute_angle_bonus
                                .min(Self::calc_acute_angle_bonus(last_last_angle).powf(3.0)));
            }
        }

        if prev_vel.max(curr_vel).not_eq(0.0) {
            // * We want to use the average velocity over the whole object when awarding
            // * differences, not the individual jump and slider path velocities.
            prev_vel = (osu_last_obj.lazy_jump_dist + osu_last_last_obj.travel_dist)
                / osu_last_obj.strain_time;
            curr_vel =
                (osu_curr_obj.lazy_jump_dist + osu_last_obj.travel_dist) / osu_curr_obj.strain_time;

            // * Scale with ratio of difference compared to 0.5 * max dist.
            let dist_ratio_base =
                (FRAC_PI_2 * (prev_vel - curr_vel).abs() / prev_vel.max(curr_vel)).sin();
            let dist_ratio = dist_ratio_base.powf(2.0);

            // * Reward for % distance up to 125 / strainTime for overlaps where velocity is still changing.
            let overlap_vel_buff = (125.0 / osu_curr_obj.strain_time.min(osu_last_obj.strain_time))
                .min((prev_vel - curr_vel).abs());

            vel_change_bonus = overlap_vel_buff * dist_ratio;

            // * Penalize for rhythm changes.
            let bonus_base = (osu_curr_obj.strain_time).min(osu_last_obj.strain_time)
                / (osu_curr_obj.strain_time).max(osu_last_obj.strain_time);
            vel_change_bonus *= bonus_base.powf(2.0);
        }

        if osu_last_obj.base.is_slider() {
            // * Reward sliders based on velocity.
            slider_bonus = osu_last_obj.travel_dist / osu_last_obj.travel_time;
        }

        // * Add in acute angle bonus or wide angle bonus + velocity change bonus, whichever is larger.
        aim_strain += (acute_angle_bonus * Self::ACUTE_ANGLE_MULTIPLIER).max(
            wide_angle_bonus * Self::WIDE_ANGLE_MULTIPLIER
                + vel_change_bonus * Self::VELOCITY_CHANGE_MULTIPLIER,
        );

        // * Add in additional slider velocity bonus.
        if with_slider_travel_dist {
            aim_strain += slider_bonus * Self::SLIDER_MULTIPLIER;
        }

        aim_strain
    }

    fn calc_wide_angle_bonus(angle: f64) -> f64 {
        (3.0 / 4.0 * ((5.0 / 6.0 * PI).min(angle.max(PI / 6.0)) - PI / 6.0))
            .sin()
            .powf(2.0)
    }

    fn calc_acute_angle_bonus(angle: f64) -> f64 {
        1.0 - Self::calc_wide_angle_bonus(angle)
    }
}
