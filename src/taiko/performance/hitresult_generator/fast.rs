use std::cmp;

use crate::{
    any::{HitResultGenerator, hitresult_generator::Fast},
    taiko::{TaikoHitResults, performance::hitresult_generator::TaikoHitResultParams},
};

impl HitResultGenerator<TaikoHitResultParams> for Fast {
    fn generate_hitresults(params: &TaikoHitResultParams) -> TaikoHitResults {
        let misses = cmp::min(params.misses, params.total_hits);
        let remain = params.total_hits - misses;

        let (n300, n100) = match (params.n300, params.n100) {
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

                let target_total = f64::round(params.acc * f64::from(2 * params.total_hits)) as u32;

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
    use super::*;

    #[test]
    fn test_both_provided() {
        let params = TaikoHitResultParams {
            total_hits: 100,
            acc: 0.95,
            n300: Some(90),
            n100: Some(10),
            misses: 0,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 90);
        assert_eq!(result.n100, 10);
        assert_eq!(result.misses, 0);
        assert_eq!(result.n300 + result.n100 + result.misses, 100);
    }

    #[test]
    fn test_n300_provided_n100_missing() {
        let params = TaikoHitResultParams {
            total_hits: 150,
            acc: 0.90,
            n300: Some(120),
            n100: None,
            misses: 10,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 120);
        assert_eq!(result.n100, 20); // 150 - 120 - 10
        assert_eq!(result.misses, 10);
        assert_eq!(result.n300 + result.n100 + result.misses, 150);
    }

    #[test]
    fn test_n100_provided_n300_missing() {
        let params = TaikoHitResultParams {
            total_hits: 200,
            acc: 0.85,
            n300: None,
            n100: Some(50),
            misses: 15,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 135); // 200 - 50 - 15
        assert_eq!(result.n100, 50);
        assert_eq!(result.misses, 15);
        assert_eq!(result.n300 + result.n100 + result.misses, 200);
    }

    #[test]
    fn test_both_missing_perfect_accuracy() {
        let params = TaikoHitResultParams {
            total_hits: 100,
            acc: 1.0,
            n300: None,
            n100: None,
            misses: 0,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 0);
        assert_eq!(result.misses, 0);

        let actual_acc = result.accuracy();
        assert_eq!(actual_acc, 1.0);
    }

    #[test]
    fn test_both_missing_high_accuracy() {
        let params = TaikoHitResultParams {
            total_hits: 500,
            acc: 0.95,
            n300: None,
            n100: None,
            misses: 10,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = TaikoHitResultParams {
            total_hits: 400,
            acc: 0.75,
            n300: None,
            n100: None,
            misses: 20,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = TaikoHitResultParams {
            total_hits: 300,
            acc: 0.60,
            n300: None,
            n100: None,
            misses: 50,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = TaikoHitResultParams {
            total_hits: 100,
            acc: 0.0,
            n300: None,
            n100: None,
            misses: 100,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.misses, 100);
        assert_eq!(result.accuracy(), 0.0);
    }

    #[test]
    fn test_edge_case_more_misses_than_hits() {
        let params = TaikoHitResultParams {
            total_hits: 50,
            acc: 0.5,
            n300: None,
            n100: None,
            misses: 100, // More than total_hits
        };

        let result = Fast::generate_hitresults(&params);

        // Should clamp misses to total_hits
        assert_eq!(result.misses, 50);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
    }

    #[test]
    fn test_clamping_n300_exceeds_remain() {
        let params = TaikoHitResultParams {
            total_hits: 100,
            acc: 0.90,
            n300: Some(200), // More than total_hits
            n100: Some(50),
            misses: 10,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        assert!(result.n300 <= 90); // Can't exceed remain
    }

    #[test]
    fn test_clamping_n100_exceeds_remain() {
        let params = TaikoHitResultParams {
            total_hits: 100,
            acc: 0.85,
            n300: None,
            n100: Some(200), // More than total_hits
            misses: 15,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300 + result.n100 + result.misses, 100);
        assert!(result.n100 <= 85); // Can't exceed remain
    }

    #[test]
    fn test_50_percent_accuracy() {
        let params = TaikoHitResultParams {
            total_hits: 200,
            acc: 0.50,
            n300: None,
            n100: None,
            misses: 0,
        };

        let result = Fast::generate_hitresults(&params);

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
            let params = TaikoHitResultParams {
                total_hits: 100,
                acc,
                n300: None,
                n100: None,
                misses: 5,
            };

            let result = Fast::generate_hitresults(&params);
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
        let params = TaikoHitResultParams {
            total_hits: 10,
            acc: 0.80,
            n300: None,
            n100: None,
            misses: 1,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300 + result.n100 + result.misses, 10);
        assert_eq!(result.misses, 1);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - 0.80).abs() < 0.02,
            "Expected ~0.80, got {actual_acc}",
        );
    }
}
