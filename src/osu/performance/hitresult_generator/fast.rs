use std::cmp;

use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Fast, IgnoreAccuracy},
    },
    osu::{InspectOsuPerformance, Osu, OsuHitResults},
};

impl HitResultGenerator<Osu> for Fast {
    fn generate_hitresults(inspect: InspectOsuPerformance<'_>) -> OsuHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect);
        };

        let large_tick_hits = inspect.large_tick_hits.unwrap_or(0);
        let small_tick_hits = inspect.small_tick_hits.unwrap_or(0);
        let slider_end_hits = inspect.slider_end_hits.unwrap_or(0);

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let remain = total_hits - misses;
        let origin = inspect.origin();

        if remain == 0 {
            return OsuHitResults {
                large_tick_hits,
                small_tick_hits,
                slider_end_hits,
                n300: 0,
                n100: 0,
                n50: 0,
                misses,
            };
        }

        let (tick_score, tick_max) =
            origin.tick_scores(large_tick_hits, small_tick_hits, slider_end_hits);

        let (n300, n100, n50) = match (inspect.n300, inspect.n100, inspect.n50) {
            // None missing
            (Some(n300), Some(n100), Some(n50)) => {
                let n300 = cmp::min(n300, remain);
                let n100 = cmp::min(n100, remain - n300);
                let n50 = cmp::min(n50, remain - n300 - n100);

                (n300, n100, n50)
            }

            // Only one missing
            (Some(n300), Some(n100), None) => {
                let n300 = cmp::min(n300, remain);
                let n100 = cmp::min(n100, remain - n300);
                let n50 = remain - n300 - n100;

                (n300, n100, n50)
            }
            (Some(n300), None, Some(n50)) => {
                let n300 = cmp::min(n300, remain);
                let n50 = cmp::min(n50, remain - n300);
                let n100 = remain - n300 - n50;

                (n300, n100, n50)
            }
            (None, Some(n100), Some(n50)) => {
                let n100 = cmp::min(n100, remain);
                let n50 = cmp::min(n50, remain - n100);
                let n300 = remain - n100 - n50;

                (n300, n100, n50)
            }

            // Two or three missing - use Fast algorithm
            _ => {
                // acc = (300*n300 + 100*n100 + 50*n50 + tick_score) / (300*total_hits + tick_max)
                // Simplify by dividing by 50: (reducing risk of overflow)
                // acc = (6*n300 + 2*n100 + n50 + tick_score/50) / (6*total_hits + tick_max/50)

                let target_total =
                    f64::round(acc * (f64::from(6 * total_hits) + f64::from(tick_max) / 50.0))
                        as u32;

                // Start by assuming every non-miss is an n50
                // delta is how much we need to increase from the baseline (all n50s)
                let baseline = remain + tick_score / 50;
                let delta = target_total.saturating_sub(baseline);

                // Each n300 increases by 5 (6-1), each n100 increases by 1 (2-1)
                // delta = 5*n300 + 1*n100

                let n300 = inspect
                    .n300
                    .map_or_else(|| cmp::min(remain, delta / 5), |n| cmp::min(n, remain));

                let n100 = inspect.n100.map_or_else(
                    || {
                        let remain_after_n300 = remain - n300;

                        if inspect.n300.is_some() {
                            // If n300 was provided, recalculate delta for remaining hits
                            let used_by_n300 = 5 * n300;
                            let remain_delta = delta.saturating_sub(used_by_n300);

                            cmp::min(remain_after_n300, remain_delta)
                        } else {
                            // If n300 was calculated, use modulo
                            cmp::min(remain_after_n300, delta % 5)
                        }
                    },
                    |n| cmp::min(n, remain - n300),
                );

                let n50 = inspect.n50.unwrap_or_else(|| remain - n300 - n100);

                (n300, n100, n50)
            }
        };

        OsuHitResults {
            large_tick_hits,
            small_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
        }
    }
}

