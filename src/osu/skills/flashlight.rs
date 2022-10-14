use std::{any::Any, mem};

use crate::{
    osu::{difficulty_object::OsuDifficultyObject, osu_object::OsuObjectKind},
    Mods,
};

use super::{previous, previous_start_time, OsuStrainSkill, Skill, StrainSkill};

#[derive(Clone)]
pub(crate) struct Flashlight {
    curr_strain: f64,
    curr_section_peak: f64,
    curr_section_end: f64,
    strain_peaks: Vec<f64>,
    has_hidden_mod: bool,
    scaling_factor: f64,
    time_preempt: f64,
    time_fade_in: f64,
}

impl Flashlight {
    const SKILL_MULTIPLIER: f64 = 0.052;
    const STRAIN_DECAY_BASE: f64 = 0.15;

    pub(crate) fn new(mods: u32, radius: f32, time_preempt: f64, time_fade_in: f64) -> Self {
        Self {
            curr_strain: 0.0,
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            strain_peaks: Vec::new(),
            has_hidden_mod: mods.hd(),
            scaling_factor: 52.0 / radius as f64,
            time_preempt,
            time_fade_in,
        }
    }

    fn strain_decay(ms: f64) -> f64 {
        Self::STRAIN_DECAY_BASE.powf(ms / 1000.0)
    }
}

impl Skill for Flashlight {
    fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        hit_window: f64,
    ) {
        <Self as StrainSkill>::process(self, curr, diff_objects, hit_window)
    }

    fn difficulty_value(&mut self) -> f64 {
        <Self as StrainSkill>::difficulty_value(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn take_strain_peaks(&mut self) -> Vec<f64> {
        mem::take(&mut self.strain_peaks)
    }
}

impl StrainSkill for Flashlight {
    const DECAY_WEIGHT: f64 = 0.9;

    fn strain_peaks_mut(&mut self) -> &mut Vec<f64> {
        &mut self.strain_peaks
    }

    fn curr_section_peak(&mut self) -> &mut f64 {
        &mut self.curr_section_peak
    }

    fn curr_section_end(&mut self) -> &mut f64 {
        &mut self.curr_section_end
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        _hit_window: f64,
    ) -> f64 {
        self.curr_strain *= Self::strain_decay(curr.delta_time);
        self.curr_strain += FlashlightEvaluator::evaluate_diff_of(
            curr,
            diff_objects,
            self.has_hidden_mod,
            self.scaling_factor,
            self.time_preempt,
            self.time_fade_in,
        ) * Self::SKILL_MULTIPLIER;

        self.curr_strain
    }

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.curr_strain * Self::strain_decay(time - previous_start_time(diff_objects, curr.idx, 0))
    }

    fn difficulty_value(&mut self) -> f64 {
        self.get_curr_strain_peaks().into_iter().sum::<f64>() * Self::DIFFICULTY_MULTIPLER
    }
}

impl OsuStrainSkill for Flashlight {}

struct FlashlightEvaluator;

impl FlashlightEvaluator {
    const MAX_OPACITY_BONUS: f64 = 0.4;
    const HIDDEN_BONUS: f64 = 0.2;

    const MIN_VELOCITY: f64 = 0.5;
    const SLIDER_MULTIPLIER: f64 = 1.3;

    const MIN_ANGLE_MULTIPLIER: f64 = 0.2;

    fn evaluate_diff_of(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        hidden: bool,
        scaling_factor: f64,
        time_preempt: f64,
        time_fade_in: f64,
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
        for i in 0..curr.idx.min(10) {
            let curr_obj = if let Some(curr_obj) = previous(diff_objects, curr.idx, i) {
                curr_obj
            } else {
                break;
            };

            let curr_hit_obj = curr_obj.base;

            if !curr_obj.base.is_spinner() {
                let jump_dist =
                    (osu_hit_obj.stacked_pos() - curr_hit_obj.stacked_end_pos()).length() as f64;
                cumulative_strain_time += last_obj.strain_time;

                // * We want to nerf objects that can be easily seen within the Flashlight circle radius.
                if i == 0 {
                    small_dist_nerf = (jump_dist / 75.0).min(1.0);
                }

                // * We also want to nerf stacks so that only the first object of the stack is accounted for.
                let stack_nerf = ((curr_obj.dists.lazy_jump_dist / scaling_factor) / 25.0).min(1.0);

                // * Bonus based on how visible the object is.
                let opacity_bonus = 1.0
                    + Self::MAX_OPACITY_BONUS
                        * (1.0
                            - osu_curr.opacity_at(
                                curr_hit_obj.start_time,
                                hidden,
                                time_preempt,
                                time_fade_in,
                            ));

                result += stack_nerf * opacity_bonus * scaling_factor * jump_dist
                    / cumulative_strain_time;

                if let Some((curr_obj_angle, osu_curr_angle)) =
                    curr_obj.dists.angle.zip(osu_curr.dists.angle)
                {
                    // * Objects further back in time should count less for the nerf.
                    if (curr_obj_angle - osu_curr_angle).abs() < 0.02 {
                        angle_repeat_count += (1.0 - 0.1 * i as f64).max(0.0);
                    }
                }
            }

            last_obj = curr_obj;
        }

        let base = small_dist_nerf * result;
        result = base * base;

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
            let pixel_travel_dist = osu_curr.dists.lazy_travel_dist as f64 / scaling_factor;

            // * Reward sliders based on velocity.
            slider_bonus = ((pixel_travel_dist / osu_curr.dists.travel_time as f64
                - Self::MIN_VELOCITY)
                .max(0.0))
            .sqrt();

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
