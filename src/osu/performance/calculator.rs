use std::f64::consts::PI;

use crate::{
    GameMods,
    osu::{
        OsuDifficultyAttributes, OsuHitResults, OsuPerformanceAttributes, OsuScoreState,
        difficulty::skills::{
            aim::Aim, flashlight::Flashlight, speed::Speed, strain::OsuStrainSkill,
        },
    },
    util::{
        difficulty::reverse_lerp,
        float_ext::FloatExt,
        special_functions::{erf, erf_inv},
    },
};

use super::{n_large_tick_miss, n_slider_ends_dropped, total_imperfect_hits};

// * This is being adjusted to keep the final pp value scaled around what it used to be when changing things.
pub const PERFORMANCE_BASE_MULTIPLIER: f64 = 1.15;

pub(super) struct OsuPerformanceCalculator<'mods> {
    attrs: OsuDifficultyAttributes,
    mods: &'mods GameMods,
    acc: f64,
    state: OsuScoreState,
    effective_miss_count: f64,
    using_classic_slider_acc: bool,
}

impl<'a> OsuPerformanceCalculator<'a> {
    pub const fn new(
        attrs: OsuDifficultyAttributes,
        mods: &'a GameMods,
        acc: f64,
        state: OsuScoreState,
        effective_miss_count: f64,
        using_classic_slider_acc: bool,
    ) -> Self {
        Self {
            attrs,
            mods,
            acc,
            state,
            effective_miss_count,
            using_classic_slider_acc,
        }
    }
}