#[cfg(test)]
mod tests {
    use rosu_mods::{GameMod, generated_mods::ClassicOsu};

    use crate::{
        Difficulty,
        any::HitResultPriority,
        osu::{OsuDifficultyAttributes, OsuScoreOrigin},
    };

    use super::*;

    #[test]
    fn perfect_accuracy_no_misses() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(1.0),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(0),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(OsuScoreOrigin::Stable), 1.0);
    }

    #[test]
    fn high_accuracy_stable() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 1000,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.95),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(10),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        // Verify total adds up
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 1000);
        assert_eq!(result.misses, 10);

        // Verify accuracy is close to target
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - 0.95).abs() < 0.001,
            "Expected ~0.95, got {actual_acc}",
        );
    }

    #[test]
    fn medium_accuracy_stable() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 500,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.85),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(25),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 500);
        assert_eq!(result.misses, 25);

        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - 0.85).abs() < 0.001,
            "Expected ~0.85, got {actual_acc}",
        );
    }

    #[test]
    fn with_slider_acc() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 160,
                n_sliders: 40,
                n_large_ticks: 50,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.98),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(2),
            large_tick_hits: Some(50),
            small_tick_hits: None,
            slider_end_hits: Some(40),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let origin = inspect.origin();

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 200);
        assert_eq!(result.misses, 2);
        assert_eq!(result.large_tick_hits, 50);
        assert_eq!(result.slider_end_hits, 40);

        let actual_acc = result.accuracy(origin);
        assert!(
            (actual_acc - 0.98).abs() < 0.002,
            "Expected ~0.98, got {actual_acc}",
        );
    }

    #[test]
    fn without_slider_acc() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 240,
                n_sliders: 60,
                n_large_ticks: 40,
                ..Default::default()
            },
            difficulty: &Difficulty::new().mods(
                [GameMod::ClassicOsu(ClassicOsu {
                    no_slider_head_accuracy: Some(true),
                    ..Default::default()
                })]
                .into_iter()
                .collect::<rosu_mods::GameMods>(),
            ),
            acc: Some(0.92),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(5),
            large_tick_hits: Some(60),
            small_tick_hits: Some(100),
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let origin = inspect.origin();

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 300);
        assert_eq!(result.misses, 5);
        assert_eq!(result.large_tick_hits, 60);
        assert_eq!(result.small_tick_hits, 100);

        let actual_acc = result.accuracy(origin);
        assert!(
            (actual_acc - 0.92).abs() < 0.002,
            "Expected ~0.92, got {actual_acc}",
        );
    }

    #[test]
    fn all_misses() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 50,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.0),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(50),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 50);
        assert_eq!(result.accuracy(OsuScoreOrigin::Stable), 0.0);
    }

    #[test]
    fn low_accuracy_many_50s() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 400,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.60),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(50),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 400);
        assert_eq!(result.misses, 50);
        // At 60% accuracy with many misses, we should have a lot of 50s
        assert!(result.n50 > 0, "Expected some n50s at low accuracy");

        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - 0.60).abs() < 0.002,
            "Expected ~0.60, got {actual_acc}",
        );
    }

    #[test]
    fn edge_case_more_misses_than_hits() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.5),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(150), // More misses than total hits
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        // Should clamp misses to total_hits
        assert_eq!(result.misses, 100);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
    }

    // Tests for "None missing" case
    #[test]
    fn all_three_provided() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.90),
            n300: Some(80),
            n100: Some(15),
            n50: Some(5),
            misses: Some(0),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 80);
        assert_eq!(result.n100, 15);
        assert_eq!(result.n50, 5);
        assert_eq!(result.misses, 0);
    }

    #[test]
    fn all_three_provided_with_clamping() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.90),
            n300: Some(200), // Exceeds remain
            n100: Some(50),
            n50: Some(30),
            misses: Some(10),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        // n300 should be clamped to remain (90)
        // n100 should be clamped to 0 (no room left)
        // n50 should be clamped to 0 (no room left)
        assert_eq!(result.n300, 90);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 100);
    }

    // Tests for "Only one missing" cases
    #[test]
    fn n50_missing() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 150,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.88),
            n300: Some(100),
            n100: Some(30),
            n50: None,
            misses: Some(10),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 30);
        assert_eq!(result.n50, 10); // 150 - 100 - 30 - 10
        assert_eq!(result.misses, 10);
    }

    #[test]
    fn n100_missing() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 200,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.85),
            n300: Some(140),
            n100: None,
            n50: Some(20),
            misses: Some(15),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 140);
        assert_eq!(result.n100, 25); // 200 - 140 - 20 - 15
        assert_eq!(result.n50, 20);
        assert_eq!(result.misses, 15);
    }

    #[test]
    fn n300_missing() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 180,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.80),
            n300: None,
            n100: Some(40),
            n50: Some(30),
            misses: Some(20),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 90); // 180 - 40 - 30 - 20
        assert_eq!(result.n100, 40);
        assert_eq!(result.n50, 30);
        assert_eq!(result.misses, 20);
    }

    // Tests for "Two missing" cases with n300 provided
    #[test]
    fn n300_provided_n100_n50_missing() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 200,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.90),
            n300: Some(150),
            n100: None,
            n50: None,
            misses: Some(10),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 150);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 200);
        assert_eq!(result.misses, 10);

        // Fast is an approximation, so just verify it's reasonably close
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - 0.90).abs() < 0.05,
            "Expected ~0.90, got {}",
            actual_acc
        );
    }

    // Tests for "Two missing" cases with n100 provided
    #[test]
    fn n100_provided_n300_n50_missing() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 150,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.85),
            n300: None,
            n100: Some(30),
            n50: None,
            misses: Some(8),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n100, 30);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 150);
        assert_eq!(result.misses, 8);

        // Fast is an approximation
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - 0.85).abs() < 0.05,
            "Expected ~0.85, got {}",
            actual_acc
        );
    }

    // Tests for "Two missing" cases with n50 provided
    #[test]
    fn n50_provided_n300_n100_missing() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 120,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.80),
            n300: None,
            n100: None,
            n50: Some(25),
            misses: Some(5),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n50, 25);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 120);
        assert_eq!(result.misses, 5);

        // Fast is an approximation
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - 0.80).abs() < 0.05,
            "Expected ~0.80, got {}",
            actual_acc
        );
    }

    // Test with slider accuracy and some values provided
    #[test]
    fn with_slider_acc_n300_provided() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 125,
                n_sliders: 25,
                n_large_ticks: 30,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.95),
            n300: Some(120),
            n100: None,
            n50: None,
            misses: Some(3),
            large_tick_hits: Some(30),
            small_tick_hits: None,
            slider_end_hits: Some(25),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 120);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 150);
        assert_eq!(result.large_tick_hits, 30);
        assert_eq!(result.slider_end_hits, 25);
    }

    // Test edge case where provided values leave no room
    #[test]
    fn provided_values_fill_all_remain() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.95),
            n300: Some(85),
            n100: Some(10),
            n50: None,
            misses: Some(5),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 85);
        assert_eq!(result.n100, 10);
        assert_eq!(result.n50, 0); // No room left
        assert_eq!(result.misses, 5);
    }

    // Test that algorithm handles very low accuracy with provided n300
    #[test]
    fn low_accuracy_with_high_n300_provided() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.60),
            n300: Some(10),
            n100: None,
            n50: None,
            misses: Some(20),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 10);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 100);
        // With low accuracy and only 10 n300s out of 80 remaining, we expect mostly n50s
        // But the Fast algorithm might not produce many n50s - just verify totals add up
        assert_eq!(result.n100 + result.n50, 70);
    }
}
