use std::{cmp, iter::Peekable, vec};

use rosu_map::section::events::BreakPeriod;

use crate::{
    any::hit_result::HitResult,
    model::beatmap::BeatmapAttributes,
    osu::{
        legacy_score_simulator::{
            AddScoreComboMultiplier, IncreaseCombo, IsBonus, LegacyScoreAttributes,
            LegacyScoreSimulatorInner,
        },
        object::{NestedSliderObjectKind, OsuObject, OsuObjectKind},
    },
    util::ruleset_ext::calculate_difficulty_peppy_stars,
};

pub struct GradualLegacyScoreSimulator {
    map_attrs: BeatmapAttributes,
    attrs: LegacyScoreAttributes,
    inner: super::LegacyScoreSimulatorInner,
    combo_score_factors: Vec<f64>,

    breaks: Peekable<vec::IntoIter<BreakPeriod>>,
    elapsed_curr_break: Option<i32>,
    break_len_prelim: i32,
    object_count: i32,
    start_time: Option<i32>,

    pub prev_score_multiplier: Option<f64>,
}

impl GradualLegacyScoreSimulator {
    pub fn new(breaks: Vec<BreakPeriod>, map_attrs: BeatmapAttributes) -> Self {
        Self {
            map_attrs,
            attrs: LegacyScoreAttributes::default(),
            inner: LegacyScoreSimulatorInner::default(),
            combo_score_factors: Vec::new(),
            breaks: breaks.into_iter().peekable(),
            elapsed_curr_break: None,
            break_len_prelim: 0,
            object_count: 0,
            start_time: None,
            prev_score_multiplier: None,
        }
    }

    const fn break_len(&self) -> i32 {
        // Manual `.unwrap_or(0)` for const-ness
        self.break_len_prelim
            + if let Some(elapsed) = self.elapsed_curr_break {
                elapsed
            } else {
                0
            }
    }

    fn score_multiplier(&mut self, obj: &OsuObject) -> f64 {
        const fn round_time(time: f64) -> i32 {
            time.round_ties_even() as i32
        }

        // Note that this logic does not handle the case properly when
        // breaks are overlapping but that seems like a pathological /
        // malicious case anyway.
        // Another unhandled case are breaks *after* the last object.
        while let Some(b) = self.breaks.peek() {
            if b.start_time >= obj.start_time {
                break;
            }

            if b.end_time < obj.start_time {
                self.break_len_prelim += round_time(b.end_time) - round_time(b.start_time);
                self.elapsed_curr_break.take();
                self.breaks.next();
            } else {
                // Do we even need to handle the case of objects appearing
                // during a break? Probably yes because those pesky mappers
                // will find a way...

                let period_end = cmp::min(round_time(b.end_time), round_time(obj.start_time));
                self.elapsed_curr_break = Some(period_end - round_time(b.start_time));
            }
        }

        self.object_count += 1;

        let start_time = *self.start_time.get_or_insert(round_time(obj.start_time));
        let end_time = round_time(obj.start_time);
        let drain_len = (end_time - start_time - self.break_len()) / 1000;

        let score_multiplier = f64::from(calculate_difficulty_peppy_stars(
            &self.map_attrs,
            self.object_count,
            drain_len,
        ));

        *self.prev_score_multiplier.insert(score_multiplier)
    }

    pub fn simulate_next(&mut self, obj: &OsuObject) -> LegacyScoreAttributes {
        let score_multiplier = self.score_multiplier(obj);
        self.simulate_hit(obj);

        let combo_score = self
            .combo_score_factors
            .iter()
            .fold(0.0, |combo_score, &factor| {
                combo_score + factor * score_multiplier
            });

        self.attrs.combo_score = i64::from(combo_score as i32);

        self.inner.finalize(&mut self.attrs);

        self.attrs.clone()
    }

