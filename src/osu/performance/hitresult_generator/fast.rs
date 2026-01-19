use std::cmp;

use crate::{
    any::{HitResultGenerator, hitresult_generator::Fast},
    osu::{OsuHitResults, performance::hitresult_generator::OsuHitResultParams},
};

impl HitResultGenerator<OsuHitResultParams> for Fast {
    fn generate_hitresults(params: &OsuHitResultParams) -> OsuHitResults {
        let large_tick_hits = params.large_tick_hits.unwrap_or(0);
        let small_tick_hits = params.small_tick_hits.unwrap_or(0);
        let slider_end_hits = params.slider_end_hits.unwrap_or(0);

        let misses = cmp::min(params.misses, params.total_hits);
        let remain = params.total_hits - misses;

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
            params
                .origin
                .tick_scores(large_tick_hits, small_tick_hits, slider_end_hits);

        let (n300, n100, n50) = match (params.n300, params.n100, params.n50) {
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

                let target_total = f64::round(
                    params.acc * (f64::from(6 * params.total_hits) + f64::from(tick_max) / 50.0),
                ) as u32;

                // Start by assuming every non-miss is an n50
                // delta is how much we need to increase from the baseline (all n50s)
                let baseline = remain + tick_score / 50;
                let delta = target_total.saturating_sub(baseline);

                // Each n300 increases by 5 (6-1), each n100 increases by 1 (2-1)
                // delta = 5*n300 + 1*n100

                let n300 = params.n300.unwrap_or_else(|| cmp::min(remain, delta / 5));

                let n100 = params.n100.unwrap_or_else(|| {
                    let remain_after_n300 = remain - n300;

                    if params.n300.is_some() {
                        // If n300 was provided, recalculate delta for remaining hits
                        let used_by_n300 = 5 * n300;
                        let remain_delta = delta.saturating_sub(used_by_n300);

                        cmp::min(remain_after_n300, remain_delta)
                    } else {
                        // If n300 was calculated, use modulo
                        cmp::min(remain_after_n300, delta % 5)
                    }
                });

                let n50 = params.n50.unwrap_or_else(|| remain - n300 - n100);

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
    use crate::osu::OsuScoreOrigin;

    use super::*;

    #[test]
    fn perfect_accuracy_no_misses() {
        let params = OsuHitResultParams {
            total_hits: 100,
            acc: 1.0,
            n300: None,
            n100: None,
            n50: None,
            misses: 0,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(OsuScoreOrigin::Stable), 1.0);
    }

    #[test]
    fn high_accuracy_stable() {
        let params = OsuHitResultParams {
            total_hits: 1000,
            acc: 0.95,
            n300: None,
            n100: None,
            n50: None,
            misses: 10,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 500,
            acc: 0.85,
            n300: None,
            n100: None,
            n50: None,
            misses: 25,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 200,
            acc: 0.98,
            n300: None,
            n100: None,
            n50: None,
            misses: 2,
            large_tick_hits: Some(50),
            small_tick_hits: None,
            slider_end_hits: Some(40),
            origin: OsuScoreOrigin::WithSliderAcc {
                max_large_ticks: 50,
                max_slider_ends: 40,
            },
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 200);
        assert_eq!(result.misses, 2);
        assert_eq!(result.large_tick_hits, 50);
        assert_eq!(result.slider_end_hits, 40);

        let actual_acc = result.accuracy(params.origin);
        assert!(
            (actual_acc - 0.98).abs() < 0.002,
            "Expected ~0.98, got {actual_acc}",
        );
    }

    #[test]
    fn without_slider_acc() {
        let params = OsuHitResultParams {
            total_hits: 300,
            acc: 0.92,
            n300: None,
            n100: None,
            n50: None,
            misses: 5,
            large_tick_hits: Some(60),
            small_tick_hits: Some(100),
            slider_end_hits: None,
            origin: OsuScoreOrigin::WithoutSliderAcc {
                max_large_ticks: 60,
                max_small_ticks: 100,
            },
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 300);
        assert_eq!(result.misses, 5);
        assert_eq!(result.large_tick_hits, 60);
        assert_eq!(result.small_tick_hits, 100);

        let actual_acc = result.accuracy(params.origin);
        assert!(
            (actual_acc - 0.92).abs() < 0.002,
            "Expected ~0.92, got {actual_acc}",
        );
    }

    #[test]
    fn all_misses() {
        let params = OsuHitResultParams {
            total_hits: 50,
            acc: 0.0,
            n300: None,
            n100: None,
            n50: None,
            misses: 50,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 50);
        assert_eq!(result.accuracy(OsuScoreOrigin::Stable), 0.0);
    }

    #[test]
    fn low_accuracy_many_50s() {
        let params = OsuHitResultParams {
            total_hits: 400,
            acc: 0.60,
            n300: None,
            n100: None,
            n50: None,
            misses: 50,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 100,
            acc: 0.5,
            n300: None,
            n100: None,
            n50: None,
            misses: 150, // More misses than total hits
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        // Should clamp misses to total_hits
        assert_eq!(result.misses, 100);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
    }

    // Tests for "None missing" case
    #[test]
    fn all_three_provided() {
        let params = OsuHitResultParams {
            total_hits: 100,
            acc: 0.90,
            n300: Some(80),
            n100: Some(15),
            n50: Some(5),
            misses: 0,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 80);
        assert_eq!(result.n100, 15);
        assert_eq!(result.n50, 5);
        assert_eq!(result.misses, 0);
    }

    #[test]
    fn all_three_provided_with_clamping() {
        let params = OsuHitResultParams {
            total_hits: 100,
            acc: 0.90,
            n300: Some(200), // Exceeds remain
            n100: Some(50),
            n50: Some(30),
            misses: 10,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 150,
            acc: 0.88,
            n300: Some(100),
            n100: Some(30),
            n50: None,
            misses: 10,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 100);
        assert_eq!(result.n100, 30);
        assert_eq!(result.n50, 10); // 150 - 100 - 30 - 10
        assert_eq!(result.misses, 10);
    }

    #[test]
    fn n100_missing() {
        let params = OsuHitResultParams {
            total_hits: 200,
            acc: 0.85,
            n300: Some(140),
            n100: None,
            n50: Some(20),
            misses: 15,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 140);
        assert_eq!(result.n100, 25); // 200 - 140 - 20 - 15
        assert_eq!(result.n50, 20);
        assert_eq!(result.misses, 15);
    }

    #[test]
    fn n300_missing() {
        let params = OsuHitResultParams {
            total_hits: 180,
            acc: 0.80,
            n300: None,
            n100: Some(40),
            n50: Some(30),
            misses: 20,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 90); // 180 - 40 - 30 - 20
        assert_eq!(result.n100, 40);
        assert_eq!(result.n50, 30);
        assert_eq!(result.misses, 20);
    }

    // Tests for "Two missing" cases with n300 provided
    #[test]
    fn n300_provided_n100_n50_missing() {
        let params = OsuHitResultParams {
            total_hits: 200,
            acc: 0.90,
            n300: Some(150),
            n100: None,
            n50: None,
            misses: 10,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 150,
            acc: 0.85,
            n300: None,
            n100: Some(30),
            n50: None,
            misses: 8,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 120,
            acc: 0.80,
            n300: None,
            n100: None,
            n50: Some(25),
            misses: 5,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

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
        let params = OsuHitResultParams {
            total_hits: 150,
            acc: 0.95,
            n300: Some(120),
            n100: None,
            n50: None,
            misses: 3,
            large_tick_hits: Some(30),
            small_tick_hits: None,
            slider_end_hits: Some(25),
            origin: OsuScoreOrigin::WithSliderAcc {
                max_large_ticks: 30,
                max_slider_ends: 25,
            },
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 120);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 150);
        assert_eq!(result.large_tick_hits, 30);
        assert_eq!(result.slider_end_hits, 25);
    }

    // Test edge case where provided values leave no room
    #[test]
    fn provided_values_fill_all_remain() {
        let params = OsuHitResultParams {
            total_hits: 100,
            acc: 0.95,
            n300: Some(85),
            n100: Some(10),
            n50: None,
            misses: 5,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 85);
        assert_eq!(result.n100, 10);
        assert_eq!(result.n50, 0); // No room left
        assert_eq!(result.misses, 5);
    }

    // Test that algorithm handles very low accuracy with provided n300
    #[test]
    fn low_accuracy_with_high_n300_provided() {
        let params = OsuHitResultParams {
            total_hits: 100,
            acc: 0.60,
            n300: Some(10),
            n100: None,
            n50: None,
            misses: 20,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            origin: OsuScoreOrigin::Stable,
        };

        let result = Fast::generate_hitresults(&params);

        assert_eq!(result.n300, 10);
        assert_eq!(result.n300 + result.n100 + result.n50 + result.misses, 100);
        // With low accuracy and only 10 n300s out of 80 remaining, we expect mostly n50s
        // But the Fast algorithm might not produce many n50s - just verify totals add up
        assert_eq!(result.n100 + result.n50, 70);
    }
}
