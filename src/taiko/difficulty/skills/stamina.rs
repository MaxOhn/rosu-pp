use crate::{
    any::difficulty::{object::IDifficultyObject, skills::strain_decay},
    taiko::difficulty::{
        evaluators::StaminaEvaluator,
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    },
    util::{
        difficulty::{logistic_exp, reverse_lerp},
        sync::Weak,
    },
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
        let mut stamina_difficulty =
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

        let mono_length_bonus = if self.is_convert {
            1.0
        } else {
            1.0 + 0.5 * reverse_lerp(index as f64, 5.0, 20.0)
        };

        // * Mono-streak bonus is only applied to colour-based stamina to reward longer sequences of same-colour hits within patterns.
        if !self.single_color {
            stamina_difficulty *= mono_length_bonus;
        }

        self.current_strain += stamina_difficulty;

        // * For converted maps, difficulty often comes entirely from long mono streams with no colour variation.
        // * To avoid over-rewarding these maps based purely on stamina strain, we dampen the strain value once the index exceeds 10.
        if self.single_color {
            logistic_exp(-(index - 10) as f64 / 2.0, Some(self.current_strain))
        } else {
            self.current_strain
        }
    }
}
