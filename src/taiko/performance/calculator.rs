use crate::{
    GameMods,
    taiko::{TaikoDifficultyAttributes, TaikoPerformanceAttributes, TaikoScoreState},
    util::{
        difficulty::{erf, erf_inv, logistic, reverse_lerp},
        float_ext::FloatExt,
    },
};

pub(super) struct TaikoPerformanceCalculator<'mods> {
    attrs: TaikoDifficultyAttributes,
    mods: &'mods GameMods,
    state: TaikoScoreState,
    is_classic: bool,
}

impl<'a> TaikoPerformanceCalculator<'a> {
    pub const fn new(
        attrs: TaikoDifficultyAttributes,
        mods: &'a GameMods,
        state: TaikoScoreState,
        is_classic: bool,
    ) -> Self {
        Self {
            attrs,
            mods,
            state,
            is_classic,
        }
    }
}

impl TaikoPerformanceCalculator<'_> {
    pub fn calculate(self) -> TaikoPerformanceAttributes {
        let estimated_unstable_rate =
            if self.state.hitresults.n300 == 0 || self.attrs.great_hit_window <= 0.0 {
                None
            } else {
                Some(
                    self.compute_deviation_upper_bound(
                        f64::from(self.state.hitresults.n300) / self.total_hits(),
                    ) * 10.0,
                )
            };

        // * Total difficult hits measures the total difficulty of a map based on its consistency factor.
        let total_difficult_hits = self.total_hits() * self.attrs.consistency_factor;

        let difficulty_value =
            self.compute_difficulty_value(total_difficult_hits, estimated_unstable_rate) * 1.08;
        let accuracy_value =
            self.compute_accuracy_value(total_difficult_hits, estimated_unstable_rate) * 1.1;

        let pp = difficulty_value + accuracy_value;

        TaikoPerformanceAttributes {
            difficulty: self.attrs,
            pp,
            pp_acc: accuracy_value,
            pp_difficulty: difficulty_value,
            estimated_unstable_rate,
        }
    }

    fn compute_difficulty_value(
        &self,
        total_difficult_hits: f64,
        estimated_unstable_rate: Option<f64>,
    ) -> f64 {
        let Some(estimated_unstable_rate) = estimated_unstable_rate else {
            return 0.0;
        };

        if FloatExt::eq(total_difficult_hits, 0.0) {
            return 0.0;
        }

        let attrs = &self.attrs;

        // * The estimated unstable rate for 100% accuracy, at which all rhythm difficulty has been played successfully.
        let rhythm_expected_unstable_rate = self.compute_deviation_upper_bound(1.0) * 10.0;

        // * The unstable rate at which it can be assumed all rhythm difficulty has been ignored.
        // * 0.8 represents 80% of total hits being greats, or 90% accuracy in-game
        let rhythm_maximum_unstable_rate = self.compute_deviation_upper_bound(0.8) * 10.0;

        // * The fraction of star rating made up by rhythm difficulty, normalised to represent rhythm's perceived contribution to star rating.
        let rhythm_factor = reverse_lerp(attrs.rhythm / attrs.stars, 0.15, 0.4);

        // * A penalty removing improperly played rhythm difficulty from star rating based on estimated unstable rate.
        let rhythm_penalty = 1.0
            - logistic(
                estimated_unstable_rate,
                (rhythm_expected_unstable_rate + rhythm_maximum_unstable_rate) / 2.0,
                10.0 / (rhythm_maximum_unstable_rate - rhythm_expected_unstable_rate),
                Some(0.25 * f64::powf(rhythm_factor, 3.0)),
            );

        let base_difficulty = 5.0 * f64::max(1.0, attrs.stars * rhythm_penalty / 0.11) - 4.0;

        let mut difficulty_value = f64::min(
            f64::powf(base_difficulty, 3.0) / 69052.51,
            f64::powf(base_difficulty, 2.25) / 1250.0,
        );

        difficulty_value *= 1.0 + 0.10 * f64::max(0.0, self.attrs.stars - 10.0);

        // * Applies a bonus to maps with more total difficulty.
        let length_bonus = 1.0 + 0.25 * total_difficult_hits / (total_difficult_hits + 4000.0);
        difficulty_value *= length_bonus;

        // * Scales miss penalty by the total difficult hits of a map, making misses more punishing on maps with less total difficulty.
        let miss_penalty = 0.97 + 0.03 * total_difficult_hits / (total_difficult_hits + 1500.0);
        difficulty_value *= f64::powf(miss_penalty, f64::from(self.state.hitresults.misses));

        if self.mods.hd() {
            let mut hidden_bonus = if self.attrs.is_convert { 0.025 } else { 0.1 };

            // * Hidden+flashlight plays are excluded from reading-based penalties to hidden.
            if !self.mods.fl() {
                // * A penalty is applied to the bonus for hidden on non-classic scores, as the playfield can be made wider to make fast reading easier.
                if !self.is_classic {
                    hidden_bonus *= 0.2;
                }

                // * A penalty is applied to classic easy+hidden scores, as notes disappear later making fast reading easier.
                if self.mods.ez() && self.is_classic {
                    hidden_bonus *= 0.5;
                }
            }

            difficulty_value *= 1.0 + hidden_bonus;
        }

        if self.mods.fl() {
            difficulty_value *= f64::max(
                1.0,
                1.05 - f64::min(self.attrs.mono_stamina_factor / 50.0, 1.0) * length_bonus,
            );
        }

        // * Scale accuracy more harshly on nearly-completely mono (single coloured) speed maps.
        let mono_acc_scaling_exponent = f64::from(2) + self.attrs.mono_stamina_factor;
        let mono_acc_scaling_shift =
            f64::from(500) - f64::from(100) * (self.attrs.mono_stamina_factor * f64::from(3));

        difficulty_value
            * (erf(mono_acc_scaling_shift / (f64::sqrt(2.0) * estimated_unstable_rate)))
                .powf(mono_acc_scaling_exponent)
    }

    fn compute_accuracy_value(
        &self,
        total_difficult_hits: f64,
        estimated_unstable_rate: Option<f64>,
    ) -> f64 {
        let Some(estimated_unstable_rate) = estimated_unstable_rate else {
            return 0.0;
        };

        if self.attrs.great_hit_window <= 0.0 {
            return 0.0;
        }

        let mut accuracy_value = 470.0 * f64::powf(0.9885, estimated_unstable_rate);

        // * Scales up the bonus for lower unstable rate as star rating increases.
        accuracy_value *= 1.0
            + f64::powf(50.0 / estimated_unstable_rate, 2.0) * f64::powf(self.attrs.stars, 2.8)
                / 600.0;

        if self.mods.hd() && !self.attrs.is_convert {
            accuracy_value *= 1.075;
        }

        // * Applies a bonus to maps with more total difficulty, calculating this with a map's total hits and consistency factor.
        accuracy_value *= 1.0 + 0.3 * total_difficult_hits / (total_difficult_hits + 4000.0);

        // * Applies a bonus to maps with more total memory required with HDFL.
        let memory_length_bonus = f64::min(1.15, f64::powf(self.total_hits() / 1500.0, 0.3));

        if self.mods.fl() && self.mods.hd() && !self.attrs.is_convert {
            accuracy_value *= f64::max(1.0, 1.05 * memory_length_bonus);
        }

        accuracy_value
    }

    // * Computes an upper bound on the player's tap deviation based on the OD, number of circles and sliders,
    // * and the hit judgements, assuming the player's mean hit error is 0. The estimation is consistent in that
    // * two SS scores on the same map with the same settings will always return the same deviation.
    fn compute_deviation_upper_bound(&self, accuracy: f64) -> f64 {
        #[expect(clippy::unreadable_literal, reason = "staying in-sync with lazer")]
        // * 99% critical value for the normal distribution (one-tailed).
        const Z: f64 = 2.32634787404;

        let n = self.total_hits();

        // * Proportion of greats hit.
        let p = accuracy;

        // * We can be 99% confident that p is at least this value.
        let p_lower_bound = (n * p + Z * Z / 2.0) / (n + Z * Z)
            - Z / (n + Z * Z) * f64::sqrt(n * p * (1.0 - p) + Z * Z / 4.0);

        // * We can be 99% confident that the deviation is not higher than:
        self.attrs.great_hit_window / (f64::sqrt(2.0) * erf_inv(p_lower_bound))
    }

    const fn total_hits(&self) -> f64 {
        self.state.hitresults.total_hits() as f64
    }
}
