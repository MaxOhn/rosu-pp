use std::cmp;

use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Fast, IgnoreAccuracy},
    },
    catch::{Catch, CatchHitResults, performance::inspect::InspectCatchPerformance},
};

impl HitResultGenerator<Catch> for Fast {
    #[expect(clippy::too_many_lines, reason = "it is what it is /shrug")]
    fn generate_hitresults(inspect: InspectCatchPerformance<'_>) -> CatchHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);
        };

        // In osu!catch:
        // - total_objects = n_fruits + n_droplets + n_tiny_droplets
        // - accuracy = (fruits + droplets + tiny_droplets) / total_objects
        // - Constraint: fruits + droplets + misses = n_fruits + n_droplets
        // - Constraint: tiny_droplets + tiny_droplet_misses = n_tiny_droplets

        let n_fruits = inspect.attrs.n_fruits;
        let n_droplets = inspect.attrs.n_droplets;
        let n_tiny_droplets = inspect.attrs.n_tiny_droplets;
        let total_objects = inspect.total_objects();

        if total_objects == 0 {
            return CatchHitResults {
                fruits: 0,
                droplets: 0,
                tiny_droplets: 0,
                tiny_droplet_misses: 0,
                misses: 0,
            };
        }

        // Get misses (fruits and droplets only, clamped)
        let misses = inspect.misses();

        // Calculate how many successful catches we need for target accuracy
        // acc = catches / total_objects
        // catches = acc * total_objects
        let catches_needed = (acc * f64::from(total_objects)).round() as u32;

        // Maximum possible catches considering misses
        let max_fruit_droplet_catches = (n_fruits + n_droplets).saturating_sub(misses);

        // Clamp and validate provided values
        let provided_fruits = inspect.fruits.map_or(0, |n| cmp::min(n, n_fruits));
        let provided_droplets = inspect.droplets.map_or(0, |n| cmp::min(n, n_droplets));
        let provided_tiny_droplets = inspect
            .tiny_droplets
            .map_or(0, |n| cmp::min(n, n_tiny_droplets));
        let provided_tiny_droplet_misses = inspect.tiny_droplet_misses.unwrap_or(0);

        // Make sure provided fruits + droplets don't exceed what's possible with the given misses
        let clamped_fruits = cmp::min(provided_fruits, max_fruit_droplet_catches);
        let clamped_droplets = cmp::min(
            provided_droplets,
            max_fruit_droplet_catches.saturating_sub(clamped_fruits),
        );

