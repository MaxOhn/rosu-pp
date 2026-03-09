use std::cmp;

use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Closest, IgnoreAccuracy},
    },
    mania::{InspectManiaPerformance, Mania, ManiaHitResults},
};

impl HitResultGenerator<Mania> for Closest {
    #[expect(clippy::too_many_lines, reason = "it is what it is /shrug")]
    fn generate_hitresults(inspect: InspectManiaPerformance) -> ManiaHitResults {
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

        let is_classic = inspect.is_classic();
        let w320: u32 = if is_classic { 60 } else { 61 };

        // Target numerator for the accuracy formula
        let target_numerator = (acc * f64::from(w320 * total_hits)).round_ties_even() as i64;

        // Handle cases where some hitresults are provided
        match (
            inspect.n320,
            inspect.n300,
            inspect.n200,
            inspect.n100,
            inspect.n50,
        ) {
            // All provided - just clamp and return
            (Some(n320), Some(n300), Some(n200), Some(n100), Some(n50)) => {
                let n320 = cmp::min(n320, remain);
                let n300 = cmp::min(n300, remain - n320);
                let n200 = cmp::min(n200, remain - n320 - n300);
                let n100 = cmp::min(n100, remain - n320 - n300 - n200);
                let n50 = cmp::min(n50, remain - n320 - n300 - n200 - n100);

                ManiaHitResults {
                    n320,
                    n300,
                    n200,
                    n100,
                    n50,
                    misses,
                }
            }

            // Only one missing - calculate it directly
            (Some(n320), Some(n300), Some(n200), Some(n100), None) => {
                let n320 = cmp::min(n320, remain);
                let n300 = cmp::min(n300, remain - n320);
                let n200 = cmp::min(n200, remain - n320 - n300);
                let n100 = cmp::min(n100, remain - n320 - n300 - n200);
                let n50 = remain - n320 - n300 - n200 - n100;

                ManiaHitResults {
                    n320,
                    n300,
                    n200,
                    n100,
                    n50,
                    misses,
                }
            }
            (Some(n320), Some(n300), Some(n200), None, Some(n50)) => {
                let n320 = cmp::min(n320, remain);
                let n300 = cmp::min(n300, remain - n320);
                let n200 = cmp::min(n200, remain - n320 - n300);
                let n50 = cmp::min(n50, remain - n320 - n300 - n200);
                let n100 = remain - n320 - n300 - n200 - n50;

                ManiaHitResults {
                    n320,
                    n300,
                    n200,
                    n100,
                    n50,
                    misses,
                }
            }
            (Some(n320), Some(n300), None, Some(n100), Some(n50)) => {
                let n320 = cmp::min(n320, remain);
                let n300 = cmp::min(n300, remain - n320);
                let n100 = cmp::min(n100, remain - n320 - n300);
                let n50 = cmp::min(n50, remain - n320 - n300 - n100);
                let n200 = remain - n320 - n300 - n100 - n50;

                ManiaHitResults {
                    n320,
                    n300,
                    n200,
                    n100,
                    n50,
                    misses,
                }
            }
            (Some(n320), None, Some(n200), Some(n100), Some(n50)) => {
                let n320 = cmp::min(n320, remain);
                let n200 = cmp::min(n200, remain - n320);
                let n100 = cmp::min(n100, remain - n320 - n200);
                let n50 = cmp::min(n50, remain - n320 - n200 - n100);
                let n300 = remain - n320 - n200 - n100 - n50;

                ManiaHitResults {
                    n320,
                    n300,
                    n200,
                    n100,
                    n50,
                    misses,
                }
            }
            (None, Some(n300), Some(n200), Some(n100), Some(n50)) => {
                let n300 = cmp::min(n300, remain);
                let n200 = cmp::min(n200, remain - n300);
                let n100 = cmp::min(n100, remain - n300 - n200);
                let n50 = cmp::min(n50, remain - n300 - n200 - n100);
                let n320 = remain - n300 - n200 - n100 - n50;

                ManiaHitResults {
                    n320,
                    n300,
                    n200,
                    n100,
                    n50,
                    misses,
                }
            }

            // Two or more missing - use optimization
            _ => optimize_hitresults(
                inspect.n320,
                inspect.n300,
                inspect.n200,
                inspect.n100,
                inspect.n50,
                remain,
                target_numerator,
                w320,
                misses,
            ),
        }
    }
}

