use std::cmp;

use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Fast, IgnoreAccuracy},
    },
    taiko::{Taiko, TaikoHitResults, performance::InspectTaikoPerformance},
};

impl HitResultGenerator<Taiko> for Fast {
    fn generate_hitresults(inspect: InspectTaikoPerformance<'_>) -> TaikoHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Taiko>>::generate_hitresults(inspect);
        };

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let remain = total_hits - misses;

        let (n300, n100) = match (inspect.n300, inspect.n100) {
            (Some(n300), Some(n100)) => {
                let n300 = cmp::min(n300, remain);
                let n100 = cmp::min(n100, remain - n300);

                (n300, n100)
            }
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
            (None, None) => {
                if remain == 0 {
                    return TaikoHitResults {
                        n300: 0,
                        n100: 0,
                        misses,
                    };
                }

                // acc = (2*n300 + n100) / (2*total_hits)
                // Simplify by multiplying by total_hits:
                // acc * (2*total_hits) = 2*n300 + n100

                let target_total = f64::round(acc * f64::from(2 * total_hits)) as u32;

                // Start by assuming every non-miss is an n100
                // delta is how much we need to increase from the baseline (all n100s)
                let baseline = remain;
                let delta = target_total.saturating_sub(baseline);

                // Each n300 increases by 1 (2-1)
                // delta = 1*n300

                let n300 = cmp::min(remain, delta);
                let n100 = remain - n300;

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

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 90);
        assert_eq!(result.n100, 10);
        assert_eq!(result.misses, 0);
        assert_eq!(result.n300 + result.n100 + result.misses, 100);
    }

    #[test]
    fn test_n300_provided_n100_missing() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 150,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n300: Some(120),
            n100: None,
            misses: Some(10),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 120);
        assert_eq!(result.n100, 20); // 150 - 120 - 10
        assert_eq!(result.misses, 10);
        assert_eq!(result.n300 + result.n100 + result.misses, 150);
    }

    #[test]
    fn test_n100_provided_n300_missing() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 200,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85),
            n300: None,
            n100: Some(50),
            misses: Some(15),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 135); // 200 - 50 - 15
        assert_eq!(result.n100, 50);
        assert_eq!(result.misses, 15);
        assert_eq!(result.n300 + result.n100 + result.misses, 200);
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

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 0);
        assert_eq!(result.misses, 0);

        let actual_acc = result.accuracy();
        assert_eq!(actual_acc, 1.0);
    }

    #[test]
    fn test_both_missing_high_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 500,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.95),
            n300: None,
            n100: None,
            misses: Some(10),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 500);
        assert_eq!(result.misses, 10);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - 0.95).abs() < 0.001,
            "Expected ~0.95, got {actual_acc}",
        );
    }

    #[test]
    fn test_both_missing_medium_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 400,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.75),
            n300: None,
            n100: None,
            misses: Some(20),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 400);
        assert_eq!(result.misses, 20);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - 0.75).abs() < 0.001,
            "Expected ~0.75, got {actual_acc}",
        );
    }

    #[test]
    fn test_both_missing_low_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 300,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.60),
            n300: None,
            n100: None,
            misses: Some(50),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 300);
        assert_eq!(result.misses, 50);
        // At 60% accuracy, we should have more n100s than n300s
        assert!(result.n100 > 0);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - 0.60).abs() < 0.002,
            "Expected ~0.60, got {actual_acc}",
        );
    }

    #[test]
    fn test_all_misses() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.0),
            n300: None,
            n100: None,
            misses: Some(100),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.misses, 100);
        assert_eq!(result.accuracy(), 0.0);
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

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        // Should clamp misses to total_hits
        assert_eq!(result.misses, 50);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
    }

    #[test]
    fn test_clamping_n300_exceeds_remain() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n300: Some(200), // More than total_hits
            n100: Some(50),
            misses: Some(10),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        assert!(result.n300 <= 90); // Can't exceed remain
    }

    #[test]
    fn test_clamping_n100_exceeds_remain() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85),
            n300: None,
            n100: Some(200), // More than total_hits
            misses: Some(15),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        assert!(result.n100 <= 85); // Can't exceed remain
    }

    #[test]
    fn test_50_percent_accuracy() {
        let inspect = InspectTaikoPerformance {
            attrs: &TaikoDifficultyAttributes {
                max_combo: 200,
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

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 200);
        assert_eq!(result.misses, 0);

        // At 50% accuracy with no misses, we should have all n100s
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 200);
        assert_eq!(result.accuracy(), 0.5);
    }

    #[test]
    fn test_accuracy_calculation_consistency() {
        // Verify that the Fast algorithm produces results that match the accuracy formula
        // Note: With 5 misses out of 100, max accuracy is (2*95)/(2*100) = 0.95
        for acc in [0.55, 0.65, 0.75, 0.85, 0.90, 0.93] {
            let inspect = InspectTaikoPerformance {
                attrs: &TaikoDifficultyAttributes {
                    max_combo: 100,
                    ..Default::default()
                },
                difficulty: &Difficulty::new(),
                acc: Some(acc),
                n300: None,
                n100: None,
                misses: Some(5),
                combo: None,
                hitresult_priority: HitResultPriority::BestCase,
            };

            let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);
            let actual_acc = result.accuracy();

            assert!(
                (actual_acc - acc).abs() < 0.002,
                "For target acc {acc}, got {actual_acc} (n300={}, n100={})",
                result.n300,
                result.n100
            );
        }
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

        let result = <Fast as HitResultGenerator<Taiko>>::generate_hitresults(inspect);

        assert_eq!(result.n300 + result.n100 + result.misses, 10);
        assert_eq!(result.misses, 1);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - 0.80).abs() < 0.02,
            "Expected ~0.80, got {actual_acc}",
        );
    }
}
