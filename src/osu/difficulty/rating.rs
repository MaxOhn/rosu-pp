use std::convert::identity;

use crate::{
    GameMods,
    util::{difficulty::reverse_lerp, float_ext::FloatExt},
};

pub struct OsuRatingCalculator<'mods> {
    mods: &'mods GameMods,
    total_hits: u32,
    approach_rate: f64,
    overall_difficulty: f64,
    mechanical_difficulty_rating: f64,
    slider_factor: f64,
}

const DIFFICULTY_MULTIPLIER: f64 = 0.0675;

impl<'mods> OsuRatingCalculator<'mods> {
    pub const fn new(
        mods: &'mods GameMods,
        total_hits: u32,
        approach_rate: f64,
        overall_difficulty: f64,
        mechanical_difficulty_rating: f64,
        slider_factor: f64,
    ) -> Self {
        Self {
            mods,
            total_hits,
            approach_rate,
            overall_difficulty,
            mechanical_difficulty_rating,
            slider_factor,
        }
    }
}

impl OsuRatingCalculator<'_> {
    pub fn compute_aim_rating(&self, aim_difficulty_value: f64) -> f64 {
        if self.mods.ap() {
            return 0.0;
        }

        let mut aim_rating = Self::calculate_difficulty_rating(aim_difficulty_value);

        if self.mods.td() {
            aim_rating = aim_rating.powf(0.8);
        }

        if self.mods.rx() {
            aim_rating *= 0.9;
        }

        if let Some(magnetised_strength) = self.mods.attraction_strength() {
            aim_rating *= 1.0 - magnetised_strength;
        }

        let mut rating_multiplier = 1.0;

        let ar_length_bonus = 0.95
            + 0.4 * (f64::from(self.total_hits) / 2000.0).min(1.0)
            + f64::from(u8::from(self.total_hits > 2000))
                * (f64::from(self.total_hits) / 2000.0).log10()
                * 0.5;

        let ar_factor = if self.mods.rx() {
            0.0
        } else if self.approach_rate > 10.33 {
            0.3 * (self.approach_rate - 10.33)
        } else if self.approach_rate < 8.0 {
            0.05 * (8.0 - self.approach_rate)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        rating_multiplier += ar_factor * ar_length_bonus;

        if self.mods.hd() {
            let visibility_factor = Self::calculate_aim_visibility_factor(
                self.mechanical_difficulty_rating,
                self.approach_rate,
            );

            rating_multiplier += Self::calculate_visibility_bonus(
                self.mods,
                self.approach_rate,
                Some(visibility_factor),
                Some(self.slider_factor),
            );
        }

        // * It is important to consider accuracy difficulty when scaling with accuracy.
        rating_multiplier *= 0.98 + self.overall_difficulty.max(0.0).powf(2.0) / 2500.0;

        aim_rating * rating_multiplier.cbrt()
    }

    pub fn compute_speed_rating(&self, speed_difficulty_value: f64) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        let mut speed_rating = Self::calculate_difficulty_rating(speed_difficulty_value);

        if self.mods.ap() {
            speed_rating *= 0.5;
        }

        if let Some(magnetised_strength) = self.mods.attraction_strength() {
            // * reduce speed rating because of the speed distance scaling, with maximum reduction being 0.7x
            speed_rating *= 1.0 - magnetised_strength * 0.3;
        }

        let mut rating_multiplier = 1.0;

        let ar_length_bonus = 0.95
            + 0.4 * (f64::from(self.total_hits) / 2000.0).min(1.0)
            + f64::from(u8::from(self.total_hits > 2000))
                * (f64::from(self.total_hits) / 2000.0).log10()
                * 0.5;

        let ar_factor = if self.mods.ap() {
            0.0
        } else if self.approach_rate > 10.33 {
            0.3 * (self.approach_rate - 10.33)
        } else if self.approach_rate < 8.0 {
            0.05 * (8.0 - self.approach_rate)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        rating_multiplier += ar_factor * ar_length_bonus;

        if self.mods.hd() {
            let visibility_factor = Self::calculate_speed_visibility_factor(
                self.mechanical_difficulty_rating,
                self.approach_rate,
            );

            rating_multiplier += Self::calculate_visibility_bonus(
                self.mods,
                self.approach_rate,
                Some(visibility_factor),
                None,
            );
        }

        rating_multiplier *= 0.95 + self.overall_difficulty.max(0.0).powf(2.0) / 750.0;

        speed_rating * rating_multiplier.cbrt()
    }

    pub fn compute_flashlight_rating(&self, flashlight_difficulty_value: f64) -> f64 {
        if !self.mods.fl() {
            return 0.0;
        }

        let mut flashlight_rating = Self::calculate_difficulty_rating(flashlight_difficulty_value);

        if self.mods.td() {
            flashlight_rating = flashlight_rating.powf(0.8);
        }

        if self.mods.rx() {
            flashlight_rating *= 0.7;
        } else if self.mods.ap() {
            flashlight_rating *= 0.4;
        }

        if let Some(magnetised_strength) = self.mods.attraction_strength() {
            flashlight_rating *= 1.0 - magnetised_strength;
        }

        if let Some(deflate_initial_scale) = self.mods.deflate_start_scale() {
            flashlight_rating *= reverse_lerp(deflate_initial_scale, 11.0, 1.0).clamp(0.1, 1.0);
        }

        let mut rating_multiplier = 1.0;

        // * Account for shorter maps having a higher ratio of 0 combo/100 combo flashlight radius.
        rating_multiplier *= 0.7
            + 0.1 * (f64::from(self.total_hits) / 200.0).min(1.0)
            + f64::from(u8::from(self.total_hits > 200))
                * 0.2
                * (f64::from(self.total_hits - 200) / 200.0).min(1.0);

        // * It is important to consider accuracy difficulty when scaling with accuracy.
        rating_multiplier *= 0.98 + self.overall_difficulty.max(0.0).powf(2.0) / 2500.0;

        flashlight_rating * rating_multiplier.sqrt()
    }

    pub fn calculate_visibility_bonus(
        mods: &GameMods,
        approach_rate: f64,
        visibility_factor: Option<f64>,
        slider_factor: Option<f64>,
    ) -> f64 {
        // * NOTE: TC's effect is only noticeable in performance calculations until lazer mods are accounted for server-side.
        let is_always_partially_visible =
            mods.hd_only_fade_approach_circles().is_some_and(identity) || mods.tc();

        // * Start from normal curve, rewarding lower AR up to AR7
        let mut reading_bonus = 0.04 * (12.0 - approach_rate.max(7.0));

        reading_bonus *= visibility_factor.unwrap_or(1.0);

        // * We want to reward slideraim on low AR less
        let slider_visibility_factor = slider_factor.unwrap_or(1.0).powf(3.0);

        // * For AR up to 0 - reduce reward for very low ARs when object is visible
        if approach_rate < 7.0 {
            let factor = if is_always_partially_visible {
                0.03
            } else {
                0.045
            };

            reading_bonus += factor * (7.0 - approach_rate.max(0.0)) * slider_visibility_factor;
        }

        // * Starting from AR0 - cap values so they won't grow to infinity
        if approach_rate < 0.0 {
            let factor = if is_always_partially_visible {
                0.075
            } else {
                0.1
            };

            reading_bonus +=
                factor * (1.0 - 1.5_f64.powf(approach_rate)) * slider_visibility_factor;
        }

        reading_bonus
    }

    pub fn calculate_difficulty_rating(difficulty_value: f64) -> f64 {
        difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER
    }

    fn calculate_aim_visibility_factor(
        mechanical_difficulty_rating: f64,
        approach_rate: f64,
    ) -> f64 {
        const AR_FACTOR_END_POINT: f64 = 11.5;

        let mechanical_difficulty_factor = reverse_lerp(mechanical_difficulty_rating, 5.0, 10.0);
        let ar_factor_starting_point = FloatExt::lerp(9.0, 10.33, mechanical_difficulty_factor);

        reverse_lerp(approach_rate, AR_FACTOR_END_POINT, ar_factor_starting_point)
    }

    fn calculate_speed_visibility_factor(
        mechanical_difficulty_rating: f64,
        approach_rate: f64,
    ) -> f64 {
        const AR_FACTOR_END_POINT: f64 = 11.5;

        let mechanical_difficulty_factor = reverse_lerp(mechanical_difficulty_rating, 5.0, 10.0);
        let ar_factor_starting_point = FloatExt::lerp(10.0, 10.33, mechanical_difficulty_factor);

        reverse_lerp(approach_rate, AR_FACTOR_END_POINT, ar_factor_starting_point)
    }
}
