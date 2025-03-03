use std::ops::Not;

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainDecaySkill},
    },
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    util::{difficulty::logistic, strains_vec::StrainsVec, sync::Weak},
};

const SKILL_MULTIPLIER: f64 = 1.0;
const STRAIN_DECAY_BASE: f64 = 0.4;

#[derive(Clone, Default)]
pub struct Reading {
    inner: StrainDecaySkill,
}

impl Reading {
    const fn curr_strain(&self) -> f64 {
        self.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.curr_strain
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn as_difficulty_value(&self) -> f64 {
        self.inner
            .clone()
            .difficulty_value(StrainDecaySkill::DECAY_WEIGHT)
            .difficulty_value()
    }
}

impl ISkill for Reading {
    type DifficultyObjects<'a> = TaikoDifficultyObjects;
}

impl Skill<'_, Reading> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &TaikoDifficultyObject) -> f64 {
        let prev_start_time = curr
            .previous(0, &self.diff_objects.objects)
            .map_or(0.0, |prev| prev.get().start_time);

        self.inner.curr_strain() * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    const fn curr_section_peak(&self) -> f64 {
        self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_peak
    }

    const fn curr_section_end(&self) -> f64 {
        self.inner.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_end
    }

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.inner.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.inner.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.inner.curr_strain()
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
    }

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        // * Drum Rolls and Swells are exempt.
        if curr.base_hit_type.is_hit().not() {
            return 0.0;
        }

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

        *self.inner.curr_strain_mut() = logistic(index as f64, 4.0, -1.0 / 25.0, Some(0.5)) + 0.5;
        *self.inner.curr_strain_mut() *= STRAIN_DECAY_BASE;
        *self.inner.curr_strain_mut() +=
            ReadingEvaluator::evaluate_diff_of(curr) * SKILL_MULTIPLIER;

        self.inner.curr_strain()
    }
}

struct ReadingEvaluator;

impl ReadingEvaluator {
    fn evaluate_diff_of(note_object: &TaikoDifficultyObject) -> f64 {
        let high_velocity = VelocityRange::new(480.0, 640.0);
        let mid_velocity = VelocityRange::new(360.0, 480.0);

        // * Apply a cap to prevent outlier values on maps that exceed the editor's parameters.
        let effective_bpm = f64::max(1.0, note_object.effective_bpm);

        let mid_velocity_diff = 0.5
            * logistic(
                effective_bpm,
                mid_velocity.center(),
                1.0 / (mid_velocity.range() / 10.0),
                None,
            );

        // * Expected DeltaTime is the DeltaTime this note would need to be spaced equally to a base slider velocity 1/4 note.
        let expected_delta_time = 21_000.0 / effective_bpm;
        let object_density = expected_delta_time / f64::max(1.0, note_object.delta_time);

        // * High density is penalised at high velocity as it is generally considered easier to read.
        // * See https://www.desmos.com/calculator/u63f3ntdsi
        let density_penalty = logistic(object_density, 0.925, 15.0, None);

        let high_velocity_diff = (1.0 - 0.33 * density_penalty)
            * logistic(
                effective_bpm,
                high_velocity.center() + 8.0 * density_penalty,
                (1.0 + 0.5 * density_penalty) / (high_velocity.range() / 10.0),
                None,
            );

        mid_velocity_diff + high_velocity_diff
    }
}

struct VelocityRange {
    min: f64,
    max: f64,
}

impl VelocityRange {
    const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    const fn center(&self) -> f64 {
        (self.max + self.min) / 2.0
    }

    const fn range(&self) -> f64 {
        self.max - self.min
    }
}
