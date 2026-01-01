use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::difficulty::{bpm_to_milliseconds, milliseconds_to_bpm},
};

pub struct SpeedEvaluator;

impl SpeedEvaluator {
    const SINGLE_SPACING_THRESHOLD: f64 = OsuDifficultyObject::NORMALIZED_DIAMETER as f64 * 1.25; // 1.25 circlers distance between centers
    const MIN_SPEED_BONUS: f64 = 200.0; // 200 BPM 1/4th
    const SPEED_BALANCING_FACTOR: f64 = 40.0;
    const DIST_MULTIPLIER: f64 = 0.8;

    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        hit_window: f64,
        autopilot: bool,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        // * derive strainTime for calculation
        let osu_curr_obj = curr;
        let osu_prev_obj = curr.previous(0, diff_objects);
        let osu_next_obj = curr.next(0, diff_objects);

        let mut strain_time = curr.adjusted_delta_time;
        // Note: Technically `osu_next_obj` is never `None` but instead the
        // default value. This could maybe invalidate the `get_doubletapness`
        // result.
        let doubletapness = 1.0 - osu_curr_obj.get_doubletapness(osu_next_obj, hit_window);

        // * Cap deltatime to the OD 300 hitwindow.
        // * 0.93 is derived from making sure 260bpm OD8 streams aren't nerfed harshly, whilst 0.92 limits the effect of the cap.
        strain_time /= ((strain_time / hit_window) / 0.93).clamp(0.92, 1.0);

        let speed_bonus = if milliseconds_to_bpm(strain_time, None) > Self::MIN_SPEED_BONUS {
            // * Add additional scaling bonus for streams/bursts higher than 200bpm
            let base = (bpm_to_milliseconds(Self::MIN_SPEED_BONUS, None) - strain_time)
                / Self::SPEED_BALANCING_FACTOR;

            0.75 * base.powf(2.0)
        } else {
            // * speedBonus will be 0.0 for BPM < 200
            0.0
        };

        let travel_dist = osu_prev_obj.map_or(0.0, |obj| obj.travel_dist);
        let mut dist = travel_dist + osu_curr_obj.min_jump_dist;

        // * Cap distance at single_spacing_threshold
        dist = Self::SINGLE_SPACING_THRESHOLD.min(dist);

        // * Max distance bonus is 1 * `distance_multiplier` at single_spacing_threshold
        let mut dist_bonus =
            (dist / Self::SINGLE_SPACING_THRESHOLD).powf(3.95) * Self::DIST_MULTIPLIER;

        dist_bonus *= osu_curr_obj.small_circle_bonus.sqrt();

        if autopilot {
            dist_bonus = 0.0;
        }

        // * Base difficulty with all bonuses
        let difficulty = (1.0 + speed_bonus + dist_bonus) * 1000.0 / strain_time;

        // * Apply penalty if there's doubletappable doubles
        difficulty * doubletapness
    }
}
