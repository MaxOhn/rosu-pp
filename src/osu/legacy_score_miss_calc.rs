use std::cmp;

use crate::{
    GameMods,
    osu::{OsuDifficultyAttributes, OsuScoreState},
};

pub struct OsuLegacyScoreMissCalculator<'a> {
    state: &'a OsuScoreState,
    acc: f64,
    mods: &'a GameMods,
    attrs: &'a OsuDifficultyAttributes,
}

impl<'a> OsuLegacyScoreMissCalculator<'a> {
    pub const fn new(
        state: &'a OsuScoreState,
        acc: f64,
        mods: &'a GameMods,
        attrs: &'a OsuDifficultyAttributes,
    ) -> Self {
        Self {
            state,
            acc,
            mods,
            attrs,
        }
    }

    pub fn calculate(self) -> f64 {
        let Self { state, attrs, .. } = self;

        if attrs.max_combo == 0 {
            return 0.0;
        }

        let Some(legacy_total_score) = state.legacy_total_score else {
            return 0.0;
        };

        let score_v1_multiplier =
            self.attrs.legacy_score_base_multiplier * self.get_legacy_score_multiplier();
        let relevant_combo_per_object = self.calculate_relevant_score_combo_per_object();

        let maximum_miss_count = self.calculate_maximum_combo_based_miss_count();

        let score_obtained_during_max_combo = self.calculate_score_at_combo(
            state.max_combo,
            relevant_combo_per_object,
            score_v1_multiplier,
        );
        let remaining_score = f64::from(legacy_total_score) - score_obtained_during_max_combo;

        if remaining_score <= 0.0 {
            return maximum_miss_count;
        }

        let remaining_combo = attrs.max_combo - state.max_combo;
        let expected_remaining_score = self.calculate_score_at_combo(
            remaining_combo,
            relevant_combo_per_object,
            score_v1_multiplier,
        );

        let mut score_based_miss_count = expected_remaining_score / remaining_score;

        // * If there's less than one miss detected - let combo-based miss count decide if this is FC or not
        score_based_miss_count = score_based_miss_count.max(1.0);

        // * Cap result by very harsh version of combo-based miss count
        score_based_miss_count.min(maximum_miss_count)
    }

    fn calculate_score_at_combo(
        &self,
        combo: u32,
        relevant_combo_per_object: f64,
        score_v1_multiplier: f64,
    ) -> f64 {
        let Self {
            state, acc, attrs, ..
        } = self;

        let total_hits = state.hitresults.total_hits();

        let estimated_objects = f64::from(combo) / relevant_combo_per_object - 1.0;

        // * The combo portion of ScoreV1 follows arithmetic progression
        // * Therefore, we calculate the combo portion of score using the combo per object and our current combo.
        let mut combo_score = if relevant_combo_per_object > 0.0 {
            (2.0 * (relevant_combo_per_object - 1.0)
                + (estimated_objects - 1.0) * relevant_combo_per_object)
                * estimated_objects
                / 2.0
        } else {
            0.0
        };

        // * We then apply the accuracy and ScoreV1 multipliers to the resulting score.
        combo_score *= acc * 300.0 / 25.0 * score_v1_multiplier;

        let objects_hit = f64::from(total_hits - state.hitresults.misses) * f64::from(combo)
            / f64::from(attrs.max_combo);

        // * Score also has a non-combo portion we need to create the final score value.
        let non_combo_score = (300.0 + attrs.nested_score_per_object) * acc * objects_hit;

        combo_score + non_combo_score
    }

    // * Calculates the relevant combo per object for legacy score.
    // * This assumes a uniform distribution for circles and sliders.
    // * This handles cases where objects (such as buzz sliders) do not fit a normal arithmetic progression model.
    fn calculate_relevant_score_combo_per_object(&self) -> f64 {
        let attrs = self.attrs;
        let mut combo_score = attrs.maximum_legacy_combo_score;

        // * We then reverse apply the ScoreV1 multipliers to get the raw value.
        combo_score /= 300.0 / 25.0 * attrs.legacy_score_base_multiplier;

        // * Reverse the arithmetic progression to work out the amount of combo per object based on the score.
        let mut result = f64::from((attrs.max_combo as i32 - 2) * attrs.max_combo as i32);
        result /= (f64::from(attrs.max_combo) + 2.0 * (combo_score - 1.0)).max(1.0);

        result
    }

    fn calculate_maximum_combo_based_miss_count(&self) -> f64 {
        let Self { state, attrs, .. } = self;

        if attrs.n_sliders == 0 {
            return f64::from(state.hitresults.misses);
        }

        let total_imperfect_hits =
            state.hitresults.n100 + state.hitresults.n50 + state.hitresults.misses;

        let mut miss_count = 0.0;

        // * Consider that full combo is maximum combo minus dropped slider tails since they don't contribute to combo but also don't break it
        // * In classic scores we can't know the amount of dropped sliders so we estimate to 10% of all sliders on the map
        let full_combo_threshold = f64::from(attrs.max_combo) - 0.1 * f64::from(attrs.n_circles);

        if f64::from(state.max_combo) < full_combo_threshold {
            miss_count = (full_combo_threshold / f64::from(state.max_combo).max(1.0)).powf(2.5);
        }

        // * In classic scores there can't be more misses than a sum of all non-perfect judgements
        miss_count = miss_count.min(f64::from(total_imperfect_hits));

        // * Every slider has *at least* 2 combo attributed in classic mechanics.
        // * If they broke on a slider with a tick, then this still works since they would have lost at least 2 combo (the tick and the end)
        // * Using this as a max means a score that loses 1 combo on a map can't possibly have been a slider break.
        // * It must have been a slider end.
        let max_possible_slider_breaks =
            cmp::min((attrs.max_combo - state.max_combo) / 2, attrs.n_sliders);

        let slider_breaks = miss_count - f64::from(state.hitresults.misses);

        if slider_breaks > f64::from(max_possible_slider_breaks) {
            miss_count = f64::from(state.hitresults.misses + max_possible_slider_breaks);
        }

        miss_count
    }

    fn get_legacy_score_multiplier(&self) -> f64 {
        let mods = self.mods;
        let score_v2 = mods.sv2();
        let mut multiplier = 1.0;

        if mods.nf() {
            multiplier *= if score_v2 { 1.0 } else { 0.5 };
        }

        if mods.ez() {
            multiplier *= 0.5;
        }

        if mods.clock_rate() < 1.0 {
            multiplier *= 0.3;
        }

        if mods.hd() {
            multiplier *= 1.06;
        }

        if mods.hr() {
            multiplier *= if score_v2 { 1.10 } else { 1.06 };
        }

        if mods.clock_rate() > 1.0 {
            multiplier *= if score_v2 { 1.20 } else { 1.12 };
        }

        if mods.fl() {
            multiplier *= 1.12;
        }

        if mods.so() {
            multiplier *= 0.9;
        }

        if mods.rx() || mods.ap() {
            multiplier *= 0.0;
        }

        multiplier
    }
}
