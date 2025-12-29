use crate::{
    any::difficulty::{object::IDifficultyObject, skills::strain_decay},
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    util::{difficulty::logistic_exp, sync::Weak},
};

define_skill! {
    #[derive(Clone)]
    pub struct Stamina: StrainSkill => TaikoDifficultyObjects[TaikoDifficultyObject] {
        single_color: bool,
        is_convert: bool,
        current_strain: f64 = 0.0,
    }
}

impl Stamina {
    const SKILL_MULTIPLIER: f64 = 1.1;
    const STRAIN_DECAY_BASE: f64 = 0.4;

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        if self.single_color {
            return 0.0;
        }

        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, |prev| prev.get().start_time);

        self.current_strain * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        self.current_strain *= strain_decay(curr.delta_time, Self::STRAIN_DECAY_BASE);
        self.current_strain +=
            StaminaEvaluator::evaluate_diff_of(curr, objects) * Self::SKILL_MULTIPLIER;

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

        if self.single_color {
            logistic_exp(-(index - 10) as f64 / 2.0, Some(self.current_strain))
        } else if self.is_convert {
            self.current_strain
        } else {
            #[expect(clippy::manual_clamp, reason = "staying in-sync with lazer")]
            let monolength_bonus = 1.0 + f64::min(f64::max((index - 5) as f64 / 50.0, 0.0), 0.30);

            self.current_strain * monolength_bonus
        }
    }
}

pub(super) struct StaminaEvaluator;

impl StaminaEvaluator {
    pub(super) fn evaluate_diff_of(
        curr: &TaikoDifficultyObject,
        objects: &TaikoDifficultyObjects,
    ) -> f64 {
        if !curr.base_hit_type.is_hit() {
            return 0.0;
        }

        // * Find the previous hit object hit by the current finger, which is n notes prior, n being the number of
        // * available fingers.
        let prev = curr.previous(1, objects);
        let prev_mono = objects.previous_mono(curr, Self::available_fingers_for(curr, objects) - 1);

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
