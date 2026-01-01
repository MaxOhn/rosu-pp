use crate::{
    any::difficulty::{object::IDifficultyObject, skills::strain_decay},
    taiko::difficulty::{
        evaluators::StaminaEvaluator,
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
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