/// Optimizes hitresults when 2+ values are missing using a smart search strategy.
///
/// Strategy:
/// 1. For cases with 2 missing values, we analytically solve for one variable
///    and iterate through a small range for the other.
/// 2. For cases with 3+ missing values, we use a hierarchical search:
///    - First, establish bounds on the highest-weighted missing value
///    - For each value in that range, recursively solve for remaining values
///    - Track the best solution found
#[expect(clippy::too_many_arguments, reason = "necessary for optimization")]
fn optimize_hitresults(
    n320: Option<u32>,
    n300: Option<u32>,
    n200: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    remain: u32,
    target_numerator: i64,
    w320: u32,
    misses: u32,
) -> ManiaHitResults {
    // Count how many are provided
    let num_provided = [n320, n300, n200, n100, n50].into_iter().flatten().count();

    match num_provided {
        // 3 provided, 2 missing
        3 => optimize_two_missing(
            n320,
            n300,
            n200,
            n100,
            n50,
            remain,
            target_numerator,
            w320,
            misses,
        ),

        // 2 or fewer provided, 3+ missing
        _ => optimize_three_plus_missing(
            n320,
            n300,
            n200,
            n100,
            n50,
            remain,
            target_numerator,
            w320,
            misses,
        ),
    }
}

/// Optimizes when exactly 2 values are missing.
#[expect(clippy::too_many_arguments)]
fn optimize_two_missing(
    n320: Option<u32>,
    n300: Option<u32>,
    n200: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    remain: u32,
    target_numerator: i64,
    w320: u32,
    misses: u32,
) -> ManiaHitResults {
    // Determine which two are missing and solve analytically
    let provided_sum = [(n320, w320), (n300, 60), (n200, 40), (n100, 20), (n50, 10)]
        .iter()
        .filter_map(|(n, w)| n.map(|val| val * w))
        .sum::<u32>();

    let num_provided = [n320, n300, n200, n100, n50]
        .into_iter()
        .flatten()
        .sum::<u32>();

    let remain_hits = remain - num_provided;
    let remain_score = target_numerator - i64::from(provided_sum);

    // Helper to solve for two variables given their weights
    let solve_two_vars = |w1: i64, w2: i64| -> (u32, u32) {
        // We have: w1*x + w2*y = remaining_score and x + y = remaining_hits
        // Solve for x: x = (remaining_score - w2*remaining_hits) / (w1 - w2)

        if w1 == w2 {
            // If weights are equal, distribute evenly
            let half = remain_hits / 2;

            return (half, remain_hits - half);
        }

        let raw_x = (remain_score - w2 * i64::from(remain_hits)) as f64 / (w1 - w2) as f64;

        // Try floor and ceil to find the best match
        let mut best_dist = f64::MAX;
        let mut best_x = 0;
        let mut best_y = remain_hits;

        for x in [
            0,
            cmp::min(remain_hits, raw_x.floor().max(0.0) as u32),
            cmp::min(remain_hits, raw_x.ceil().max(0.0) as u32),
            remain_hits,
        ] {
            let y = remain_hits - x;
            let score = w1 * i64::from(x) + w2 * i64::from(y);
            let dist = (score - remain_score).abs() as f64;

            if dist < best_dist {
                best_dist = dist;
                best_x = x;
                best_y = y;
            }
        }

        (best_x, best_y)
    };

    // Build result based on which two are missing
    let mut result = ManiaHitResults {
        n320: n320.unwrap_or(0),
        n300: n300.unwrap_or(0),
        n200: n200.unwrap_or(0),
        n100: n100.unwrap_or(0),
        n50: n50.unwrap_or(0),
        misses,
    };

    match (n320, n300, n200, n100, n50) {
        (None, None, _, _, _) => {
            let (x, y) = solve_two_vars(i64::from(w320), 60);
            result.n320 = x;
            result.n300 = y;
        }
        (None, _, None, _, _) => {
            let (x, y) = solve_two_vars(i64::from(w320), 40);
            result.n320 = x;
            result.n200 = y;
        }
        (None, _, _, None, _) => {
            let (x, y) = solve_two_vars(i64::from(w320), 20);
            result.n320 = x;
            result.n100 = y;
        }
        (None, _, _, _, None) => {
            let (x, y) = solve_two_vars(i64::from(w320), 10);
            result.n320 = x;
            result.n50 = y;
        }
        (_, None, None, _, _) => {
            let (x, y) = solve_two_vars(60, 40);
            result.n300 = x;
            result.n200 = y;
        }
        (_, None, _, None, _) => {
            let (x, y) = solve_two_vars(60, 20);
            result.n300 = x;
            result.n100 = y;
        }
        (_, None, _, _, None) => {
            let (x, y) = solve_two_vars(60, 10);
            result.n300 = x;
            result.n50 = y;
        }
        (_, _, None, None, _) => {
            let (x, y) = solve_two_vars(40, 20);
            result.n200 = x;
            result.n100 = y;
        }
        (_, _, None, _, None) => {
            let (x, y) = solve_two_vars(40, 10);
            result.n200 = x;
            result.n50 = y;
        }
        (_, _, _, None, None) => {
            let (x, y) = solve_two_vars(20, 10);
            result.n100 = x;
            result.n50 = y;
        }
        _ => unreachable!("Should have exactly 2 missing"),
    }

    result
}

