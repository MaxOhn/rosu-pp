use std::cmp;

use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Closest, IgnoreAccuracy},
    },
    taiko::{Taiko, TaikoHitResults, performance::InspectTaikoPerformance},
};

impl HitResultGenerator<Taiko> for Closest {
    fn generate_hitresults(inspect: InspectTaikoPerformance<'_>) -> TaikoHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Taiko>>::generate_hitresults(inspect);
        };

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let remain = total_hits - misses;

        let (n300, n100) = match (inspect.n300, inspect.n100) {
            // Both provided
            (Some(n300), Some(n100)) => {
                let n300 = cmp::min(n300, remain);
                let n100 = cmp::min(n100, remain - n300);

                (n300, n100)
            }

            // Only one provided
            (Some(n300), None) => {
                let n300 = cmp::min(n300, remain);
                let n100 = remain - n300;

                (n300, n100)
            }
            (None, Some(n100)) => {
                let n100 = cmp::min(n100, remain);
                let n300 = remain - n100;

                (n300, n100)
            }

            // Both missing
            (None, None) => {
                if remain == 0 {
                    return TaikoHitResults {
                        n300: 0,
                        n100: 0,
                        misses,
                    };
                }

                // acc = (2*n300 + n100) / (2*total_hits)
                // => target_total = acc * 2*total_hits = 2*n300 + n100
                // where n300 + n100 = remain
                // => target_total = 2*n300 + (remain - n300) = n300 + remain
                // => n300 = target_total - remain

                let target_total = acc * f64::from(2 * total_hits);

                let raw300 = target_total - f64::from(remain);

                let min300 = cmp::min(remain, f64::floor(raw300).max(0.0) as u32);
                let max300 = cmp::min(remain, f64::ceil(raw300) as u32);

                let mut best_dist = f64::MAX;
                let mut n300 = 0;
                let mut n100 = remain;

                for new300 in min300..=max300 {
                    let new100 = remain - new300;

                    let state = TaikoHitResults {
                        n300: new300,
                        n100: new100,
                        misses,
                    };

                    let dist = f64::abs(acc - state.accuracy());

                    if dist < best_dist {
                        best_dist = dist;
                        n300 = new300;
                        n100 = new100;
                    }
                }

                (n300, n100)
            }
        };

        TaikoHitResults { n300, n100, misses }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Difficulty, any::HitResultPriority, taiko::TaikoDifficultyAttributes};

    use super::*;

    // Helper function to verify that the result is the closest possible
    fn verify_is_closest(inspect: &InspectTaikoPerformance<'_>, result: &TaikoHitResults) {
        let result_acc = result.accuracy();
        let acc = inspect.acc.unwrap();
        let result_dist = f64::abs(acc - result_acc);

        let total_hits = inspect.total_hits();
        let remain = total_hits - result.misses;

        // Check all possible combinations of n300, n100
        for n300 in 0..=remain {
            let n100 = remain - n300;

            // Skip if any provided constraints are violated
            if let Some(expected_n300) = inspect.n300 {
                if n300 != expected_n300 {
                    continue;
                }
            }
            if let Some(expected_n100) = inspect.n100 {
                if n100 != expected_n100 {
                    continue;
                }
            }

            let candidate = TaikoHitResults {
                n300,
                n100,
                misses: result.misses,
            };

            let candidate_acc = candidate.accuracy();
            let candidate_dist = f64::abs(acc - candidate_acc);

            assert!(
                result_dist <= candidate_dist + 1e-10, // Small epsilon for floating point
                "Found closer result: result has distance {result_dist}, \
                but ({n300}, {n100}) has distance {candidate_dist}",
            );
        }
    }

    #[test]
    fn test_both_provided() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.95),
            n300: Some(90),
            n100: Some(10),
            misses: Some(0),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 90);
        assert_eq!(result.n100, 10);
        assert_eq!(result.misses, 0);
    }

    #[test]
    fn test_n300_provided_n100_missing() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 80,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n300: Some(60),
            n100: None,
            misses: Some(5),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 60);
        assert_eq!(result.n100, 15); // 80 - 60 - 5
        assert_eq!(result.misses, 5);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_n100_provided_n300_missing() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 120,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85),
            n300: None,
            n100: Some(30),
            misses: Some(8),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 82); // 120 - 30 - 8
        assert_eq!(result.n100, 30);
        assert_eq!(result.misses, 8);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_both_missing_perfect_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(1.0),
            n300: None,
            n100: None,
            misses: Some(0),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(), 1.0);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_both_missing_high_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 200,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.96),
            n300: None,
            n100: None,
            misses: Some(5),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300 + result.n100 + result.misses, 200);
        assert_eq!(result.misses, 5);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_both_missing_medium_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 150,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.75),
            n300: None,
            n100: None,
            misses: Some(10),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300 + result.n100 + result.misses, 150);
        assert_eq!(result.misses, 10);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_both_missing_low_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 180,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.60),
            n300: None,
            n100: None,
            misses: Some(20),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300 + result.n100 + result.misses, 180);
        assert_eq!(result.misses, 20);
        // At 60% accuracy, should have more n100s than n300s
        assert!(result.n100 > result.n300);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_both_missing_exactly_50_percent() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.50),
            n300: None,
            n100: None,
            misses: Some(0),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 100);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(), 0.5);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_misses() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 80,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.0),
            n300: None,
            n100: None,
            misses: Some(80),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.misses, 80);
        assert_eq!(result.accuracy(), 0.0);
    }

    #[test]
    fn test_clamping_when_values_exceed_remain() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n300: Some(150), // More than total_hits
            n100: Some(50),
            misses: Some(10),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        assert!(result.n300 <= 90); // Can't exceed remain
    }

    #[test]
    fn test_edge_case_more_misses_than_hits() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 50,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.5),
            n300: None,
            n100: None,
            misses: Some(100), // More than total_hits
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.misses, 50);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
    }

    #[test]
    fn test_rounding_edge_case() {
        // Test case where the exact n300 value falls between integers
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.875), // Should give n300 = 75.5, need to round
            n300: None,
            n100: None,
            misses: Some(0),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_small_total_hits() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 10,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.80),
            n300: None,
            n100: None,
            misses: Some(1),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300 + result.n100 + result.misses, 10);
        assert_eq!(result.misses, 1);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_various_accuracies() {
        // Test a range of accuracies to ensure consistency
        for acc in [0.55, 0.65, 0.72, 0.83, 0.91, 0.98] {
            let inspect = InspectTaikoPerformance {
                attrs: &TaikoDifficultyAttributes {
                    max_combo: 100,
                    ..Default::default()
                },
                difficulty: &Difficulty::new(),
                acc: Some(acc),
                n300: None,
                n100: None,
                misses: Some(3),
                combo: None,
                hitresult_priority: HitResultPriority::BestCase,
            };

            let result =
                <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

            assert_eq!(result.n300 + result.n100 + result.misses, 100);
            verify_is_closest(&inspect, &result);
        }
    }

    #[test]
    fn test_accuracy_closer_than_fast() {
        // Test that Closest produces results at least as close as Fast would
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.847), // Odd accuracy that might not round well
            n300: None,
            n100: None,
            misses: Some(5),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let acc = inspect.acc.unwrap();

        let result = <Closest as HitResultGenerator<Taiko>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        verify_is_closest(&inspect, &result);

        // The result should be very close to target
        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - acc).abs() < 0.01,
            "Expected close to {acc}, got {actual_acc}",
        );
    }
}
