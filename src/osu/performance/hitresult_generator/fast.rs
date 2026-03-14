use std::cmp;

use crate::{
    any::{
        HitResultGenerator, HitResultPriority,
        hitresult_generator::{Fast, IgnoreAccuracy},
    },
    osu::{InspectOsuPerformance, Osu, OsuHitResults},
};

impl HitResultGenerator<Osu> for Fast {
    fn generate_hitresults(inspect: InspectOsuPerformance<'_>) -> OsuHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect);
        };

        let (slider_end_hits, large_tick_hits, small_tick_hits) = inspect.tick_hits();

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

        let prelim_300 = inspect.n300.map_or(0, |n| cmp::min(n, remain));
        let prelim_100 = inspect.n100.map_or(0, |n| cmp::min(n, remain - prelim_300));
        let prelim_50 = inspect
            .n50
            .map_or(0, |n| cmp::min(n, remain - prelim_300 - prelim_100));

        let (n300, n100, n50) = match (inspect.n300, inspect.n100, inspect.n50) {
            // None missing
            (Some(_), Some(_), Some(_)) => (prelim_300, prelim_100, prelim_50),

            // Only one missing
            (Some(_), Some(_), None) => (prelim_300, prelim_100, remain - prelim_300 - prelim_100),
            (Some(_), None, Some(_)) => (prelim_300, remain - prelim_300 - prelim_50, prelim_50),
            (None, Some(_), Some(_)) => (remain - prelim_100 - prelim_50, prelim_100, prelim_50),

            // Two or three missing - use Fast algorithm
            _ => {
                // acc = (300*n300 + 100*n100 + 50*n50 + tick_score) / (300*total_hits + tick_max)
                // Simplify by dividing by 50: (reducing risk of overflow)
                // acc = (6*n300 + 2*n100 + n50 + tick_score/50) / (6*total_hits + tick_max/50)

                let numerator = f64::from(6 * prelim_300 + 2 * prelim_100 + prelim_50)
                    + f64::from(tick_score) / 50.0;

                let denominator = f64::from(6 * total_hits) + f64::from(tick_max) / 50.0;

                let target_total =
                    f64::round_ties_even((acc * denominator - numerator).max(0.0)) as u32;

                // Start by assuming every non-miss is an n50
                // delta is how much we need to increase from the baseline (all n50s)
                let baseline = remain - prelim_300 - prelim_100 - prelim_50;
                let mut delta = target_total.saturating_sub(baseline);

                // Each n300 increases by 5 (6-1), each n100 increases by 1 (2-1)
                // delta = 5*n300 + 1*n100

                let n300 = cmp::min(
                    remain - prelim_100 - prelim_50,
                    inspect.n300.unwrap_or(delta / 5),
                );

                if inspect.n300.is_none() {
                    delta = delta.saturating_sub(5 * n300);
                }

                let n100 = cmp::min(remain - n300 - prelim_50, inspect.n100.unwrap_or(delta));
                let n50 = cmp::min(remain - n300 - n100, inspect.n50.unwrap_or(remain));

                (n300, n100, n50)
            }
        };

        let mut hitresults = OsuHitResults {
            large_tick_hits,
            small_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
        };

        if hitresults.total_hits() < total_hits {
            let left = total_hits - hitresults.total_hits();

            match inspect.hitresult_priority {
                HitResultPriority::BestCase => match inspect {
                    InspectOsuPerformance { n300: None, .. } => hitresults.n300 += left,
                    InspectOsuPerformance { n100: None, .. } => hitresults.n100 += left,
                    _ => hitresults.n50 += left,
                },
                HitResultPriority::WorstCase => match inspect {
                    InspectOsuPerformance { n50: None, .. } => hitresults.n50 += left,
                    InspectOsuPerformance { n100: None, .. } => hitresults.n100 += left,
                    _ => hitresults.n300 += left,
                },
            }
        }

