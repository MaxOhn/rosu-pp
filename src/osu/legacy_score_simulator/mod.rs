use rosu_map::section::general::GameMode;

use crate::{
    Beatmap,
    any::hit_result::HitResult,
    model::beatmap::BeatmapAttributes,
    osu::object::{NestedSliderObjectKind, OsuObject, OsuObjectKind},
    util::ruleset_ext::calculate_difficulty_peppy_stars,
};

pub mod gradual;

pub struct OsuLegacyScoreSimulator<'a> {
    osu_objects: &'a [OsuObject],
    passed_objects: usize,
    inner: LegacyScoreSimulatorInner,
    score_multiplier: f64,
}

impl<'a> OsuLegacyScoreSimulator<'a> {
    pub fn new(osu_objects: &'a [OsuObject], map: &Beatmap, passed_objects: usize) -> Self {
        // Note that no mods are being applied here. Apparently, this is how
        // lazer wants to it /shrug
        let map_attrs = map.attributes().build();

        let score_multiplier = Self::score_multiplier(map, &map_attrs, passed_objects);

        Self {
            osu_objects,
            passed_objects,
            inner: LegacyScoreSimulatorInner::default(),
            score_multiplier: f64::from(score_multiplier),
        }
    }

    pub fn score_multiplier(
        map: &Beatmap,
        map_attrs: &BeatmapAttributes,
        passed_objects: usize,
    ) -> i32 {
        let hit_objects = &map.hit_objects;

        let object_count = hit_objects.iter().take(passed_objects).count() as i32;

        let mut drain_len = 0;

        let first = hit_objects.first();

        let last = hit_objects
            .get(passed_objects.saturating_sub(1))
            .or(hit_objects.last());

        if let Some((first, last)) = first.zip(last) {
            let break_len: i32 = map
                .breaks
                .iter()
                // Note that this does not account for notes appearing during
                // breaks or overlapping breaks.
                .take_while(|b| b.end_time < last.start_time)
                .map(|b| {
                    b.end_time.round_ties_even() as i32 - b.start_time.round_ties_even() as i32
                })
                .sum();

            let full_len = last.start_time.round_ties_even() as i32
                - first.start_time.round_ties_even() as i32;

            drain_len = (full_len - break_len) / 1000;
        }

        calculate_difficulty_peppy_stars(map_attrs, object_count, drain_len)
    }

    pub fn simulate(&mut self) -> LegacyScoreAttributes {
        let mut attrs = LegacyScoreAttributes::default();

        for obj in self.osu_objects.iter().take(self.passed_objects) {
            self.simulate_hit(obj, &mut attrs);
        }

        self.inner.finalize(&mut attrs);

        attrs
    }

    fn simulate_hit(&mut self, hit_object: &OsuObject, attrs: &mut LegacyScoreAttributes) {
        const DEFAULT_BONUS_RESULT: HitResult = HitResult::None;

        match hit_object.kind {
            OsuObjectKind::Circle => {
                self.unrolled_recursion(
                    attrs,
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
                    attrs,
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
                                attrs,
                                AddScoreComboMultiplier::default(),
                                IsBonus::default(),
                                IncreaseCombo::default(),
                                30,
                                DEFAULT_BONUS_RESULT,
                            );
                        }
                        NestedSliderObjectKind::Tick => {
                            self.unrolled_recursion(
                                attrs,
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
                    attrs,
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
                            attrs,
                            AddScoreComboMultiplier::default(),
                            IsBonus::Yes,
                            IncreaseCombo::No,
                            1100,
                            HitResult::LargeBonus,
                        );
                    } else if i > 1 && i % 2 == 0 {
                        self.unrolled_recursion(
                            attrs,
                            AddScoreComboMultiplier::default(),
                            IsBonus::Yes,
                            IncreaseCombo::No,
                            100,
                            HitResult::SmallBonus,
                        );
                    }
                }

                self.unrolled_recursion(
                    attrs,
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
        attrs: &mut LegacyScoreAttributes,
        add_score_combo_multiplier: AddScoreComboMultiplier,
        is_bonus: IsBonus,
        increase_combo: IncreaseCombo,
        score_increase: i32,
        bonus_result: HitResult,
    ) {
        let factor = self.inner.unrolled_recursion(
            attrs,
            add_score_combo_multiplier,
            is_bonus,
            increase_combo,
            score_increase,
            bonus_result,
        );

        if let Some(factor) = factor {
            attrs.combo_score += i64::from((factor * self.score_multiplier) as i32);
        }
    }
}

#[derive(Default)]
struct LegacyScoreSimulatorInner {
    legacy_bonus_score: i32,
    standardised_bonus_score: i32,
    combo: u32,
}

#[derive(Copy, Clone, Default)]
enum AddScoreComboMultiplier {
    Yes,
    #[default]
    No,
}

#[derive(Copy, Clone, Default)]
enum IsBonus {
    Yes,
    #[default]
    No,
}

#[derive(Copy, Clone, Default)]
enum IncreaseCombo {
    #[default]
    Yes,
    No,
}

impl LegacyScoreSimulatorInner {
    fn unrolled_recursion(
        &mut self,
        attrs: &mut LegacyScoreAttributes,
        add_score_combo_multiplier: AddScoreComboMultiplier,
        is_bonus: IsBonus,
        increase_combo: IncreaseCombo,
        score_increase: i32,
        bonus_result: HitResult,
    ) -> Option<f64> {
        let mut factor = None;

        if let AddScoreComboMultiplier::Yes = add_score_combo_multiplier {
            factor = Some(f64::from(self.combo.saturating_sub(1)) * f64::from(score_increase / 25));
        }

        if let IsBonus::Yes = is_bonus {
            self.legacy_bonus_score += score_increase;
            self.standardised_bonus_score += bonus_result.base_score(GameMode::Osu);
        } else {
            attrs.accuracy_score += score_increase;
        }

        if let IncreaseCombo::Yes = increase_combo {
            self.combo += 1;
        }

        factor
    }

    fn finalize(&self, attrs: &mut LegacyScoreAttributes) {
        attrs.bonus_score_ratio = if self.legacy_bonus_score == 0 {
            0.0
        } else {
            f64::from(self.standardised_bonus_score) / f64::from(self.legacy_bonus_score)
        };

        attrs.bonus_score = self.legacy_bonus_score;
        attrs.max_combo = self.combo;
    }
}

#[derive(Clone, Default)]
pub struct LegacyScoreAttributes {
    pub accuracy_score: i32,
    pub combo_score: i64,
    pub bonus_score_ratio: f64,
    pub bonus_score: i32,
    pub max_combo: u32,
}
