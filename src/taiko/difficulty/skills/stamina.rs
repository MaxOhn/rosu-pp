use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill, StrainDecaySkill},
    },
    taiko::{
        difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
        object::HitType,
    },
    util::strains_vec::StrainsVec,
};

const SKILL_MULTIPLIER: f64 = 1.1;
const STRAIN_DECAY_BASE: f64 = 0.4;

#[derive(Clone, Default)]
pub struct Stamina {
    inner: StrainDecaySkill,
}

impl Stamina {
    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks()
    }

    pub fn as_difficulty_value(&self) -> f64 {
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
        let prev_start_time = curr
            .previous(0, &self.diff_objects.objects)
            .map_or(0.0, |prev| prev.get().start_time);

        self.curr_strain() * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    const fn curr_strain(&self) -> f64 {
        self.inner.inner.curr_strain
    }

    fn curr_strain_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.curr_strain
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

    fn strain_value_at(&mut self, curr: &TaikoDifficultyObject) -> f64 {
        *self.curr_strain_mut() *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        *self.curr_strain_mut() += self.strain_value_of(curr) * SKILL_MULTIPLIER;

        self.curr_strain()
    }

    fn strain_value_of(&self, curr: &TaikoDifficultyObject) -> f64 {
        StaminaEvaluator::evaluate_diff_of(curr, self.diff_objects)
    }
}

struct StaminaEvaluator;

impl StaminaEvaluator {
    fn speed_bonus(mut interval: f64) -> f64 {
        // * Interval is capped at a very small value to prevent infinite values.
        interval = interval.max(1.0);

        30.0 / interval
    }

    fn available_fingers_for(
        hit_object: &TaikoDifficultyObject,
        hit_objects: &TaikoDifficultyObjects,
    ) -> usize {
        let prev_color_change = hit_object.color.previous_color_change(hit_objects);

        if prev_color_change
            .is_some_and(|change| hit_object.start_time - change.get().start_time < 300.0)
        {
            return 2;
        }

        let next_color_change = hit_object.color.next_color_change(hit_objects);

        if next_color_change
            .is_some_and(|change| change.get().start_time - hit_object.start_time < 300.0)
        {
            return 2;
        }

        4
    }

    fn evaluate_diff_of(curr: &TaikoDifficultyObject, hit_objects: &TaikoDifficultyObjects) -> f64 {
        if matches!(curr.base_hit_type, HitType::NonHit) {
            return 0.0;
        }

        // * Find the previous hit object hit by the current finger, which is n notes prior, n being the number of
        // * available fingers.
        let taiko_curr = curr;
        let key_prev = hit_objects.previous_mono(
            taiko_curr,
            Self::available_fingers_for(taiko_curr, hit_objects) - 1,
        );

        if let Some(key_prev) = key_prev {
            // * Add a base strain to all objects
            0.5 + Self::speed_bonus(taiko_curr.start_time - key_prev.get().start_time)
        } else {
            // * There is no previous hit object hit by the current finger
            0.0
        }
    }
}