impl OsuPerformanceCalculator<'_> {
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        let total_hits = self.state.hitresults.total_hits();

        if total_hits == 0 {
            return OsuPerformanceAttributes {
                difficulty: self.attrs,
                ..Default::default()
            };
        }

        let total_hits = f64::from(total_hits);

        let mut multiplier = PERFORMANCE_BASE_MULTIPLIER;

        if self.mods.nf() {
            multiplier *= (1.0 - 0.02 * self.effective_miss_count).max(0.9);
        }

        if self.mods.so() && total_hits > 0.0 {
            multiplier *= 1.0 - (f64::from(self.attrs.n_spinners) / total_hits).powf(0.85);
        }

        if self.mods.rx() {
            let od = self.attrs.od();

            // * https://www.desmos.com/calculator/bc9eybdthb
            // * we use OD13.3 as maximum since it's the value at which great hitwidow becomes 0
            // * this is well beyond currently maximum achievable OD which is 12.17 (DTx2 + DA with OD11)
            let (n100_mult, n50_mult) = if od > 0.0 {
                (
                    (1.0 - (od / 13.33).powf(1.8)).max(0.0),
                    (1.0 - (od / 13.33).powf(5.0)).max(0.0),
                )
            } else {
                (1.0, 1.0)
            };

            // * As we're adding Oks and Mehs to an approximated number of combo breaks the result can be
            // * higher than total hits in specific scenarios (which breaks some calculations) so we need to clamp it.
            self.effective_miss_count = (self.effective_miss_count
                + f64::from(self.state.hitresults.n100) * n100_mult
                + f64::from(self.state.hitresults.n50) * n50_mult)
                .min(total_hits);
        }

        let speed_deviation = self.calculate_speed_deviation();

        let aim_value = self.compute_aim_value();
        let speed_value = self.compute_speed_value(speed_deviation);
        let acc_value = self.compute_accuracy_value();
        let flashlight_value = self.compute_flashlight_value();

        let pp = (aim_value.powf(1.1)
            + speed_value.powf(1.1)
            + acc_value.powf(1.1)
            + flashlight_value.powf(1.1))
        .powf(1.0 / 1.1)
            * multiplier;

        OsuPerformanceAttributes {
            difficulty: self.attrs,
            pp_acc: acc_value,
            pp_aim: aim_value,
            pp_flashlight: flashlight_value,
            pp_speed: speed_value,
            pp,
            effective_miss_count: self.effective_miss_count,
            speed_deviation,
        }
    }

    fn compute_aim_value(&self) -> f64 {
        if self.mods.ap() {
            return 0.0;
        }

        let mut aim_difficulty = self.attrs.aim;

        if self.attrs.n_sliders > 0 && self.attrs.aim_difficult_slider_count > 0.0 {
            let estimate_improperly_followed_difficult_sliders = if self.using_classic_slider_acc {
                // * When the score is considered classic (regardless if it was made on old client or not)
                // * we consider all missing combo to be dropped difficult sliders
                let maximum_possible_dropped_sliders = total_imperfect_hits(&self.state.hitresults);

                f64::clamp(
                    f64::min(
                        maximum_possible_dropped_sliders,
                        f64::from(self.attrs.max_combo - self.state.max_combo),
                    ),
                    0.0,
                    self.attrs.aim_difficult_slider_count,
                )
            } else {
                // * We add tick misses here since they too mean that the player didn't follow the slider properly
                // * We however aren't adding misses here because missing slider heads has a harsh penalty
                // * by itself and doesn't mean that the rest of the slider wasn't followed properly
                f64::clamp(
                    f64::from(
                        n_slider_ends_dropped(&self.attrs, &self.state)
                            + n_large_tick_miss(&self.attrs, &self.state),
                    ),
                    0.0,
                    self.attrs.aim_difficult_slider_count,
                )
            };

            let slider_nerf_factor = (1.0 - self.attrs.slider_factor)
                * f64::powf(
                    1.0 - estimate_improperly_followed_difficult_sliders
                        / self.attrs.aim_difficult_slider_count,
                    3.0,
                )
                + self.attrs.slider_factor;
            aim_difficulty *= slider_nerf_factor;
        }

        let mut aim_value = Aim::difficulty_to_performance(aim_difficulty);

        let total_hits = self.total_hits();

        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + f64::from(u8::from(total_hits > 2000.0)) * (total_hits / 2000.0).log10() * 0.5;

        aim_value *= len_bonus;

        if self.effective_miss_count > 0.0 {
            aim_value *= Self::calculate_miss_penalty(
                self.effective_miss_count,
                self.attrs.aim_difficult_strain_count,
            );
        }

        let ar_factor = if self.mods.rx() {
            0.0
        } else if self.attrs.ar > 10.33 {
            0.3 * (self.attrs.ar - 10.33)
        } else if self.attrs.ar < 8.0 {
            0.05 * (8.0 - self.attrs.ar)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        aim_value *= 1.0 + ar_factor * len_bonus;

        if self.mods.bl() {
            aim_value *= 1.3
                + (total_hits
                    * (0.0016 / (1.0 + 2.0 * self.effective_miss_count))
                    * self.acc.powf(16.0))
                    * (1.0 - 0.003 * self.attrs.hp * self.attrs.hp);
        } else if self.mods.hd() || self.mods.tc() {
            // * We want to give more reward for lower AR when it comes to aim and HD. This nerfs high AR and buffs lower AR.
            aim_value *= 1.0 + 0.04 * (12.0 - self.attrs.ar);
        }

        aim_value *= self.acc;
        // * It is important to consider accuracy difficulty when scaling with accuracy.
        aim_value *= 0.98 + f64::powf(f64::max(0.0, self.attrs.od()), 2.0) / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self, speed_deviation: Option<f64>) -> f64 {
        let Some(speed_deviation) = speed_deviation.filter(|_| !self.mods.rx()) else {
            return 0.0;
        };

        let mut speed_value = Speed::difficulty_to_performance(self.attrs.speed);

        let total_hits = self.total_hits();

        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + f64::from(u8::from(total_hits > 2000.0)) * (total_hits / 2000.0).log10() * 0.5;

        speed_value *= len_bonus;

        if self.effective_miss_count > 0.0 {
            speed_value *= Self::calculate_miss_penalty(
                self.effective_miss_count,
                self.attrs.speed_difficult_strain_count,
            );
        }

        let ar_factor = if self.mods.ap() {
            0.0
        } else if self.attrs.ar > 10.33 {
            0.3 * (self.attrs.ar - 10.33)
        } else {
            0.0
        };

        // * Buff for longer maps with high AR.
        speed_value *= 1.0 + ar_factor * len_bonus;

        if self.mods.bl() {
            // * Increasing the speed value by object count for Blinds isn't
            // * ideal, so the minimum buff is given.
            speed_value *= 1.12;
        } else if self.mods.hd() || self.mods.tc() {
            // * We want to give more reward for lower AR when it comes to aim and HD.
            // * This nerfs high AR and buffs lower AR.
            speed_value *= 1.0 + 0.04 * (12.0 - self.attrs.ar);
        }

        let speed_high_deviation_mult = self.calculate_speed_high_deviation_nerf(speed_deviation);
        speed_value *= speed_high_deviation_mult;

        // * Calculate accuracy assuming the worst case scenario
        let relevant_total_diff = f64::max(0.0, total_hits - self.attrs.speed_note_count);
        let hitresults = &self.state.hitresults;
        let relevant_n300 = (f64::from(hitresults.n300) - relevant_total_diff).max(0.0);
        let relevant_n100 = (f64::from(hitresults.n100)
            - (relevant_total_diff - f64::from(hitresults.n300)).max(0.0))
        .max(0.0);
        let relevant_n50 = (f64::from(hitresults.n50)
            - (relevant_total_diff - f64::from(hitresults.n300 + hitresults.n100)).max(0.0))
        .max(0.0);

        let relevant_acc = if self.attrs.speed_note_count.eq(0.0) {
            0.0
        } else {
            (relevant_n300 * 6.0 + relevant_n100 * 2.0 + relevant_n50)
                / (self.attrs.speed_note_count * 6.0)
        };

        let od = self.attrs.od();

        // * Scale the speed value with accuracy and OD.
        speed_value *= (0.95 + f64::powf(f64::max(0.0, od), 2.0) / 750.0)
            * f64::powf((self.acc + relevant_acc) / 2.0, (14.5 - od) / 2.0);

        speed_value
    }

    fn compute_accuracy_value(&self) -> f64 {
        if self.mods.rx() {
            return 0.0;
        }

        // * This percentage only considers HitCircles of any value - in this part
        // * of the calculation we focus on hitting the timing hit window.
        let mut amount_hit_objects_with_acc = self.attrs.n_circles;

        if !self.using_classic_slider_acc {
            amount_hit_objects_with_acc += self.attrs.n_sliders;
        }

        let hitresults = &self.state.hitresults;

        let mut better_acc_percentage = if amount_hit_objects_with_acc > 0 {
            f64::from(
                (hitresults.n300 as i32
                    - (i32::max(
                        hitresults.total_hits() as i32 - amount_hit_objects_with_acc as i32,
                        0,
                    )))
                    * 6
                    + hitresults.n100 as i32 * 2
                    + hitresults.n50 as i32,
            ) / f64::from(amount_hit_objects_with_acc * 6)
        } else {
            0.0
        };

        // * It is possible to reach a negative accuracy with this formula. Cap it at zero - zero points.
        if better_acc_percentage < 0.0 {
            better_acc_percentage = 0.0;
        }

        // * Lots of arbitrary values from testing.
        // * Considering to use derivation from perfect accuracy in a probabilistic manner - assume normal distribution.
        let mut acc_value =
            1.52163_f64.powf(self.attrs.od()) * better_acc_percentage.powf(24.0) * 2.83;

        // * Bonus for many hitcircles - it's harder to keep good accuracy up for longer.
        acc_value *= (f64::from(amount_hit_objects_with_acc) / 1000.0)
            .powf(0.3)
            .min(1.15);

        // * Increasing the accuracy value by object count for Blinds isn't
        // * ideal, so the minimum buff is given.
        if self.mods.bl() {
            acc_value *= 1.14;
        } else if self.mods.hd() || self.mods.tc() {
            acc_value *= 1.08;
        }

        if self.mods.fl() {
            acc_value *= 1.02;
        }

        acc_value
    }

    fn compute_flashlight_value(&self) -> f64 {
        if !self.mods.fl() {
            return 0.0;
        }

        let mut flashlight_value = Flashlight::difficulty_to_performance(self.attrs.flashlight);

        let total_hits = self.total_hits();

        // * Penalize misses by assessing # of misses relative to the total # of objects. Default a 3% reduction for any # of misses.
        if self.effective_miss_count > 0.0 {
            flashlight_value *= 0.97
                * (1.0 - (self.effective_miss_count / total_hits).powf(0.775))
                    .powf(self.effective_miss_count.powf(0.875));
        }

        flashlight_value *= self.get_combo_scaling_factor();

        // * Account for shorter maps having a higher ratio of 0 combo/100 combo flashlight radius.
        flashlight_value *= 0.7
            + 0.1 * (total_hits / 200.0).min(1.0)
            + f64::from(u8::from(total_hits > 200.0))
                * 0.2
                * ((total_hits - 200.0) / 200.0).min(1.0);

        // * Scale the flashlight value with accuracy _slightly_.
        flashlight_value *= 0.5 + self.acc / 2.0;
        // * It is important to also consider accuracy difficulty when doing that.
        flashlight_value *= 0.98 + f64::powf(f64::max(0.0, self.attrs.od()), 2.0) / 2500.0;

        flashlight_value
    }

    fn calculate_speed_deviation(&self) -> Option<f64> {
        if total_successful_hits(&self.state.hitresults) == 0 {
            return None;
        }

        let hitresults = &self.state.hitresults;

        // * Calculate accuracy assuming the worst case scenario
        let mut speed_note_count = self.attrs.speed_note_count;
        speed_note_count +=
            (f64::from(hitresults.total_hits()) - self.attrs.speed_note_count) * 0.1;

        // * Assume worst case: all mistakes were on speed notes
        let relevant_count_miss = f64::min(f64::from(hitresults.misses), speed_note_count);
        let relevant_count_meh = f64::min(
            f64::from(hitresults.n50),
            speed_note_count - relevant_count_miss,
        );
        let relevant_count_ok = f64::min(
            f64::from(hitresults.n100),
            speed_note_count - relevant_count_miss - relevant_count_meh,
        );
        let relevant_count_great = f64::max(
            0.0,
            speed_note_count - relevant_count_miss - relevant_count_meh - relevant_count_ok,
        );

        self.calculate_deviation(
            relevant_count_great,
            relevant_count_ok,
            relevant_count_meh,
            relevant_count_miss,
        )
    }

    fn calculate_deviation(
        &self,
        relevant_count_great: f64,
        relevant_count_ok: f64,
        relevant_count_meh: f64,
        relevant_count_miss: f64,
    ) -> Option<f64> {
        if relevant_count_great + relevant_count_ok + relevant_count_meh <= 0.0 {
            return None;
        }

        let object_count =
            relevant_count_great + relevant_count_ok + relevant_count_meh + relevant_count_miss;

        // * The probability that a player hits a circle is unknown, but we can estimate it to be
        // * the number of greats on circles divided by the number of circles, and then add one
        // * to the number of circles as a bias correction.

        let n = f64::max(1.0, object_count - relevant_count_miss - relevant_count_meh);

        #[allow(clippy::items_after_statements, clippy::unreadable_literal)]
        const Z: f64 = 2.32634787404; // * 99% critical value for the normal distribution (one-tailed).

        // * Proportion of greats hit on circles, ignoring misses and 50s.
        let p = relevant_count_great / n;

        // * We can be 99% confident that p is at least this value.
        let p_lower_bound = (n * p + Z * Z / 2.0) / (n + Z * Z)
            - Z / (n + Z * Z) * f64::sqrt(n * p * (1.0 - p) + Z * Z / 4.0);

        let great_hit_window: f64 = self.attrs.great_hit_window;
        let ok_hit_window: f64 = self.attrs.ok_hit_window;
        let meh_hit_window: f64 = self.attrs.meh_hit_window;

        // * Compute the deviation assuming greats and oks are normally distributed, and mehs are uniformly distributed.
        // * Begin with greats and oks first. Ignoring mehs, we can be 99% confident that the deviation is not higher than:
        let mut deviation = great_hit_window / (f64::sqrt(2.0) * erf_inv(p_lower_bound));

        let random_value = f64::sqrt(2.0 / PI)
            * ok_hit_window
            * f64::exp(-0.5 * f64::powf(ok_hit_window / deviation, 2.0))
            / (deviation * erf(ok_hit_window / (f64::sqrt(2.0) * deviation)));

        deviation *= f64::sqrt(1.0 - random_value);

        // * Value deviation approach as greatCount approaches 0
        let limit_value = ok_hit_window / f64::sqrt(3.0);

        // * If precision is not enough to compute true deviation - use limit value
        if p_lower_bound == 0.0 || random_value >= 1.0 || deviation > limit_value {
            deviation = limit_value;
        }

        // * Then compute the variance for mehs.
        let meh_variance = (meh_hit_window * meh_hit_window
            + ok_hit_window * meh_hit_window
            + ok_hit_window * ok_hit_window)
            / 3.0;

        // * Find the total deviation.
        let deviation = f64::sqrt(
            ((relevant_count_great + relevant_count_ok) * f64::powf(deviation, 2.0)
                + relevant_count_meh * meh_variance)
                / (relevant_count_great + relevant_count_ok + relevant_count_meh),
        );

        Some(deviation)
    }

    fn calculate_speed_high_deviation_nerf(&self, speed_deviation: f64) -> f64 {
        let speed_value = Speed::difficulty_to_performance(self.attrs.speed);

        // * Decides a point where the PP value achieved compared to the speed deviation is assumed to be tapped improperly. Any PP above this point is considered "excess" speed difficulty.
        // * This is used to cause PP above the cutoff to scale logarithmically towards the original speed value thus nerfing the value.
        let excess_speed_difficulty_cutoff = 100.0 + 220.0 * f64::powf(22.0 / speed_deviation, 6.5);

        if speed_value <= excess_speed_difficulty_cutoff {
            return 1.0;
        }

        #[allow(clippy::items_after_statements)]
        const SCALE: f64 = 50.0;

        let mut adjusted_speed_value = SCALE
            * (f64::ln((speed_value - excess_speed_difficulty_cutoff) / SCALE + 1.0)
                + excess_speed_difficulty_cutoff / SCALE);

        // * 220 UR and less are considered tapped correctly to ensure that normal scores will be punished as little as possible
        let lerp = 1.0 - reverse_lerp(speed_deviation, 22.0, 27.0);
        adjusted_speed_value = f64::lerp(adjusted_speed_value, speed_value, lerp);

        adjusted_speed_value / speed_value
    }

    // * Miss penalty assumes that a player will miss on the hardest parts of a map,
    // * so we use the amount of relatively difficult sections to adjust miss penalty
    // * to make it more punishing on maps with lower amount of hard sections.
    fn calculate_miss_penalty(miss_count: f64, diff_strain_count: f64) -> f64 {
        0.96 / ((miss_count / (4.0 * diff_strain_count.ln().powf(0.94))) + 1.0)
    }

    fn get_combo_scaling_factor(&self) -> f64 {
        if self.attrs.max_combo == 0 {
            1.0
        } else {
            (f64::from(self.state.max_combo).powf(0.8) / f64::from(self.attrs.max_combo).powf(0.8))
                .min(1.0)
        }
    }

    const fn total_hits(&self) -> f64 {
        self.state.hitresults.total_hits() as f64
    }
}

const fn total_successful_hits(state: &OsuHitResults) -> u32 {
    state.n300 + state.n100 + state.n50
}