        match (
            inspect.fruits,
            inspect.droplets,
            inspect.tiny_droplets,
            inspect.tiny_droplet_misses,
        ) {
            // All provided - clamp and ensure pool constraints
            // Priority: misses > fruits > droplets (for fruit/droplet pool)
            // Priority: tiny_droplets > tiny_droplet_misses (for tiny droplet pool)
            (Some(_), Some(_), Some(_), Some(_)) => {
                // Handle fruit/droplet pool constraint
                let pool_total = n_fruits + n_droplets;
                let current_sum = clamped_fruits + clamped_droplets + misses;

                let (final_fruits, final_droplets) = match current_sum.cmp(&pool_total) {
                    cmp::Ordering::Less => {
                        // Need to add more - prioritize droplets (adjust lower priority first)
                        let needed = pool_total - current_sum;
                        let new_droplets = cmp::min(clamped_droplets + needed, n_droplets);
                        let still_needed =
                            pool_total.saturating_sub(clamped_fruits + new_droplets + misses);
                        let new_fruits = cmp::min(clamped_fruits + still_needed, n_fruits);

                        (new_fruits, new_droplets)
                    }
                    cmp::Ordering::Equal => (clamped_fruits, clamped_droplets),
                    cmp::Ordering::Greater => {
                        // Have too many - reduce droplets first (adjust lower priority first)
                        let excess = current_sum - pool_total;
                        let new_droplets = clamped_droplets.saturating_sub(excess);
                        let still_excess =
                            (clamped_fruits + new_droplets + misses).saturating_sub(pool_total);
                        let new_fruits = clamped_fruits.saturating_sub(still_excess);

                        (new_fruits, new_droplets)
                    }
                };

                // Handle tiny droplet pool constraint
                let tiny_pool_total = n_tiny_droplets;
                let tiny_current_sum = provided_tiny_droplets + provided_tiny_droplet_misses;

                let (final_tiny_droplets, final_tiny_droplet_misses) = match tiny_current_sum
                    .cmp(&tiny_pool_total)
                {
                    cmp::Ordering::Less => {
                        // Need to add more - prioritize tiny_droplets (higher priority)
                        let needed = tiny_pool_total - tiny_current_sum;
                        let new_tiny_droplets =
                            cmp::min(provided_tiny_droplets + needed, n_tiny_droplets);
                        let still_needed = tiny_pool_total.saturating_sub(new_tiny_droplets);

                        (new_tiny_droplets, still_needed)
                    }
                    cmp::Ordering::Equal => (provided_tiny_droplets, provided_tiny_droplet_misses),
                    cmp::Ordering::Greater => {
                        // Have too many - reduce tiny_droplet_misses first (lower priority)
                        let excess = tiny_current_sum - tiny_pool_total;
                        let new_tiny_droplet_misses =
                            provided_tiny_droplet_misses.saturating_sub(excess);
                        let still_excess = (provided_tiny_droplets + new_tiny_droplet_misses)
                            .saturating_sub(tiny_pool_total);
                        let new_tiny_droplets = provided_tiny_droplets.saturating_sub(still_excess);

                        (new_tiny_droplets, new_tiny_droplet_misses)
                    }
                };

                CatchHitResults {
                    fruits: final_fruits,
                    droplets: final_droplets,
                    tiny_droplets: final_tiny_droplets,
                    tiny_droplet_misses: final_tiny_droplet_misses,
                    misses,
                }
            }

            // Only one missing
            (Some(_), Some(_), Some(_), None) => {
                // tiny_droplet_misses is the only unknown
                let tiny_droplet_misses = n_tiny_droplets.saturating_sub(provided_tiny_droplets);

                CatchHitResults {
                    fruits: clamped_fruits,
                    droplets: clamped_droplets,
                    tiny_droplets: provided_tiny_droplets,
                    tiny_droplet_misses,
                    misses,
                }
            }
            (Some(_), Some(_), None, Some(_)) => {
                // tiny_droplets is the only unknown
                // We need to figure out how many tiny droplets to catch
                let current_catches = clamped_fruits + clamped_droplets;
                let remaining_catches = catches_needed.saturating_sub(current_catches);
                let tiny_droplets = cmp::min(remaining_catches, n_tiny_droplets);

                CatchHitResults {
                    fruits: clamped_fruits,
                    droplets: clamped_droplets,
                    tiny_droplets,
                    tiny_droplet_misses: provided_tiny_droplet_misses,
                    misses,
                }
            }
            (Some(_), None, Some(_), Some(_)) => {
                // droplets is the only unknown
                // Use pool constraint: droplets = n_fruits + n_droplets - fruits - misses
                let droplets_by_pool =
                    (n_fruits + n_droplets).saturating_sub(clamped_fruits + misses);
                let droplets = cmp::min(droplets_by_pool, n_droplets);

                CatchHitResults {
                    fruits: clamped_fruits,
                    droplets,
                    tiny_droplets: provided_tiny_droplets,
                    tiny_droplet_misses: provided_tiny_droplet_misses,
                    misses,
                }
            }
            (None, Some(_), Some(_), Some(_)) => {
                // fruits is the only unknown
                // Use pool constraint: fruits = n_fruits + n_droplets - droplets - misses
                let fruits_by_pool =
                    (n_fruits + n_droplets).saturating_sub(clamped_droplets + misses);
                let fruits = cmp::min(fruits_by_pool, n_fruits);

                CatchHitResults {
                    fruits,
                    droplets: clamped_droplets,
                    tiny_droplets: provided_tiny_droplets,
                    tiny_droplet_misses: provided_tiny_droplet_misses,
                    misses,
                }
            }

            // Two or more missing - use fast approximation
            _ => {
                // Calculate how many catches we still need after accounting for provided values
                let provided_catches = clamped_fruits + clamped_droplets + provided_tiny_droplets;
                let mut remain_catches = catches_needed.saturating_sub(provided_catches);

                // We need to distribute remaining catches among missing types
                // Priority: fruits > droplets > tiny_droplets (for performance)

                let fruits = if inspect.fruits.is_none() {
                    // Maximum fruits we can catch considering:
                    // 1. How many fruits exist (n_fruits)
                    // 2. How many fruit/droplet slots are available after misses and provided droplets
                    let max_by_pool = max_fruit_droplet_catches.saturating_sub(clamped_droplets);
                    let max_fruits = cmp::min(n_fruits, max_by_pool);
                    let caught = cmp::min(remain_catches, max_fruits);
                    remain_catches = remain_catches.saturating_sub(caught);

                    caught
                } else {
                    clamped_fruits
                };

                let droplets = if inspect.droplets.is_some() {
                    clamped_droplets
                } else if inspect.fruits.is_none() {
                    // If fruits is also missing, calculate based on remaining catches

                    // Both fruits and droplets are missing
                    let max_by_pool = max_fruit_droplet_catches.saturating_sub(fruits);
                    let max_droplets = cmp::min(n_droplets, max_by_pool);
                    let caught = cmp::min(remain_catches, max_droplets);
                    remain_catches = remain_catches.saturating_sub(caught);

                    caught
                } else {
                    // Only droplets is missing, fruits was provided
                    // Use pool constraint: droplets = n_fruits + n_droplets - fruits - misses
                    let droplets_by_pool = (n_fruits + n_droplets).saturating_sub(fruits + misses);
                    let droplets = cmp::min(droplets_by_pool, n_droplets);

                    // Decrement remaining_catches by the droplets we're catching
                    remain_catches = remain_catches.saturating_sub(droplets);

                    droplets
                };

                // If fruits was provided but droplets couldn't fill the pool,
                // adjust fruits upward to satisfy the pool constraint
                let fruits = if inspect.fruits.is_some() && inspect.droplets.is_none() {
                    let pool_sum = fruits + droplets + misses;
                    let expected = n_fruits + n_droplets;

                    if pool_sum < expected {
                        // Add the difference to fruits
                        let adjusted = cmp::min(n_fruits, fruits + (expected - pool_sum));
                        // Also update remaining_catches since we added more catches
                        let added_catches = adjusted - fruits;
                        remain_catches = remain_catches.saturating_sub(added_catches);

                        adjusted
                    } else {
                        fruits
                    }
                } else {
                    fruits
                };

                let tiny_droplets = if inspect.tiny_droplets.is_none() {
                    cmp::min(remain_catches, n_tiny_droplets)
                } else {
                    provided_tiny_droplets
                };

                let tiny_droplet_misses = if inspect.tiny_droplet_misses.is_none() {
                    // Calculate how many tiny droplets were missed
                    n_tiny_droplets.saturating_sub(tiny_droplets)
                } else {
                    provided_tiny_droplet_misses
                };

                CatchHitResults {
                    fruits,
                    droplets,
                    tiny_droplets,
                    tiny_droplet_misses,
                    misses,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Difficulty, catch::CatchDifficultyAttributes};

    use super::*;

    #[test]
    fn perfect_accuracy() {
        const N_FRUITS: u32 = 100;
        const N_DROPLETS: u32 = 50;
        const N_TINY_DROPLETS: u32 = 200;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(1.0),
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(0),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        assert_eq!(result.fruits, N_FRUITS);
        assert_eq!(result.droplets, N_DROPLETS);
        assert_eq!(result.tiny_droplets, N_TINY_DROPLETS);
        assert_eq!(result.tiny_droplet_misses, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(), 1.0);
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
    }

    #[test]
    fn high_accuracy_with_misses() {
        const N_FRUITS: u32 = 80;
        const N_DROPLETS: u32 = 40;
        const N_TINY_DROPLETS: u32 = 100;
        const MISSES: u32 = 5; // fruit/droplet misses
        const ACC: f64 = 0.95;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(ACC),
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(MISSES),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // Total objects is fixed
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, MISSES);

        // Verify accuracy
        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.01,
            "Expected ~{ACC}, got {actual_acc}"
        );
    }

