use std::cmp;

use crate::{
    any::{
        HitResultGenerator, HitResultPriority,
        hitresult_generator::{Fast, IgnoreAccuracy},
    },
    mania::{Mania, ManiaHitResults, performance::InspectManiaPerformance},
};

impl HitResultGenerator<Mania> for Fast {
    #[expect(clippy::too_many_lines, reason = "it is what it is /shrug")]
    fn generate_hitresults(inspect: InspectManiaPerformance<'_>) -> ManiaHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Mania>>::generate_hitresults(inspect);
        };

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let remain = total_hits - misses;

        if remain == 0 {
            return ManiaHitResults {
                n320: 0,
                n300: 0,
                n200: 0,
                n100: 0,
                n50: 0,
                misses,
            };
        }

        let prelim_320 = inspect.n320.map_or(0, |n| cmp::min(n, remain));
        let prelim_300 = inspect.n300.map_or(0, |n| cmp::min(n, remain - prelim_320));
        let prelim_200 = inspect
            .n200
            .map_or(0, |n| cmp::min(n, remain - prelim_320 - prelim_300));
        let prelim_100 = inspect.n100.map_or(0, |n| {
            cmp::min(n, remain - prelim_320 - prelim_300 - prelim_200)
        });
        let prelim_50 = inspect.n50.map_or(0, |n| {
            cmp::min(
                n,
                remain - prelim_320 - prelim_300 - prelim_200 - prelim_100,
            )
        });

        // Handle cases based on how many values are provided
        let num_provided = [
            inspect.n320,
            inspect.n300,
            inspect.n200,
            inspect.n100,
            inspect.n50,
        ]
        .into_iter()
        .flatten()
        .count();

        if num_provided == 5 {
            // All provided
            return ManiaHitResults {
                n320: prelim_320,
                n300: prelim_300,
                n200: prelim_200,
                n100: prelim_100,
                n50: prelim_50,
                misses,
            };
        }

        if num_provided == 4 {
            // One missing

            // Fill in the missing value
            let used = prelim_320 + prelim_300 + prelim_200 + prelim_100 + prelim_50;
            let left = remain - used;

            let n320 = if inspect.n320.is_none() {
                left
            } else {
                prelim_320
            };

            let n300 = if inspect.n300.is_none() {
                left
            } else {
                prelim_300
            };

            let n200 = if inspect.n200.is_none() {
                left
            } else {
                prelim_200
            };

            let n100 = if inspect.n100.is_none() {
                left
            } else {
                prelim_100
            };

            let n50 = if inspect.n50.is_none() {
                left
            } else {
                prelim_50
            };

            return ManiaHitResults {
                n320,
                n300,
                n200,
                n100,
                n50,
                misses,
            };
        }

        let is_classic = inspect.is_classic();

        // For 3 or fewer provided values, use Fast algorithm
        // Accuracy formula: acc = (w*n320 + 60*n300 + 40*n200 + 20*n100 + 10*n50) / (w*total_hits)
        // where w = 60 (classic) or 61 (non-classic)
        let perfect_weight = if is_classic { 60 } else { 61 };

        let numerator = perfect_weight * prelim_320
            + 60 * prelim_300
            + 40 * prelim_200
            + 20 * prelim_100
            + 10 * prelim_50;

        let denominator = perfect_weight * total_hits;

        let target_total =
            f64::round((acc * f64::from(denominator) - f64::from(numerator)).max(0.0)) as u32;

        // Start by assuming every non-miss is an n50 (baseline)
        let baseline =
            10 * (remain - prelim_320 - prelim_300 - prelim_200 - prelim_100 - prelim_50);
        let mut delta = target_total.saturating_sub(baseline);

        // Each upgrade from n50 increases total:
        // n50 -> n100: +10 (20-10)
        // n50 -> n200: +30 (40-10)
        // n50 -> n300: +50 (60-10)
        // n50 -> n320: +(perfect_weight-10) = 50 or 51

        let n320_increase = perfect_weight - 10;

        let n320 = cmp::min(
            remain - prelim_300 - prelim_200 - prelim_100 - prelim_50,
            inspect.n320.unwrap_or(delta / n320_increase),
        );

        if inspect.n320.is_none() {
            delta = delta.saturating_sub(n320_increase * n320);
        }

        let n300 = cmp::min(
            remain - n320 - prelim_200 - prelim_100 - prelim_50,
            inspect.n300.unwrap_or(delta / 50),
        );

        if inspect.n300.is_none() {
            delta = delta.saturating_sub(50 * n300);
        }

        let n200 = cmp::min(
            remain - n320 - n300 - prelim_100 - prelim_50,
            inspect.n200.unwrap_or(delta / 30),
        );

