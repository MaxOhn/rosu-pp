use std::ops::Not;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{
            strain_decay, DifficultyValue, ISkill, Skill, StrainDecaySkill, StrainSkill,
            UsedStrainSkills,
        },
    },
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    util::{difficulty::logistic_exp, strains_vec::StrainsVec, sync::Weak},
};

const SKILL_MULTIPLIER: f64 = 1.1;
const STRAIN_DECAY_BASE: f64 = 0.4;

#[derive(Clone)]
pub struct Stamina {
    inner: StrainSkill,
    single_color: bool,
    curr_strain: f64,
    is_convert: bool,
}

impl Stamina {
    pub fn new(single_color: bool, is_convert: bool) -> Self {
        Self {
            inner: StrainSkill::default(),
            single_color,
            curr_strain: 0.0,
            is_convert,
        }
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks().into_strains()
    }

    pub fn as_difficulty_value(&self) -> UsedStrainSkills<DifficultyValue> {
        self.inner
            .clone()
            .difficulty_value(StrainDecaySkill::DECAY_WEIGHT)
    }
}

impl ISkill for Stamina {
    type DifficultyObjects<'a> = TaikoDifficultyObjects;
}

impl Skill<'_, Stamina> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &TaikoDifficultyObject) -> f64 {
        if self.inner.single_color {
            return 0.0;
        }

        let prev_start_time = curr
            .previous(0, &self.diff_objects.objects)
            .map_or(0.0, |prev| prev.get().start_time);

        self.curr_strain() * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    const fn curr_strain(&self) -> f64 {
        self.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.curr_strain
    }

    const fn curr_section_peak(&self) -> f64 {
        self.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.curr_section_peak
    }

    const fn curr_section_end(&self) -> f64 {
        self.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &TaikoDifficultyObject) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / StrainDecaySkill::SECTION_LEN).ceil()
                * StrainDecaySkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += StrainDecaySkill::SECTION_LEN;
        }

        let strain_value_at = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
        self.inner.inner.object_strains.push(strain_value_at);
    }

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.curr_strain_mut() +=
            StaminaEvaluator::evaluate_diff_of(curr, self.diff_objects) * SKILL_MULTIPLIER;

        // * Safely prevents previous strains from shifting as new notes are added.
        let index = curr
            .color_data
            .mono_streak
            .as_ref()
            .and_then(Weak::upgrade)
            .and_then(|mono| {
                mono.get().hit_objects.iter().position(|h| {
                    let Some(h) = h.upgrade() else { return false };
                    let h = h.get();

                    h.idx == curr.idx
                })
            })
            .unwrap_or(0) as isize;

        if self.inner.single_color {
            logistic_exp(-(index - 10) as f64 / 2.0, Some(self.curr_strain()))
        } else if self.inner.is_convert {
            self.curr_strain()
        } else {
            #[allow(clippy::manual_clamp)]
            let monolength_bonus = 1.0 + f64::min(f64::max((index - 5) as f64 / 50.0, 0.0), 0.30);

            self.curr_strain() * monolength_bonus
        }
    }
}

pub(super) struct StaminaEvaluator;

impl StaminaEvaluator {
    pub(super) fn evaluate_diff_of(
        curr: &TaikoDifficultyObject,
        hit_objects: &TaikoDifficultyObjects,
    ) -> f64 {
        if curr.base_hit_type.is_hit().not() {
            return 0.0;
        }

        // * Find the previous hit object hit by the current finger, which is n notes prior, n being the number of
        // * available fingers.
        let prev = curr.previous(1, &hit_objects.objects);
        let prev_mono =
            hit_objects.previous_mono(curr, Self::available_fingers_for(curr, hit_objects) - 1);

        // * Add a base strain to all objects
        let mut object_strain = 0.5;

        let Some(prev) = prev else {
            return object_strain;
        };

        if let Some(prev_mono) = prev_mono {
            object_strain += Self::speed_bonus(curr.start_time - prev_mono.get().start_time)
                + 0.5 * Self::speed_bonus(curr.start_time - prev.get().start_time);
        }

        object_strain
    }

    fn available_fingers_for(
        hit_object: &TaikoDifficultyObject,
        hit_objects: &TaikoDifficultyObjects,
    ) -> usize {
        let prev_color_change = hit_object.color_data.previous_color_change(hit_objects);

        if prev_color_change
            .is_some_and(|change| hit_object.start_time - change.get().start_time < 300.0)
        {
            return 2;
        }

        let next_color_change = hit_object.color_data.next_color_change(hit_objects);

        if next_color_change
            .is_some_and(|change| change.get().start_time - hit_object.start_time < 300.0)
        {
            return 2;
        }

        8
    }

    fn speed_bonus(mut interval: f64) -> f64 {
        // * Interval is capped at a very small value to prevent infinite values.
        interval = f64::max(interval, 1.0);

        20.0 / interval
    }
}
