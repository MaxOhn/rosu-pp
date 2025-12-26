use crate::{mania::difficulty::object::ManiaDifficultyObject, util::difficulty::logistic};

pub struct OverallStrainEvaluator;

impl OverallStrainEvaluator {
    const RELEASE_THRESHOLD: f64 = 30.0;

    pub fn evaluate_diff_of(curr: &ManiaDifficultyObject) -> f64 {
        let mania_curr = curr;
        let start_time = curr.start_time;
        let end_time = curr.end_time;
        let mut is_overlapping = false;

        // * Lowest value we can assume with the current information
        let mut closest_end_time = (end_time - start_time).abs();
        // * Factor to all additional strains in case something else is held
        let mut hold_factor = 1.0;
        // * Addition to the current note in case it's a hold and has to be released awkwardly
        let mut hold_addition = 0.0;

        for mania_prev in mania_curr.prev_hit_objects.iter().flatten() {
            let mania_prev_ref = mania_prev.get();

            // * The current note is overlapped if a previous note or end is
            // * overlapping the current note body
            is_overlapping |= mania_prev_ref.end_time > start_time + 1.0
                && end_time > mania_prev_ref.end_time + 1.0
                && start_time > mania_prev_ref.start_time + 1.0;

            // * We give a slight bonus to everything if something is held meanwhile
            if mania_prev_ref.end_time > end_time + 1.0
                && start_time > mania_prev_ref.start_time + 1.0
            {
                hold_factor = 1.25;
            }

            closest_end_time = closest_end_time.min((end_time - mania_prev_ref.end_time).abs());
        }

        // * The hold addition is given if there was an overlap, however it is only valid if there are no other note with a similar ending.
        // * Releasing multiple notes is just as easy as releasing 1. Nerfs the hold addition by half if the closest release is release_threshold away.
        // * holdAddition
        // *     ^
        // * 1.0 + - - - - - -+-----------
        // *     |           /
        // * 0.5 + - - - - -/   Sigmoid Curve
        // *     |         /|
        // * 0.0 +--------+-+---------------> Release Difference / ms
        // *         release_threshold
        if is_overlapping {
            hold_addition = logistic(closest_end_time, Self::RELEASE_THRESHOLD, 0.27, None);
        }

        (1.0 + hold_addition) * hold_factor
    }
}
