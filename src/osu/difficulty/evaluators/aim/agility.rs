use crate::{
    any::difficulty::object::IDifficultyObject, osu::difficulty::object::OsuDifficultyObject,
};

pub struct AgilityEvaluator;

impl AgilityEvaluator {
    // * 1.2 circles distance between centers
    const DISTANCE_CAP: f64 = (OsuDifficultyObject::NORMALIZED_DIAMETER as f64) * 1.2;

    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let osu_curr_obj = curr;
        let osu_prev_obj = curr.previous(0, diff_objects);

        let travel_dist = osu_prev_obj.map_or(0.0, |obj| obj.lazy_travel_dist);
        let dist = travel_dist + osu_curr_obj.lazy_jump_dist;

        let dist_scaled = dist.min(Self::DISTANCE_CAP) / Self::DISTANCE_CAP;

        let mut agility_diff = dist_scaled * 1000.0 / osu_curr_obj.adjusted_delta_time;

        agility_diff *= osu_curr_obj.small_circle_bonus.powf(1.5);

        agility_diff *= Self::high_bpm_bonus(osu_curr_obj.adjusted_delta_time);

        agility_diff
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - f64::powf(0.2, ms / 1000.0))
    }
}
