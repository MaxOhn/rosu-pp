use crate::{
    GameMods,
    taiko::{TaikoDifficultyAttributes, TaikoPerformanceAttributes, TaikoScoreState},
    util::special_functions::{erf, erf_inv},
};

pub(super) struct TaikoPerformanceCalculator<'mods> {
    attrs: TaikoDifficultyAttributes,
    mods: &'mods GameMods,
    state: TaikoScoreState,
}

impl<'a> TaikoPerformanceCalculator<'a> {
    pub const fn new(
        attrs: TaikoDifficultyAttributes,
        mods: &'a GameMods,
        state: TaikoScoreState,
    ) -> Self {
        Self { attrs, mods, state }
    }
}

impl TaikoPerformanceCalculator<'_> {
    pub fn calculate(self) -> TaikoPerformanceAttributes {
        // * The effectiveMissCount is calculated by gaining a ratio for totalSuccessfulHits
        // * and increasing the miss penalty for shorter object counts lower than 1000.
        let total_successful_hits = self.total_successful_hits();

        let estimated_unstable_rate = self.compute_deviation_upper_bound().map(|v| v * 10.0);

        let effective_miss_count = if total_successful_hits > 0 {
            (1000.0 / f64::from(total_successful_hits)).max(1.0) * f64::from(self.state.misses)
        } else {
            0.0
        };

        let mut multiplier = 1.13;

        if self.mods.hd() && !self.attrs.is_convert {
            multiplier *= 1.075;
        }

        if self.mods.ez() {
            multiplier *= 0.95;
        }

        let diff_value =
            self.compute_difficulty_value(effective_miss_count, estimated_unstable_rate);
        let acc_value = self.compute_accuracy_value(estimated_unstable_rate);

        let pp = (diff_value.powf(1.1) + acc_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

        TaikoPerformanceAttributes {
            difficulty: self.attrs,
            pp,
            pp_acc: acc_value,
            pp_difficulty: diff_value,
            effective_miss_count,
            estimated_unstable_rate,
        }
    }

    fn compute_difficulty_value(
        &self,
        effective_miss_count: f64,
        estimated_unstable_rate: Option<f64>,
    ) -> f64 {
        let Some(estimated_unstable_rate) = estimated_unstable_rate else {
            return 0.0;
        };

        let attrs = &self.attrs;
        let base_difficulty = 5.0 * f64::max(1.0, attrs.stars / 0.110) - 4.0;

        let mut difficulty_value = f64::min(
            f64::powf(base_difficulty, 3.0) / 69052.51,
            f64::powf(base_difficulty, 2.25) / 1250.0,
        );

        difficulty_value *= 1.0 + 0.10 * f64::max(0.0, self.attrs.stars - 10.0);

        let length_bonus = 1.0 + 0.1 * f64::min(1.0, f64::from(attrs.max_combo) / 1500.0);
        difficulty_value *= length_bonus;

        difficulty_value *= f64::powf(0.986, effective_miss_count);

        if self.mods.ez() {
            difficulty_value *= 0.9;
        }

        if self.mods.hd() {
            difficulty_value *= 1.025;
        }

        if self.mods.fl() {
            difficulty_value *= f64::max(
                1.0,
                1.05 - f64::min(self.attrs.mono_stamina_factor / 50.0, 1.0) * length_bonus,
            );
        }

        // * Scale accuracy more harshly on nearly-completely mono (single coloured) speed maps.
        let acc_scaling_exp = f64::from(2) + self.attrs.mono_stamina_factor;
        let acc_scaling_shift =
            f64::from(500) - f64::from(100) * (self.attrs.mono_stamina_factor * f64::from(3));

        difficulty_value
            * (erf(acc_scaling_shift / (f64::sqrt(2.0) * estimated_unstable_rate)))
                .powf(acc_scaling_exp)
    }

    fn compute_accuracy_value(&self, estimated_unstable_rate: Option<f64>) -> f64 {
        if self.attrs.great_hit_window <= 0.0 {
            return 0.0;
        }

        let Some(estimated_unstable_rate) = estimated_unstable_rate else {
            return 0.0;
        };

        let mut acc_value = f64::powf(70.0 / estimated_unstable_rate, 1.1)
            * f64::powf(self.attrs.stars, 0.4)
            * 100.0;

        let length_bonus = f64::min(1.15, f64::powf(self.total_hits() / 1500.0, 0.3));

        // * Slight HDFL Bonus for accuracy. A clamp is used to prevent against negative values.
        if self.mods.hd() && self.mods.fl() && !self.attrs.is_convert {
            acc_value *= f64::max(1.0, 1.05 * length_bonus);
        }

        acc_value
    }

    // * Computes an upper bound on the player's tap deviation based on the OD, number of circles and sliders,
    // * and the hit judgements, assuming the player's mean hit error is 0. The estimation is consistent in that
    // * two SS scores on the same map with the same settings will always return the same deviation.
    fn compute_deviation_upper_bound(&self) -> Option<f64> {
        if self.state.n300 == 0 || self.attrs.great_hit_window <= 0.0 {
            return None;
        }

        #[allow(clippy::items_after_statements, clippy::unreadable_literal)]
        // * 99% critical value for the normal distribution (one-tailed).
        const Z: f64 = 2.32634787404;

        let n = self.total_hits();

        // * Proportion of greats hit.
        let p = f64::from(self.state.n300) / n;

        // * We can be 99% confident that p is at least this value.
        let p_lower_bound = (n * p + Z * Z / 2.0) / (n + Z * Z)
            - Z / (n + Z * Z) * f64::sqrt(n * p * (1.0 - p) + Z * Z / 4.0);

        // * We can be 99% confident that the deviation is not higher than:
        Some(self.attrs.great_hit_window / (f64::sqrt(2.0) * erf_inv(p_lower_bound)))
    }

    const fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    const fn total_successful_hits(&self) -> u32 {
        self.state.n300 + self.state.n100
    }
}
