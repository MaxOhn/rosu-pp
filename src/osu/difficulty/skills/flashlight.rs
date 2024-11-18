use std::cmp;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainSkill},
    },
    model::mods::GameMods,
    osu::{difficulty::object::OsuDifficultyObject, object::OsuObjectKind},
    util::strains_vec::StrainsVec,
};

const SKILL_MULTIPLIER: f64 = 0.05512;
const STRAIN_DECAY_BASE: f64 = 0.15;

pub struct Flashlight {
    curr_strain: f64,
    has_hidden_mod: bool,
    inner: StrainSkill,
    evaluator: FlashlightEvaluator,
}

impl Flashlight {
    pub fn new(mods: &GameMods, radius: f64, time_preempt: f64, time_fade_in: f64) -> Self {
        let scaling_factor = 52.0 / radius;

        Self {
            curr_strain: 0.0,
            has_hidden_mod: mods.hd(),
            inner: StrainSkill::default(),
            evaluator: FlashlightEvaluator::new(scaling_factor, time_preempt, time_fade_in),
        }
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn difficulty_value(self) -> f64 {
        Self::static_difficulty_value(self.inner)
    }

    /// Use [`difficulty_value`] instead whenever possible because
    /// [`as_difficulty_value`] clones internally.
    pub fn as_difficulty_value(&self) -> f64 {
        Self::static_difficulty_value(self.inner.clone())
    }

    fn static_difficulty_value(skill: StrainSkill) -> f64 {
        skill.get_curr_strain_peaks().sum()
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        25.0 * (difficulty).powf(2.0)
    }
}

impl ISkill for Flashlight {
    type DifficultyObjects<'a> = [OsuDifficultyObject<'a>];
}

impl<'a> Skill<'a, Flashlight> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        let prev_start_time = curr
            .previous(0, self.diff_objects)
            .map_or(0.0, |prev| prev.start_time);

        self.inner.curr_strain * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    fn curr_section_peak(&self) -> f64 {
        self.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.curr_section_peak
    }

    fn curr_section_end(&self) -> f64 {
        self.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &'a OsuDifficultyObject<'a>) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() =
                (curr.start_time / StrainSkill::SECTION_LEN).ceil() * StrainSkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += StrainSkill::SECTION_LEN;
        }

        let strain_value_at = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }

    fn strain_value_at(&mut self, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        self.inner.curr_strain *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        self.inner.curr_strain += self.inner.evaluator.evaluate_diff_of(
            curr,
            self.diff_objects,
            self.inner.has_hidden_mod,
        ) * SKILL_MULTIPLIER;

        self.inner.curr_strain
    }
}

struct FlashlightEvaluator {
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

    const fn new(scaling_factor: f64, time_preempt: f64, time_fade_in: f64) -> Self {
        Self {
            scaling_factor,
            time_preempt,
            time_fade_in,
        }
    }

    fn evaluate_diff_of<'a>(
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

            let curr_hit_obj = curr_obj.base;

            if !curr_obj.base.is_spinner() {
                let jump_dist = f64::from(
                    (osu_hit_obj.stacked_pos() - curr_hit_obj.stacked_end_pos()).length(),
                );
                cumulative_strain_time += last_obj.strain_time;

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
            let pixel_travel_dist = f64::from(slider.lazy_travel_dist) / self.scaling_factor;

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
