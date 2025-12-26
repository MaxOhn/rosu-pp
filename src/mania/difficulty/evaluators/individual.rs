use crate::{mania::difficulty::object::ManiaDifficultyObject, util::sync::RefCount};

pub struct IndividualStrainEvaluator;

impl IndividualStrainEvaluator {
    pub fn evaluate_diff_of(curr: &ManiaDifficultyObject) -> f64 {
        let mania_curr = curr;
        let start_time = curr.start_time;
        let end_time = curr.end_time;

        // * We award a bonus if this note starts and ends before the end of another hold note.
        let with_bonus = mania_curr
            .prev_hit_objects
            .iter()
            .flatten()
            .map(RefCount::get)
            .any(|mania_prev| {
                mania_prev.end_time > end_time + 1.0 && start_time > mania_prev.start_time + 1.0
            });

        // * Factor to all additional strains in case something else is held
        let hold_factor = if with_bonus { 1.25 } else { 1.0 };

        2.0 * hold_factor
    }
}