    #[test]
    fn medium_accuracy() {
        const N_FRUITS: u32 = 60;
        const N_DROPLETS: u32 = 30;
        const N_TINY_DROPLETS: u32 = 150;
        const MISSES: u32 = 10;
        const ACC: f64 = 0.85;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(ACC),
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(MISSES),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, MISSES);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.01,
            "Expected ~{ACC}, got {actual_acc}"
        );
    }

    #[test]
    fn with_fruits_provided() {
        const N_FRUITS: u32 = 50;
        const N_DROPLETS: u32 = 25;
        const N_TINY_DROPLETS: u32 = 100;
        const ACC: f64 = 0.90;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(ACC),
            combo: None,
            fruits: Some(45),
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(3),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // 25 droplets are the max but with 45 fruits and 3 misses we're still
        // missing 2 hits. Since droplets are at the max and misses have the
        // highest priority, the amount of fruits needs to be adjusted to 47
        // despite being specified to be lower.
        assert_eq!(result.fruits, 47);
        assert_eq!(result.misses, 3);
        assert_eq!(result.droplets, 25);
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.02,
            "Expected ~{ACC}, got {actual_acc}"
        );
    }

    #[test]
    fn all_values_provided() {
        const N_FRUITS: u32 = 40;
        const N_DROPLETS: u32 = 20;
        const N_TINY_DROPLETS: u32 = 80;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85), // Ignored when all provided
            combo: None,
            fruits: Some(35),
            droplets: Some(18),
            tiny_droplets: Some(70),
            tiny_droplet_misses: Some(7),
            misses: Some(4),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // Pool constraints will be enforced:
        // Fruit/droplet pool: 40 + 20 = 60, provided sum: 35 + 18 + 4 = 57 (missing 3)
        // Priority: misses > fruits > droplets
        // - droplets are capped to 20 so we assign 2 + 18 but have 1 left
        // - fruits are capped to 40 so we can assign 1 + 35
        assert_eq!(result.fruits, 36); // Adjusted from 35
        assert_eq!(result.droplets, 20); // Adjusted from 18
        assert_eq!(result.misses, 4); // Remains as given

        // Tiny droplet pool: 80, provided sum: 70 + 7 = 77 (missing 3)
        // Priority: tiny_droplets_misses > tiny_droplet, so adjust tiny_droplets: 70 + 3 = 73
        assert_eq!(result.tiny_droplets, 73); // Adjusted from 70
        assert_eq!(result.tiny_droplet_misses, 7); // Remains as given

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
    }

    #[test]
    fn low_accuracy_many_tiny_droplet_misses() {
        const N_FRUITS: u32 = 30;
        const N_DROPLETS: u32 = 15;
        const N_TINY_DROPLETS: u32 = 60;
        const MISSES: u32 = 10;
        const ACC: f64 = 0.60;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(ACC),
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(MISSES),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, MISSES);

        // With 60% acc on 105 objects, we need 63 catches
        // We have 10 fruit/droplet misses, so max fruit+droplet = 35
        // But we only need 63 total catches
        let catches = result.fruits + result.droplets + result.tiny_droplets;
        assert_eq!(catches, 63);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.01,
            "Expected ~{ACC}, got {actual_acc}"
        );
    }

    #[test]
    fn zero_accuracy_all_misses() {
        const N_FRUITS: u32 = 20;
        const N_DROPLETS: u32 = 10;
        const N_TINY_DROPLETS: u32 = 40;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.0),
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(N_FRUITS + N_DROPLETS), // All fruits and droplets missed
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        assert_eq!(result.fruits, 0);
        assert_eq!(result.droplets, 0);
        assert_eq!(result.tiny_droplets, 0);
        assert_eq!(result.misses, 30);
        assert_eq!(result.tiny_droplet_misses, N_TINY_DROPLETS);
        assert_eq!(result.accuracy(), 0.0);
    }

    #[test]
    fn only_tiny_droplet_misses_missing() {
        const N_FRUITS: u32 = 25;
        const N_DROPLETS: u32 = 15;
        const N_TINY_DROPLETS: u32 = 50;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.88),
            combo: None,
            fruits: Some(20),
            droplets: Some(12),
            tiny_droplets: Some(47),
            tiny_droplet_misses: None,
            misses: Some(5),
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        assert_eq!(result.fruits, 20);
        assert_eq!(result.droplets, 12);
        assert_eq!(result.tiny_droplets, 47);
        assert_eq!(result.misses, 5);

        // tiny_droplet_misses = n_tiny_droplets - tiny_droplets
        assert_eq!(result.tiny_droplet_misses, N_TINY_DROPLETS - 47);

        // Note: total_hits() will be 87 not 90 because the provided values
        // (fruits=20, droplets=12, misses=5) only account for 37 of the 40 fruits+droplets
    }

    #[test]
    fn misses_clamped_to_fruits_plus_droplets() {
        const N_FRUITS: u32 = 20;
        const N_DROPLETS: u32 = 10;
        const N_TINY_DROPLETS: u32 = 30;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.80),
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(100), // Way more than possible
        };

        let result = <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // Misses should be clamped to n_fruits + n_droplets
        assert_eq!(result.misses, N_FRUITS + N_DROPLETS);
        assert_eq!(result.fruits, 0);
        assert_eq!(result.droplets, 0);
    }
}