        if inspect.n200.is_none() {
            delta = delta.saturating_sub(30 * n200);
        }

        let n100 = cmp::min(
            remain - n320 - n300 - n200 - prelim_50,
            inspect.n100.unwrap_or(delta / 10),
        );

        let n50 = cmp::min(
            remain - n320 - n300 - n200 - n100,
            inspect.n50.unwrap_or(remain),
        );

        let mut hitresults = ManiaHitResults {
            n320,
            n300,
            n200,
            n100,
            n50,
            misses,
        };

        if hitresults.total_hits() < total_hits {
            let left = total_hits - hitresults.total_hits();

            match inspect.hitresult_priority {
                HitResultPriority::BestCase => match inspect {
                    InspectManiaPerformance { n320: None, .. } => hitresults.n320 += left,
                    InspectManiaPerformance { n300: None, .. } => hitresults.n300 += left,
                    InspectManiaPerformance { n200: None, .. } => hitresults.n200 += left,
                    InspectManiaPerformance { n100: None, .. } => hitresults.n100 += left,
                    _ => hitresults.n50 += left,
                },
                HitResultPriority::WorstCase => match inspect {
                    InspectManiaPerformance { n50: None, .. } => hitresults.n50 += left,
                    InspectManiaPerformance { n100: None, .. } => hitresults.n100 += left,
                    InspectManiaPerformance { n200: None, .. } => hitresults.n200 += left,
                    InspectManiaPerformance { n300: None, .. } => hitresults.n300 += left,
                    _ => hitresults.n320 += left,
                },
            }
        }

        hitresults
    }
}

#[cfg(test)]
mod tests {
    use crate::{Difficulty, mania::ManiaDifficultyAttributes};

    use super::*;

