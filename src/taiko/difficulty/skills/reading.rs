use crate::{
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    util::{difficulty::logistic, sync::Weak},
};

define_skill! {
    #[derive(Clone)]
    pub struct Reading: StrainDecaySkill => TaikoDifficultyObjects[TaikoDifficultyObject] {
        current_strain: f64 = 0.0,
    }
}

impl Reading {
    const SKILL_MULTIPLIER: f64 = 1.0;
    const STRAIN_DECAY_BASE: f64 = 0.4;

    fn strain_value_of(&mut self, curr: &TaikoDifficultyObject, _: &TaikoDifficultyObjects) -> f64 {
        // * Drum Rolls and Swells are exempt.
        if !curr.base_hit_type.is_hit() {
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

        self.current_strain *= logistic(index as f64, 4.0, -1.0 / 25.0, Some(0.5)) + 0.5;
        self.current_strain *= Self::STRAIN_DECAY_BASE;
        self.current_strain += ReadingEvaluator::evaluate_diff_of(curr) * Self::SKILL_MULTIPLIER;

        self.current_strain
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