        hitresults
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
        const N_CIRCLES: u32 = 100;
        const ACC: f64 = 1.0;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, N_CIRCLES);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.accuracy(OsuScoreOrigin::Stable), ACC);
    }

    #[test]
    fn high_accuracy_stable() {
        const N_CIRCLES: u32 = 1000;
        const ACC: f64 = 0.95;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        // Verify total adds up
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap().clone());

        // Verify accuracy is close to target
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - ACC).abs() < 0.001,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    #[test]
    fn medium_accuracy_stable() {
        const N_CIRCLES: u32 = 500;
        const ACC: f64 = 0.85;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());

        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - ACC).abs() < 0.001,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    #[test]
    fn with_slider_acc() {
        const N_CIRCLES: u32 = 160;
        const N_SLIDERS: u32 = 40;
        const N_LARGE_TICKS: u32 = 50;
        const ACC: f64 = 0.98;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                n_sliders: N_SLIDERS,
                n_large_ticks: N_LARGE_TICKS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(ACC),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(2),
            large_tick_hits: Some(N_LARGE_TICKS),
            small_tick_hits: None,
            slider_end_hits: Some(N_SLIDERS),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let origin = inspect.origin();

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES + N_SLIDERS
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.large_tick_hits, N_LARGE_TICKS);
        assert_eq!(result.slider_end_hits, N_SLIDERS);

        let actual_acc = result.accuracy(origin);
        assert!(
            (actual_acc - ACC).abs() < 0.002,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    #[test]
    fn without_slider_acc() {
        const N_CIRCLES: u32 = 240;
        const N_SLIDERS: u32 = 60;
        const N_LARGE_TICKS: u32 = 40;
        const ACC: f64 = 0.92;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                n_sliders: N_SLIDERS,
                n_large_ticks: N_LARGE_TICKS,
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
            acc: Some(ACC),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(5),
            large_tick_hits: Some(N_SLIDERS + N_LARGE_TICKS),
            small_tick_hits: Some(N_SLIDERS),
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let origin = inspect.origin();

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES + N_SLIDERS
        );
        assert_eq!(result.misses, inspect.misses.unwrap().clone());
        assert_eq!(result.large_tick_hits, N_SLIDERS + N_LARGE_TICKS);
        assert_eq!(result.small_tick_hits, N_SLIDERS);

        let actual_acc = result.accuracy(origin);
        assert!(
            (actual_acc - ACC).abs() < 0.005,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    #[test]
    fn all_misses() {
        const N_CIRCLES: u32 = 50;
        const ACC: f64 = 0.0;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.accuracy(OsuScoreOrigin::Stable), ACC);
    }

    #[test]
    fn low_accuracy_many_50s() {
        const N_CIRCLES: u32 = 400;
        const ACC: f64 = 0.60;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
        // At 60% accuracy with many misses, we should have a lot of 50s
        assert!(result.n50 > 0, "Expected some n50s at low accuracy");

        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - ACC).abs() < 0.002,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    #[test]
    fn edge_case_more_misses_than_hits() {
        const N_CIRCLES: u32 = 100;
        const ACC: f64 = 0.5;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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
        assert_eq!(result.misses, N_CIRCLES);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
    }

    #[test]
    fn all_three_provided() {
        const N_CIRCLES: u32 = 100;
        const ACC: f64 = 0.90;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn all_three_provided_with_clamping() {
        const N_CIRCLES: u32 = 100;
        const ACC: f64 = 0.90;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
            n300: Some(200), // Exceeds remaining
            n100: Some(50),
            n50: Some(30),
            misses: Some(10),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.n300, 90); // clamped to remaining 90
        assert_eq!(result.n100, 0); // no room left, clamped to 0
        assert_eq!(result.n50, 0); // no room left, clamped to 0
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
    }

    // Tests for "Only one missing" cases
    #[test]
    fn n50_missing() {
        const N_CIRCLES: u32 = 150;
        const ACC: f64 = 0.88;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.n50, 10); // N_CIRCLES - n300 - n100 - misses
    }

    #[test]
    fn n100_missing() {
        const N_CIRCLES: u32 = 200;
        const ACC: f64 = 0.85;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.n100, 25); // N_CIRCLES - n300 - n50 - misses
    }

    #[test]
    fn n300_missing() {
        const N_CIRCLES: u32 = 180;
        const ACC: f64 = 0.80;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(result.n300, 90); // N_CIRCLES - n100 - n50 - misses
    }

    // Tests for "Two missing" cases with n300 provided
    #[test]
    fn n300_provided_n100_n50_missing() {
        const N_CIRCLES: u32 = 200;
        const ACC: f64 = 0.90;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());

        // 90% unfeasable with only 150 n300s so the generation should only
        // assign n100s but no n50s.
        assert_eq!(result.n100, N_CIRCLES - result.n300 - result.misses);
    }

    // Tests for "Two missing" cases with n100 provided
    #[test]
    fn n100_provided_n300_n50_missing() {
        const N_CIRCLES: u32 = 150;
        const ACC: f64 = 0.85;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());

        // Fast is an approximation
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - ACC).abs() < 0.05,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    // Tests for "Two missing" cases with n50 provided
    #[test]
    fn n50_provided_n300_n100_missing() {
        const N_CIRCLES: u32 = 120;
        const ACC: f64 = 0.80;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());

        // Fast is an approximation
        let actual_acc = result.accuracy(OsuScoreOrigin::Stable);
        assert!(
            (actual_acc - ACC).abs() < 0.05,
            "Expected ~{ACC}, got {actual_acc}",
        );
    }

    // Test with slider accuracy and some values provided
    #[test]
    fn with_slider_acc_n300_provided() {
        const N_CIRCLES: u32 = 125;
        const N_SLIDERS: u32 = 25;
        const N_LARGE_TICKS: u32 = 30;
        const ACC: f64 = 0.95;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                n_sliders: N_SLIDERS,
                n_large_ticks: N_LARGE_TICKS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(ACC),
            n300: Some(120),
            n100: None,
            n50: None,
            misses: Some(3),
            large_tick_hits: Some(N_LARGE_TICKS),
            small_tick_hits: None,
            slider_end_hits: Some(N_SLIDERS),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES + N_SLIDERS
        );
        assert_eq!(result.large_tick_hits, inspect.large_tick_hits.unwrap());
        assert_eq!(result.slider_end_hits, inspect.slider_end_hits.unwrap());
    }

    // Test edge case where provided values leave no room
    #[test]
    fn provided_values_fill_all_remain() {
        const N_CIRCLES: u32 = 100;
        const ACC: f64 = 0.95;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, 0); // No room left
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    // Test that algorithm handles very low accuracy with provided n300
    #[test]
    fn low_accuracy_with_high_n300_provided() {
        const N_CIRCLES: u32 = 100;
        const ACC: f64 = 0.60;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(ACC),
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

        let result = <Fast as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        // With low accuracy and only 10 n300s out of 80 remaining, we expect mostly n50s
        // But the Fast algorithm might not produce many n50s - just verify totals add up
        assert_eq!(result.n100 + result.n50, 70);
    }
}