    #[test]
    fn test_perfect_accuracy_non_classic() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(1.0),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(0),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 100);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(false), 1.0);
    }

    #[test]
    fn test_perfect_accuracy_classic() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(1.0),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(0),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 100);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(true), 1.0);
    }

    #[test]
    fn test_high_accuracy_non_classic() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 500,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.95),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.total_hits(), 500);
        assert_eq!(result.misses, 10);

        let actual_acc = result.accuracy(false);

        assert!(
            (actual_acc - 0.95).abs() < 0.001,
            "Expected ~0.95, got {actual_acc}",
        );
    }

    #[test]
    fn test_medium_accuracy_classic() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 400,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.80),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(20),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.total_hits(), 400);
        assert_eq!(result.misses, 20);

        let actual_acc = result.accuracy(true);

        assert!(
            (actual_acc - 0.80).abs() < 0.002,
            "Expected ~0.80, got {actual_acc}",
        );
    }

    #[test]
    fn test_low_accuracy() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 300,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.60),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(30),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.total_hits(), 300);
        assert_eq!(result.misses, 30);

        // At low accuracy, should have many lower-value hits
        assert!(result.n100 > 0 || result.n50 > 0);

        let actual_acc = result.accuracy(false);

        assert!(
            (actual_acc - 0.60).abs() < 0.002,
            "Expected ~0.60, got {actual_acc}",
        );
    }

    #[test]
    fn test_all_misses() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 50,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.0),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(50),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 0);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 50);
        assert_eq!(result.accuracy(false), 0.0);
    }

    #[test]
    fn test_all_five_provided() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n320: Some(50),
            n300: Some(30),
            n200: Some(10),
            n100: Some(5),
            n50: Some(5),
            misses: Some(0),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 50);
        assert_eq!(result.n300, 30);
        assert_eq!(result.n200, 10);
        assert_eq!(result.n100, 5);
        assert_eq!(result.n50, 5);
        assert_eq!(result.misses, 0);
    }

    #[test]
    fn test_four_provided_n50_missing() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 150,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.88),
            n320: Some(80),
            n300: Some(40),
            n200: Some(15),
            n100: Some(10),
            n50: None,
            misses: Some(5),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 80);
        assert_eq!(result.n300, 40);
        assert_eq!(result.n200, 15);
        assert_eq!(result.n100, 10);
        assert_eq!(result.n50, 0); // 150 - 80 - 40 - 15 - 10 - 5
        assert_eq!(result.misses, 5);
    }

    #[test]
    fn test_four_provided_n320_missing() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 200,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.85),
            n320: None,
            n300: Some(100),
            n200: Some(50),
            n100: Some(20),
            n50: Some(10),
            misses: Some(20),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 0); // 200 - 100 - 50 - 20 - 10 - 20
        assert_eq!(result.n300, 100);
        assert_eq!(result.n200, 50);
        assert_eq!(result.n100, 20);
        assert_eq!(result.n50, 10);
        assert_eq!(result.misses, 20);
    }

    #[test]
    fn test_three_provided_n320_n300_missing() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 200,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.65),
            n320: None,
            n300: None,
            n200: Some(50),
            n100: Some(20),
            n50: Some(10),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), inspect.total_hits());
        assert_eq!(result.n200, inspect.n200.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses());

        let actual_acc = result.accuracy(inspect.is_classic());

        // The algorithm realizes too late that n200 to n50 are already assigned
        // and adds the missing values to n320, resulting in a higher accuracy
        assert!(
            (actual_acc - 0.75).abs() < 0.1,
            "Expected ~0.75, got {actual_acc}",
        );
    }

    #[test]
    fn test_two_provided_n200_n100_n50_missing() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 180,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.92),
            n320: Some(100),
            n300: Some(50),
            n200: None,
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 100);
        assert_eq!(result.n300, 50);
        assert_eq!(result.total_hits(), 180);
        assert_eq!(result.misses, 10);

        let actual_acc = result.accuracy(false);

        assert!(
            (actual_acc - 0.92).abs() < 0.05,
            "Expected ~0.92, got {actual_acc}",
        );
    }

    #[test]
    fn test_two_provided_n320_n300_n50_missing() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 120,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.75),
            n320: None,
            n300: None,
            n200: Some(30),
            n100: Some(5),
            n50: None,
            misses: Some(15),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n200, 30);
        assert_eq!(result.n100, 5);
        assert_eq!(result.total_hits(), 120);
        assert_eq!(result.misses, 15);

        let actual_acc = result.accuracy(false);

        assert!(
            (actual_acc - 0.75).abs() < 0.05,
            "Expected ~0.75, got {actual_acc}",
        );
    }

    #[test]
    fn test_one_provided_n320_only() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.88),
            n320: Some(60),
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(8),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.n320, 60);
        assert_eq!(result.total_hits(), 100);
        assert_eq!(result.misses, 8);

        let actual_acc = result.accuracy(true);

        assert!(
            (actual_acc - 0.88).abs() < 0.05,
            "Expected ~0.88, got {actual_acc}",
        );
    }

    #[test]
    fn test_edge_case_more_misses_than_hits() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 50,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.5),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(100), // More than total_hits
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        assert_eq!(result.misses, 50); // Should be clamped
        assert_eq!(result.n320, 0);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
    }

    #[test]
    fn test_clamping_provided_values_exceed_remain() {
        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n320: Some(150), // Exceeds total
            n300: Some(50),
            n200: Some(30),
            n100: Some(20),
            n50: Some(10),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

        // n320 should be clamped to remain (90)
        assert_eq!(result.n320, 90);
        assert_eq!(result.n300, 0); // No room left
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.total_hits(), 100);
    }

    #[test]
    fn test_accuracy_difference_classic_vs_non_classic() {
        // Test that the algorithm handles classic vs non-classic correctly
        for is_classic in [false, true] {
            let inspect = InspectManiaPerformance {
                attrs: &ManiaDifficultyAttributes {
                    n_objects: 200,
                    ..Default::default()
                },
                difficulty: &Difficulty::new().lazer(!is_classic),
                acc: Some(0.90),
                n320: None,
                n300: None,
                n200: None,
                n100: None,
                n50: None,
                misses: Some(5),
                hitresult_priority: HitResultPriority::BestCase,
            };

            let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

            assert_eq!(result.total_hits(), 200);
            assert_eq!(result.misses, 5);

            let actual_acc = result.accuracy(is_classic);

            assert!(
                (actual_acc - 0.90).abs() < 0.002,
                "Expected ~0.90 for is_classic={is_classic}, got {actual_acc}",
            );
        }
    }

    #[test]
    fn test_various_accuracies_non_classic() {
        for acc in [0.50, 0.65, 0.75, 0.85, 0.95, 0.98] {
            let inspect = InspectManiaPerformance {
                attrs: &ManiaDifficultyAttributes {
                    n_objects: 400,
                    ..Default::default()
                },
                difficulty: &Difficulty::new(),
                acc: Some(acc),
                n320: None,
                n300: None,
                n200: None,
                n100: None,
                n50: None,
                misses: Some(8),
                hitresult_priority: HitResultPriority::BestCase,
            };

            let result = <Fast as HitResultGenerator<Mania>>::generate_hitresults(inspect);

            assert_eq!(result.total_hits(), 400);
            assert_eq!(result.misses, 8);

            let actual_acc = result.accuracy(false);

            assert!(
                (actual_acc - acc).abs() < 0.001,
                "For target acc {acc}, got {actual_acc}",
            );
        }
    }
}