/// Optimizes when 3 or more values are missing using hierarchical search.
#[expect(clippy::too_many_arguments)]
fn optimize_three_plus_missing(
    n320: Option<u32>,
    n300: Option<u32>,
    n200: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    remain: u32,
    target_numerator: i64,
    w320: u32,
    misses: u32,
) -> ManiaHitResults {
    let mut best_dist = f64::MAX;

    let mut best_result = ManiaHitResults {
        n320: n320.unwrap_or(0),
        n300: n300.unwrap_or(0),
        n200: n200.unwrap_or(0),
        n100: n100.unwrap_or(0),
        n50: n50.unwrap_or(remain),
        misses,
    };

    // Calculate minimum space needed for provided values
    let min_space_needed = n200.unwrap_or(0) + n100.unwrap_or(0) + n50.unwrap_or(0);

    // If n320 is provided, account for it
    let n320_values: Vec<u32> = if let Some(n) = n320 {
        let clamped = cmp::min(n, remain.saturating_sub(min_space_needed));

        vec![clamped]
    } else {
        // Calculate bounds for n320
        let min_raw = (target_numerator - 10 * i64::from(remain)) as f64 / f64::from(w320 - 10);
        let min_val = cmp::min(
            remain.saturating_sub(min_space_needed),
            min_raw.floor().max(0.0) as u32,
        );
        let max_raw = target_numerator as f64 / f64::from(w320);
        let max_val = cmp::min(
            remain.saturating_sub(min_space_needed),
            max_raw.ceil() as u32 + 1,
        );

        (min_val..=max_val).collect::<Vec<_>>()
    };

    for &new320 in &n320_values {
        let used_320 = new320;
        let score_320 = i64::from(w320 * new320);
        let remain_after_320 = remain - used_320;

        // Skip if not enough space for provided values
        if remain_after_320 < min_space_needed {
            continue;
        }

        // If n300 is provided, account for it
        let n300_values: Vec<u32> = if let Some(n) = n300 {
            let clamped = cmp::min(n, remain_after_320.saturating_sub(min_space_needed));

            vec![clamped]
        } else {
            let target_after_320 = target_numerator - score_320;

            let min_raw = (target_after_320 - 10 * i64::from(remain_after_320)) as f64 / 50.0;
            let min_val = cmp::min(
                remain_after_320.saturating_sub(min_space_needed),
                min_raw.floor().max(0.0) as u32,
            );
            let max_raw = target_after_320 as f64 / 60.0;
            let max_val = cmp::min(
                remain_after_320.saturating_sub(min_space_needed),
                max_raw.ceil() as u32 + 1,
            );

            (min_val..=max_val).collect::<Vec<_>>()
        };

        for &new300 in &n300_values {
            let used_300 = used_320 + new300;
            let score_300 = score_320 + i64::from(60 * new300);

            let remain_after_300 = remain - used_300;

            // Skip if not enough space for provided values
            if remain_after_300 < min_space_needed {
                continue;
            }

            let target_after_300 = target_numerator - score_300;

            // Optimize the last 3 values
            let (new200, new100, new50) =
                optimize_last_three(n200, n100, n50, remain_after_300, target_after_300);

            let result = ManiaHitResults {
                n320: new320,
                n300: new300,
                n200: new200,
                n100: new100,
                n50: new50,
                misses,
            };

            let actual_acc = result.accuracy(w320 == 60);
            let target_acc = target_numerator as f64 / f64::from(w320 * (remain + misses));
            let dist = (actual_acc - target_acc).abs();

            if dist < best_dist {
                best_dist = dist;
                best_result = result;
            }
        }
    }

    best_result
}

