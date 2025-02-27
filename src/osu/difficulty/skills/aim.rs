use std::f64::consts::FRAC_PI_2;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, DifficultyValue, ISkill, Skill, UsedStrainSkills},
    },
    osu::difficulty::object::OsuDifficultyObject,
    util::{
        difficulty::{milliseconds_to_bpm, reverse_lerp, smootherstep, smoothstep},
        float_ext::FloatExt,
        strains_vec::StrainsVec,
    },
};

use super::strain::OsuStrainSkill;

const SKILL_MULTIPLIER: f64 = 25.6;
const STRAIN_DECAY_BASE: f64 = 0.15;

#[derive(Clone)]
pub struct Aim {
    include_sliders: bool,
    curr_strain: f64,
    slider_strains: Vec<f64>, // TODO: use StrainVec?
    inner: OsuStrainSkill,
}

impl Aim {
    pub fn new(include_sliders: bool) -> Self {
        Self {
            include_sliders,
            curr_strain: 0.0,
            slider_strains: Vec::with_capacity(64), // TODO: check default
            inner: OsuStrainSkill::default(),
        }
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.inner.get_curr_strain_peaks().into_strains()
    }

    pub fn difficulty_value(self) -> UsedStrainSkills<AimResidue> {
        Self::static_difficulty_value(self.inner, self.slider_strains)
    }

    /// Use [`difficulty_value`] instead whenever possible because
    /// [`as_difficulty_value`] clones internally.
    pub fn as_difficulty_value(&self) -> UsedStrainSkills<AimResidue> {
        Self::static_difficulty_value(self.inner.clone(), self.slider_strains.clone())
    }

    fn static_difficulty_value(
        skill: OsuStrainSkill,
        slider_strains: Vec<f64>,
    ) -> UsedStrainSkills<AimResidue> {
        let used = skill.difficulty_value(
            OsuStrainSkill::REDUCED_SECTION_COUNT,
            OsuStrainSkill::REDUCED_STRAIN_BASELINE,
            OsuStrainSkill::DECAY_WEIGHT,
        );

        UsedStrainSkills {
            value: AimResidue {
                difficulty_value: used.value.0,
                slider_strains,
            },
            object_strains: used.object_strains,
        }
    }
}

pub(crate) struct AimResidue {
    difficulty_value: f64,
    slider_strains: Vec<f64>,
}

impl UsedStrainSkills<AimResidue> {
    pub const fn difficulty_value(&self) -> f64 {
        self.value.difficulty_value
    }

    pub fn count_top_weighted_strains(&self) -> f64 {
        UsedStrainSkills::<DifficultyValue>::static_count_top_weighted_strains(
            &self.object_strains,
            self.value.difficulty_value,
        )
    }

    pub fn get_difficult_sliders(&self) -> f64 {
        if self.value.slider_strains.is_empty() {
            return 0.0;
        }

        let max_slider_strain = self
            .value
            .slider_strains
            .iter()
            .fold(0.0, |max, next| f64::max(max, *next));

        if max_slider_strain.eq(0.0) {
            return 0.0;
        }

        self.value
            .slider_strains
            .iter()
            .copied()
            .map(|strain| 1.0 / (1.0 + f64::exp(-(strain / max_slider_strain * 12.0 - 6.0))))
            .sum()
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

        let strain = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain.max(self.curr_section_peak());
        self.inner.inner.inner.object_strains.push(strain);
    }

    fn strain_value_at(&mut self, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        self.inner.curr_strain *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        self.inner.curr_strain +=
            AimEvaluator::evaluate_diff_of(curr, self.diff_objects, self.inner.include_sliders)
                * SKILL_MULTIPLIER;

        if curr.base.is_slider() {
            self.inner.slider_strains.push(self.inner.curr_strain);
        }

        self.inner.curr_strain
    }
}

struct AimEvaluator;

impl AimEvaluator {
    const WIDE_ANGLE_MULTIPLIER: f64 = 1.5;
    const ACUTE_ANGLE_MULTIPLIER: f64 = 2.6;
    const SLIDER_MULTIPLIER: f64 = 1.35;
    const VELOCITY_CHANGE_MULTIPLIER: f64 = 0.75;
    const WIGGLE_MULTIPLIER: f64 = 1.02;

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

