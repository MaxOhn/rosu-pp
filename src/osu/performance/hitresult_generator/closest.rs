use std::cmp;

use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Closest, IgnoreAccuracy},
    },
    osu::{InspectOsuPerformance, Osu, OsuHitResults},
};

impl HitResultGenerator<Osu> for Closest {
    #[expect(clippy::too_many_lines, reason = "it's pretty clean though")]
    fn generate_hitresults(inspect: InspectOsuPerformance) -> OsuHitResults {
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

        let (tick_score, tick_max) =
            origin.tick_scores(large_tick_hits, small_tick_hits, slider_end_hits);

        let target_total = acc * f64::from(300 * total_hits + tick_max);

        let compute_n100_n50 = |n300| {
            //     target_total = 300*n300 + 100*n100 + 50*n50 + tick_score
            // <=> target_total - 300*n300 - 50*n50 - tick_score = 100*n100
            // <=> target_total - 300*n300 - 50*n100 - 50*n50 - tick_score = 50*n100
            // <=> target_total - 250*n300 - 50*remain - tick_score) / 50 = n100

            let n300 = cmp::min(n300, remain);

            let raw100 = (target_total - f64::from(50 * remain + 250 * n300 + tick_score)) / 50.0;

            let remain = remain - n300;
            let min100 = cmp::min(remain, f64::floor(raw100) as u32);
            let max100 = cmp::min(remain, f64::ceil(raw100) as u32);

            let mut best_dist = f64::MAX;
            let mut n100 = 0;
            let mut n50 = remain;

            for new100 in min100..=max100 {
                let new50 = remain - new100;

                let state = OsuHitResults {
                    large_tick_hits,
                    small_tick_hits,
                    slider_end_hits,
                    n300,
                    n100: new100,
                    n50: new50,
                    misses,
                };

                let dist = f64::abs(acc - state.accuracy(origin));

                if dist < best_dist {
                    best_dist = dist;
                    n100 = new100;
                    n50 = new50;
                }
            }

            (n300, n100, n50)
        };

        let compute_n300_n50 = |n100| {
            //     target_total = 300*n300 + 100*n100 + 50*n50 + tick_score
            // <=> target_total - 100*n100 - 50*n50 - tick_score = 300*n300
            // <=> target_total - 50n300 - 100*n100 - 50*n50 - tick_score = 250*n300
            // <=> (target_total - 50*remain - 50*n100 - tick_score) / 250 = n300

            let n100 = cmp::min(n100, remain);

            let raw300 = (target_total - f64::from(50 * remain + 50 * n100 + tick_score)) / 250.0;

            let remain = remain - n100;
            let min300 = cmp::min(remain, f64::floor(raw300) as u32);
            let max300 = cmp::min(remain, f64::ceil(raw300) as u32);

            let mut best_dist = f64::MAX;
            let mut n300 = 0;
            let mut n50 = remain;

            for new300 in min300..=max300 {
                let new50 = remain - new300;

                let state = OsuHitResults {
                    large_tick_hits,
                    small_tick_hits,
                    slider_end_hits,
                    n300: new300,
                    n100,
                    n50: new50,
                    misses,
                };

                let dist = f64::abs(acc - state.accuracy(origin));

                if dist < best_dist {
                    best_dist = dist;
                    n300 = new300;
                    n50 = new50;
                }
            }

            (n300, n100, n50)
        };

        let compute_n300_n100 = |n50| {
            //     target_total = 300*n300 + 100*n100 + 50*n50 + tick_score
            // <=> target_total - 100*n100 - 50*n50 - tick_score = 300*n300
            // <=> target_total - 100n300 - 100*n100 - 50*n50 - tick_score = 200*n300
            // <=> (target_total - 100*remain + 50*n50 - tick_score) / 200 = n300

            let n50 = cmp::min(n50, remain);

            let raw300 =
                (target_total + f64::from(50 * n50) - f64::from(100 * remain + tick_score)) / 200.0;

            let remain = remain - n50;
            let min300 = cmp::min(remain, f64::floor(raw300) as u32);
            let max300 = cmp::min(remain, f64::ceil(raw300) as u32);

            let mut best_dist = f64::MAX;
            let mut n300 = 0;
            let mut n100 = remain;

            for new300 in min300..=max300 {
                let new100 = remain - new300;

                let state = OsuHitResults {
                    large_tick_hits,
                    small_tick_hits,
                    slider_end_hits,
                    n300: new300,
                    n100: new100,
                    n50,
                    misses,
                };

                let dist = f64::abs(acc - state.accuracy(origin));

                if dist < best_dist {
                    best_dist = dist;
                    n300 = new300;
                    n100 = new100;
                }
            }

            (n300, n100, n50)
        };

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

            // Two missing
            (Some(n300), None, None) => compute_n100_n50(n300),
            (None, Some(n100), None) => compute_n300_n50(n100),
            (None, None, Some(n50)) => compute_n300_n100(n50),

            // All three missing
            (None, None, None) => {
                // Deriving bounds on n300:
                // - Lower bound: minimize n300 by maximizing n50 (n100 = 0)
                //       target_total = 300*n300 + 100*n100 + 50*n50 + tick_score
                //   <=> target_total = 300*n300 + 50*(remaing - n300) + tick_score
                //   <=> target_total - 50*remain - tick_score = 250*n300
                //   <=> (target_total - 50*remain - tick_score) / 250 = n300
                let raw_min300 = (target_total - f64::from(50 * remain + tick_score)) / 250.0;

                // - Upper bound: maximize n300 by minimizing n100 and n50 (both = 0)
                //       target_total = 300*n300 + 100*n100 + 50*n50 + tick_score
                //   <=> (target_total - tick_score) / 300 = n300
                let raw_max300 = (target_total - f64::from(tick_score)) / 300.0;

                let min300 = f64::floor(raw_min300).max(0.0) as u32;
                // 1+ to account for potential floating point inaccuracies
                let max300 = cmp::min(remain, 1 + f64::ceil(raw_max300) as u32);

                let mut best_dist = f64::MAX;
                let mut n300 = 0;
                let mut n100 = 0;
                let mut n50 = remain;

                for new300 in min300..=max300 {
                    let (new300, new100, new50) = compute_n100_n50(new300);

                    let state = OsuHitResults {
                        large_tick_hits,
                        small_tick_hits,
                        slider_end_hits,
                        n300: new300,
                        n100: new100,
                        n50: new50,
                        misses,
                    };

                    let dist = f64::abs(acc - state.accuracy(origin));

                    if dist < best_dist {
                        best_dist = dist;
                        n300 = new300;
                        n100 = new100;
                        n50 = new50;
                    }
                }

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

    use crate::{Difficulty, any::HitResultPriority, osu::OsuDifficultyAttributes};

    use super::*;

    // Helper function to verify that the result is the closest possible
    fn verify_is_closest(inspect: &InspectOsuPerformance, result: &OsuHitResults) {
        let acc = inspect.acc.unwrap();

        let total_hits = inspect.total_hits();
        let origin = inspect.origin();

        let result_acc = result.accuracy(origin);
        let result_dist = f64::abs(acc - result_acc);

        let remain = total_hits - result.misses;

        // Check all possible combinations of n300, n100, n50
        for n300 in 0..=remain {
            for n100 in 0..=(remain - n300) {
                let n50 = remain - n300 - n100;

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

                if let Some(expected_n50) = inspect.n50 {
                    if n50 != expected_n50 {
                        continue;
                    }
                }

                let candidate = OsuHitResults {
                    large_tick_hits: result.large_tick_hits,
                    small_tick_hits: result.small_tick_hits,
                    slider_end_hits: result.slider_end_hits,
                    n300,
                    n100,
                    n50,
                    misses: result.misses,
                };

                let candidate_acc = candidate.accuracy(origin);
                let candidate_dist = f64::abs(acc - candidate_acc);

                assert!(
                    result_dist <= candidate_dist + 1e-10, // Small epsilon for floating point
                    "Found closer result: result has distance {result_dist}, \
                    but ({n300}, {n100}, {n50}) has distance {candidate_dist}",
                );
            }
        }
    }

    #[test]
    fn test_none_missing_all_provided() {
        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: 100,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.95),
            n300: Some(90),
            n100: Some(8),
            n50: Some(2),
            misses: Some(0),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn test_one_missing_n50() {
        const N_CIRCLES: u32 = 50;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.95),
            n300: Some(45),
            n100: Some(3),
            n50: None,
            misses: Some(2),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_two_missing_n100_n50_given_n300() {
        const N_CIRCLES: u32 = 80;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.90),
            n300: Some(60),
            n100: None,
            n50: None,
            misses: Some(5),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_two_missing_n300_n50_given_n100() {
        const N_CIRCLES: u32 = 70;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.85),
            n300: None,
            n100: Some(15),
            n50: None,
            misses: Some(8),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_two_missing_n300_n100_given_n50() {
        const N_CIRCLES: u32 = 60;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.80),
            n300: None,
            n100: None,
            n50: Some(12),
            misses: Some(6),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_missing_high_accuracy() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.98),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(2),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_missing_medium_accuracy() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.75),
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

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_missing_perfect_accuracy() {
        const N_CIRCLES: u32 = 50;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
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

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, N_CIRCLES);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_with_slider_acc_all_missing() {
        const N_CIRCLES: u32 = 80;
        const N_SLIDERS: u32 = 15;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                n_sliders: N_SLIDERS,
                n_large_ticks: 20,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.96),
            n300: None,
            n100: None,
            n50: None,
            misses: Some(2),
            large_tick_hits: Some(20),
            small_tick_hits: None,
            slider_end_hits: Some(15),
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES + N_SLIDERS
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_without_slider_acc_two_missing() {
        const N_CIRCLES: u32 = 70;
        const N_SLIDERS: u32 = 15;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                n_sliders: N_SLIDERS,
                n_large_ticks: 10,
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
            acc: Some(0.88),
            n300: Some(50),
            n100: None,
            n50: None,
            misses: Some(5),
            large_tick_hits: Some(15),
            small_tick_hits: Some(25),
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES + N_SLIDERS
        );
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_clamping_when_values_exceed_remain() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.90),
            n300: Some(200), // More than total_hits
            n100: Some(50),
            n50: Some(30),
            misses: Some(10),
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            combo: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert!(result.n300 <= 90);
    }

    #[test]
    fn test_edge_case_low_accuracy_many_50s() {
        const N_CIRCLES: u32 = 60;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.55),
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

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300 + result.n100 + result.n50 + result.misses,
            N_CIRCLES
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_misses() {
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

        let result = <Closest as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }
}