    fn simulate_hit(&mut self, hit_object: &OsuObject) {
        const DEFAULT_BONUS_RESULT: HitResult = HitResult::None;

        match hit_object.kind {
            OsuObjectKind::Circle => {
                self.unrolled_recursion(
                    AddScoreComboMultiplier::Yes,
                    IsBonus::default(),
                    IncreaseCombo::default(),
                    300,
                    DEFAULT_BONUS_RESULT,
                );
            }
            OsuObjectKind::Slider(ref slider) => {
                // Slider head
                self.unrolled_recursion(
                    AddScoreComboMultiplier::default(),
                    IsBonus::default(),
                    IncreaseCombo::default(),
                    30,
                    DEFAULT_BONUS_RESULT,
                );

                for nested in slider.nested_objects.iter() {
                    match nested.kind {
                        NestedSliderObjectKind::Repeat | NestedSliderObjectKind::Tail => {
                            self.unrolled_recursion(
                                AddScoreComboMultiplier::default(),
                                IsBonus::default(),
                                IncreaseCombo::default(),
                                30,
                                DEFAULT_BONUS_RESULT,
                            );
                        }
                        NestedSliderObjectKind::Tick => {
                            self.unrolled_recursion(
                                AddScoreComboMultiplier::default(),
                                IsBonus::default(),
                                IncreaseCombo::default(),
                                10,
                                DEFAULT_BONUS_RESULT,
                            );
                        }
                    }
                }

                self.unrolled_recursion(
                    AddScoreComboMultiplier::Yes,
                    IsBonus::default(),
                    IncreaseCombo::No,
                    300,
                    DEFAULT_BONUS_RESULT,
                );
            }
            OsuObjectKind::Spinner(ref spinner) => {
                // * The spinner object applies a lenience because gameplay mechanics differ from osu-stable.
                // * We'll redo the calculations to match osu-stable here...
                const MAXIMUM_ROTATIONS_PER_SECOND: f64 = 477.0 / 60.0;

                // * Normally, this value depends on the final overall difficulty. For simplicity, we'll only consider the worst case that maximises bonus score.
                // * As we're primarily concerned with computing the maximum theoretical final score,
                // * this will have the final effect of slightly underestimating bonus score achieved on stable when converting from score V1.
                const MINIMUM_ROTATIONS_PER_SECOND: f64 = 3.0;

                let seconds_duration = spinner.duration / 1000.0;

                // * The total amount of half spins possible for the entire spinner.
                let total_half_spins_possible =
                    (seconds_duration * MAXIMUM_ROTATIONS_PER_SECOND * 2.0) as i32;
                // * The amount of half spins that are required to successfully complete the spinner (i.e. get a 300).
                let half_spins_required_for_completion =
                    (seconds_duration * MINIMUM_ROTATIONS_PER_SECOND) as i32;
                // * To be able to receive bonus points, the spinner must be rotated another 1.5 times.
                let half_spins_required_before_bonus = half_spins_required_for_completion + 3;

                for i in 0..=total_half_spins_possible {
                    if i > half_spins_required_before_bonus
                        && (i - half_spins_required_before_bonus) % 2 == 0
                    {
                        self.unrolled_recursion(
                            AddScoreComboMultiplier::default(),
                            IsBonus::Yes,
                            IncreaseCombo::No,
                            1100,
                            HitResult::LargeBonus,
                        );
                    } else if i > 1 && i % 2 == 0 {
                        self.unrolled_recursion(
                            AddScoreComboMultiplier::default(),
                            IsBonus::Yes,
                            IncreaseCombo::No,
                            100,
                            HitResult::SmallBonus,
                        );
                    }
                }

                self.unrolled_recursion(
                    AddScoreComboMultiplier::Yes,
                    IsBonus::default(),
                    IncreaseCombo::default(),
                    300,
                    DEFAULT_BONUS_RESULT,
                );
            }
        }
    }

    fn unrolled_recursion(
        &mut self,
        add_score_combo_multiplier: AddScoreComboMultiplier,
        is_bonus: IsBonus,
        increase_combo: IncreaseCombo,
        score_increase: i32,
        bonus_result: HitResult,
    ) {
        let factor = self.inner.unrolled_recursion(
            &mut self.attrs,
            add_score_combo_multiplier,
            is_bonus,
            increase_combo,
            score_increase,
            bonus_result,
        );

        if let Some(factor) = factor {
            self.combo_score_factors.push(factor);
        }
    }
}