        #[allow(clippy::items_after_statements)]
        const RADIUS: i32 = OsuDifficultyObject::NORMALIZED_RADIUS;
        #[allow(clippy::items_after_statements)]
        const DIAMETER: i32 = OsuDifficultyObject::NORMALIZED_DIAMETER;

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
        let mut wiggle_bonus = 0.0;

        // * Start strain with regular velocity.
        let mut aim_strain = curr_vel;

        // * If rhythms are the same.
        if osu_curr_obj.strain_time.max(osu_last_obj.strain_time)
            < 1.25 * osu_curr_obj.strain_time.min(osu_last_obj.strain_time)
        {
            if let Some((curr_angle, last_angle)) = osu_curr_obj.angle.zip(osu_last_obj.angle) {
                // * Rewarding angles, take the smaller velocity as base.
                let angle_bonus = curr_vel.min(prev_vel);

                wide_angle_bonus = Self::calc_wide_angle_bonus(curr_angle);
                acute_angle_bonus = Self::calc_acute_angle_bonus(curr_angle);

                // * Penalize angle repetition.
                wide_angle_bonus *= 1.0
                    - f64::min(
                        wide_angle_bonus,
                        f64::powf(Self::calc_wide_angle_bonus(last_angle), 3.0),
                    );
                acute_angle_bonus *= 0.08
                    + 0.92
                        * (1.0
                            - f64::min(
                                acute_angle_bonus,
                                f64::powf(Self::calc_acute_angle_bonus(last_angle), 3.0),
                            ));

                // * Apply full wide angle bonus for distance more than one diameter
                wide_angle_bonus *= angle_bonus
                    * smootherstep(osu_curr_obj.lazy_jump_dist, 0.0, f64::from(DIAMETER));

                // * Apply acute angle bonus for BPM above 300 1/2 and distance more than one diameter
                acute_angle_bonus *= angle_bonus
                    * smootherstep(
                        milliseconds_to_bpm(osu_curr_obj.strain_time, Some(2)),
                        300.0,
                        400.0,
                    )
                    * smootherstep(
                        osu_curr_obj.lazy_jump_dist,
                        f64::from(DIAMETER),
                        f64::from(DIAMETER * 2),
                    );

                // * Apply wiggle bonus for jumps that are [radius, 3*diameter] in distance, with < 110 angle
                // * https://www.desmos.com/calculator/dp0v0nvowc
                wiggle_bonus = angle_bonus
                    * smootherstep(
                        osu_curr_obj.lazy_jump_dist,
                        f64::from(RADIUS),
                        f64::from(DIAMETER),
                    )
                    * f64::powf(
                        reverse_lerp(
                            osu_curr_obj.lazy_jump_dist,
                            f64::from(DIAMETER * 3),
                            f64::from(DIAMETER),
                        ),
                        1.8,
                    )
                    * smootherstep(curr_angle, f64::to_radians(110.0), f64::to_radians(60.0))
                    * smootherstep(
                        osu_last_obj.lazy_jump_dist,
                        f64::from(RADIUS),
                        f64::from(DIAMETER),
                    )
                    * f64::powf(
                        reverse_lerp(
                            osu_last_obj.lazy_jump_dist,
                            f64::from(DIAMETER * 3),
                            f64::from(DIAMETER),
                        ),
                        1.8,
                    )
                    * smootherstep(last_angle, f64::to_radians(110.0), f64::to_radians(60.0));
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
            let overlap_vel_buff = (f64::from(DIAMETER) * 1.25
                / osu_curr_obj.strain_time.min(osu_last_obj.strain_time))
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

        aim_strain += wiggle_bonus * Self::WIGGLE_MULTIPLIER;

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
        smoothstep(angle, f64::to_radians(40.0), f64::to_radians(140.0))
    }

    fn calc_acute_angle_bonus(angle: f64) -> f64 {
        smoothstep(angle, f64::to_radians(140.0), f64::to_radians(40.0))
    }
}
