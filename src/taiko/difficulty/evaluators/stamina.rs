use crate::{
    any::difficulty::object::IDifficultyObject,
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
};

pub struct StaminaEvaluator;

impl StaminaEvaluator {
    pub fn evaluate_diff_of(curr: &TaikoDifficultyObject, objects: &TaikoDifficultyObjects) -> f64 {
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