/// Optimizes the last three values (n200, n100, n50) given constraints.
fn optimize_last_three(
    n200: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    remain: u32,
    target_score: i64,
) -> (u32, u32, u32) {
    match (n200, n100, n50) {
        // All provided - use the values as-is (caller ensures enough space)
        (Some(n2), Some(n1), Some(n5)) => (n2, n1, n5),

        // Only one missing
        (Some(n2), Some(n1), None) => (n2, n1, remain.saturating_sub(n2 + n1)),
        (Some(n2), None, Some(n5)) => (n2, remain.saturating_sub(n2 + n5), n5),
        (None, Some(n1), Some(n5)) => (remain.saturating_sub(n1 + n5), n1, n5),

        // Two missing - solve analytically
        (Some(n2), None, None) => {
            let remaining = remain.saturating_sub(n2);
            let remaining_score = target_score - i64::from(40 * n2);

            // Solve: 20*n100 + 10*n50 = remaining_score, n100 + n50 = remaining
            let raw100 = (remaining_score - 10 * i64::from(remaining)) as f64 / 10.0;
            let n100 = cmp::min(remaining, raw100.round_ties_even().max(0.0) as u32);
            let n50 = remaining.saturating_sub(n100);

            (n2, n100, n50)
        }
        (None, Some(n1), None) => {
            let remaining = remain.saturating_sub(n1);
            let remaining_score = target_score - i64::from(20 * n1);

            // Solve: 40*n200 + 10*n50 = remaining_score, n200 + n50 = remaining
            let raw200 = (remaining_score - 10 * i64::from(remaining)) as f64 / 30.0;
            let n200 = cmp::min(remaining, raw200.round_ties_even().max(0.0) as u32);
            let n50 = remaining.saturating_sub(n200);

            (n200, n1, n50)
        }
        (None, None, Some(n5)) => {
            let remaining = remain.saturating_sub(n5);
            let remaining_score = target_score - i64::from(10 * n5);

            // Solve: 40*n200 + 20*n100 = remaining_score, n200 + n100 = remaining
            let raw200 = (remaining_score - 20 * i64::from(remaining)) as f64 / 20.0;
            let n200 = cmp::min(remaining, raw200.round_ties_even().max(0.0) as u32);
            let n100 = remaining.saturating_sub(n200);

            (n200, n100, n5)
        }

        // All three missing - full optimization
        (None, None, None) => {
            let mut best_dist = f64::MAX;
            let mut best = (0, 0, remain);

            // Establish bounds on n200
            let min200 = ((target_score - 20 * i64::from(remain)) as f64 / 20.0)
                .floor()
                .max(0.0) as u32;
            let max200 = cmp::min(remain, (target_score as f64 / 40.0).ceil() as u32 + 1);

            for n200 in min200..=max200 {
                let remaining = remain.saturating_sub(n200);
                let remaining_score = target_score - i64::from(40 * n200);

                // Solve for n100 and n50
                let raw100 = (remaining_score - 10 * i64::from(remaining)) as f64 / 10.0;
                let n100 = cmp::min(remaining, raw100.round_ties_even().max(0.0) as u32);
                let n50 = remaining.saturating_sub(n100);

                let actual_score = 40 * n200 + 20 * n100 + 10 * n50;
                let dist = (i64::from(actual_score) - target_score).abs() as f64;

                if dist < best_dist {
                    best_dist = dist;
                    best = (n200, n100, n50);
                }
            }

            best
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Difficulty,
        any::{HitResultGenerator, HitResultPriority},
        mania::{Mania, ManiaDifficultyAttributes, ManiaHitResults},
    };

    use super::*;

    /// Helper function to verify that a result is truly the closest to target accuracy.
    /// Tests neighboring states to ensure none are closer.
    fn verify_is_closest(inspect: &InspectManiaPerformance, result: &ManiaHitResults) {
        let is_classic = inspect.is_classic();
        let actual_acc = result.accuracy(is_classic);
        let target_acc = inspect.acc.unwrap();
        let current_dist = (actual_acc - target_acc).abs();

        let total = result.total_hits();

        // Test all possible single-step variations
        let variations = [
            // Increase n320, decrease others
            (1, 0, 0, 0, -1),
            (1, -1, 0, 0, 0),
            (1, 0, -1, 0, 0),
            (1, 0, 0, -1, 0),
            // Increase n300, decrease others
            (0, 1, 0, 0, -1),
            (0, 1, -1, 0, 0),
            (0, 1, 0, -1, 0),
            (-1, 1, 0, 0, 0),
            // Increase n200, decrease others
            (0, 0, 1, 0, -1),
            (0, 0, 1, -1, 0),
            (0, -1, 1, 0, 0),
            (-1, 0, 1, 0, 0),
            // Increase n100, decrease others
            (0, 0, 0, 1, -1),
            (0, 0, -1, 1, 0),
            (0, -1, 0, 1, 0),
            (-1, 0, 0, 1, 0),
            // Decrease n320, increase others
            (-1, 1, 0, 0, 0),
            (-1, 0, 1, 0, 0),
            (-1, 0, 0, 1, 0),
            (-1, 0, 0, 0, 1),
        ];

        for (d320, d300, d200, d100, d50) in variations {
            let new320 = (result.n320 as i32 + d320).max(0) as u32;
            let new300 = (result.n300 as i32 + d300).max(0) as u32;
            let new200 = (result.n200 as i32 + d200).max(0) as u32;
            let new100 = (result.n100 as i32 + d100).max(0) as u32;
            let new50 = (result.n50 as i32 + d50).max(0) as u32;

            // Skip if total doesn't match
            if new320 + new300 + new200 + new100 + new50 != total {
                continue;
            }

            // Skip if this violates user constraints
            if let Some(n) = inspect.n320 {
                if new320 != n {
                    continue;
                }
            }
            if let Some(n) = inspect.n300 {
                if new300 != n {
                    continue;
                }
            }
            if let Some(n) = inspect.n200 {
                if new200 != n {
                    continue;
                }
            }
            if let Some(n) = inspect.n100 {
                if new100 != n {
                    continue;
                }
            }
            if let Some(n) = inspect.n50 {
                if new50 != n {
                    continue;
                }
            }

            let neighbor = ManiaHitResults {
                n320: new320,
                n300: new300,
                n200: new200,
                n100: new100,
                n50: new50,
                misses: result.misses,
            };

            let neighbor_acc = neighbor.accuracy(is_classic);
            let neighbor_dist = (neighbor_acc - target_acc).abs();

            assert!(
                current_dist <= neighbor_dist + 1e-10,
                "Found closer neighbor! \
                Current: {result:?} (acc={actual_acc}, dist={current_dist}), \
                Neighbor: {neighbor:?} (acc={neighbor_acc}, dist={neighbor_dist})",
            );
        }
    }

    #[test]
    fn test_perfect_accuracy() {
        const N_OBJECTS: u32 = 1000;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
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

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n320, N_OBJECTS);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(false), 1.0);
    }

    #[test]
    fn test_all_missing_high_accuracy() {
        const N_OBJECTS: u32 = 500;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
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

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 10);

        let actual_acc = result.accuracy(false);
        assert!(
            (actual_acc - 0.95).abs() < 0.001,
            "Expected ~0.95, got {actual_acc}"
        );

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_missing_medium_accuracy() {
        const N_OBJECTS: u32 = 300;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.75),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(15),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 15);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_missing_low_accuracy() {
        const N_OBJECTS: u32 = 200;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.55),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(20),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 20);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_classic_mode() {
        const N_OBJECTS: u32 = 400;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new().lazer(false),
            acc: Some(0.88),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(8),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 8);

        // Verify it uses classic scoring (weight 60 for 320s)
        let actual_acc = result.accuracy(true);
        assert!(
            (actual_acc - 0.88).abs() < 0.001,
            "Expected ~0.88, got {actual_acc}"
        );

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_one_missing_n50() {
        const N_OBJECTS: u32 = 250;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.92),
            n320: Some(180),
            n300: Some(30),
            n200: Some(15),
            n100: Some(10),
            n50: None,
            misses: Some(5),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n320, 180);
        assert_eq!(result.n300, 30);
        assert_eq!(result.n200, 15);
        assert_eq!(result.n100, 10);
        assert_eq!(result.n50, 10); // N_OBJECTS - others
        assert_eq!(result.misses, 5);
        assert_eq!(result.total_hits(), N_OBJECTS);
    }

    #[test]
    fn test_two_missing_n320_n300() {
        const N_OBJECTS: u32 = 180;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85),
            n320: None,
            n300: None,
            n200: Some(25),
            n100: Some(20),
            n50: Some(15),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n200, 25);
        assert_eq!(result.n100, 20);
        assert_eq!(result.n50, 15);
        assert_eq!(result.misses, 10);
        assert_eq!(result.total_hits(), N_OBJECTS);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_two_missing_n100_n50() {
        const N_OBJECTS: u32 = 150;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n320: Some(80),
            n300: Some(40),
            n200: Some(15),
            n100: None,
            n50: None,
            misses: Some(5),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n320, 80);
        assert_eq!(result.n300, 40);
        assert_eq!(result.n200, 15);
        assert_eq!(result.misses, 5);
        assert_eq!(result.total_hits(), N_OBJECTS);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_three_missing() {
        const N_OBJECTS: u32 = 220;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.78),
            n320: Some(100),
            n300: None,
            n200: None,
            n100: None,
            n50: Some(30),
            misses: Some(12),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n320, 100);
        assert_eq!(result.n50, 30);
        assert_eq!(result.misses, 12);
        assert_eq!(result.total_hits(), N_OBJECTS);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_four_missing() {
        const N_OBJECTS: u32 = 280;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.82),
            n320: None,
            n300: None,
            n200: Some(45),
            n100: None,
            n50: None,
            misses: Some(18),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n200, 45);
        assert_eq!(result.misses, 18);
        assert_eq!(result.total_hits(), N_OBJECTS);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_all_provided() {
        const N_OBJECTS: u32 = 100;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85), // Accuracy doesn't matter when all are provided
            n320: Some(50),
            n300: Some(20),
            n200: Some(15),
            n100: Some(8),
            n50: Some(5),
            misses: Some(2),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n320, 50);
        assert_eq!(result.n300, 20);
        assert_eq!(result.n200, 15);
        assert_eq!(result.n100, 8);
        assert_eq!(result.n50, 5);
        assert_eq!(result.misses, 2);
    }

    #[test]
    fn test_clamping_when_exceed_remain() {
        const N_OBJECTS: u32 = 100;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.90),
            n320: Some(200), // Exceeds total
            n300: Some(50),
            n200: Some(30),
            n100: Some(20),
            n50: Some(10),
            misses: Some(5),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        // Should clamp to total_hits
        assert_eq!(result.total_hits(), N_OBJECTS);
        assert!(result.n320 <= 95); // N_OBJECTS - misses
    }

    #[test]
    fn test_all_misses() {
        const N_OBJECTS: u32 = 50;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.0),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(N_OBJECTS),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n320, 0);
        assert_eq!(result.n300, 0);
        assert_eq!(result.n200, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, N_OBJECTS);
    }

    #[test]
    fn test_edge_case_very_high_accuracy() {
        const N_OBJECTS: u32 = 350;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.9875),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(2),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 2);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_edge_case_very_low_accuracy() {
        const N_OBJECTS: u32 = 120;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.25),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(30),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 30);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_small_map() {
        const N_OBJECTS: u32 = 20;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.80),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(3),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 3);
        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn test_large_map() {
        const N_OBJECTS: u32 = 5000;

        let inspect = InspectManiaPerformance {
            attrs: &ManiaDifficultyAttributes {
                n_objects: N_OBJECTS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.87),
            n320: None,
            n300: None,
            n200: None,
            n100: None,
            n50: None,
            misses: Some(42),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <Closest as HitResultGenerator<Mania>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_OBJECTS);
        assert_eq!(result.misses, 42);

        let actual_acc = result.accuracy(false);
        assert!(
            (actual_acc - 0.87).abs() < 0.001,
            "Expected ~0.87, got {actual_acc}"
        );
    }
}
