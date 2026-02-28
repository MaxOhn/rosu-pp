use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Closest, Fast},
    },
    catch::{Catch, CatchHitResults, performance::inspect::InspectCatchPerformance},
};

impl HitResultGenerator<Catch> for Closest {
    fn generate_hitresults(inspect: InspectCatchPerformance) -> CatchHitResults {
        <Fast as HitResultGenerator<Catch>>::generate_hitresults(inspect)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Difficulty,
        any::HitResultGenerator,
        catch::{
            Catch, CatchDifficultyAttributes, CatchHitResults,
            performance::inspect::InspectCatchPerformance,
        },
    };

    use super::*;

    /// Helper function to verify that a result is truly the closest to target accuracy.
    /// Tests neighboring states to ensure none are closer.
    fn verify_is_closest(inspect: &InspectCatchPerformance, result: &CatchHitResults) {
        let actual_acc = result.accuracy();
        let target_acc = inspect.acc.unwrap();
        let current_dist = (actual_acc - target_acc).abs();

        let n_fruits = inspect.attrs.n_fruits;
        let n_droplets = inspect.attrs.n_droplets;
        let n_tiny_droplets = inspect.attrs.n_tiny_droplets;
        let misses = inspect.misses();

        // Test all possible single-step variations
        let variations = [
            // Increase fruits, decrease droplets (within pool constraint)
            (1, -1, 0, 0),
            // Increase fruits, decrease tiny_droplets
            (1, 0, -1, 0),
            // Increase droplets, decrease fruits
            (-1, 1, 0, 0),
            // Increase droplets, decrease tiny_droplets
            (0, 1, -1, 0),
            // Increase tiny_droplets, decrease fruits
            (-1, 0, 1, 0),
            // Increase tiny_droplets, decrease droplets
            (0, -1, 1, 0),
            // Increase tiny_droplets, decrease tiny_droplet_misses
            (0, 0, 1, -1),
            // Increase tiny_droplet_misses, decrease tiny_droplets
            (0, 0, -1, 1),
        ];

        for (d_fruits, d_droplets, d_tiny_droplets, d_tiny_droplet_misses) in variations {
            let new_fruits = (result.fruits as i32 + d_fruits).max(0) as u32;
            let new_droplets = (result.droplets as i32 + d_droplets).max(0) as u32;
            let new_tiny_droplets = (result.tiny_droplets as i32 + d_tiny_droplets).max(0) as u32;
            let new_tiny_droplet_misses =
                (result.tiny_droplet_misses as i32 + d_tiny_droplet_misses).max(0) as u32;

            // Skip if exceeds limits
            if new_fruits > n_fruits
                || new_droplets > n_droplets
                || new_tiny_droplets > n_tiny_droplets
                || new_tiny_droplet_misses > n_tiny_droplets
            {
                continue;
            }

            // Skip if violates pool constraints
            if new_fruits + new_droplets + misses != n_fruits + n_droplets {
                continue;
            }

            if new_tiny_droplets + new_tiny_droplet_misses != n_tiny_droplets {
                continue;
            }

            // Skip if this violates user constraints
            if let Some(n) = inspect.fruits {
                if new_fruits != n {
                    continue;
                }
            }

            if let Some(n) = inspect.droplets {
                if new_droplets != n {
                    continue;
                }
            }

            if let Some(n) = inspect.tiny_droplets {
                if new_tiny_droplets != n {
                    continue;
                }
            }

            if let Some(n) = inspect.tiny_droplet_misses {
                if new_tiny_droplet_misses != n {
                    continue;
                }
            }

            let neighbor = CatchHitResults {
                fruits: new_fruits,
                droplets: new_droplets,
                tiny_droplets: new_tiny_droplets,
                tiny_droplet_misses: new_tiny_droplet_misses,
                misses: result.misses,
            };

            let neighbor_acc = neighbor.accuracy();
            let neighbor_dist = (neighbor_acc - target_acc).abs();

            assert!(
                current_dist <= neighbor_dist + 1e-10,
                "Found closer neighbor! \
                Current: {result:?} (acc={actual_acc:.6}, dist={current_dist:.6}), \
                Neighbor: {neighbor:?} (acc={neighbor_acc:.6}, dist={neighbor_dist:.6})",
            );
        }
    }

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

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.fruits, N_FRUITS);
        assert_eq!(result.droplets, N_DROPLETS);
        assert_eq!(result.tiny_droplets, N_TINY_DROPLETS);
        assert_eq!(result.tiny_droplet_misses, 0);
        assert_eq!(result.misses, 0);
        assert_eq!(result.accuracy(), 1.0);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn high_accuracy_with_misses() {
        const N_FRUITS: u32 = 80;
        const N_DROPLETS: u32 = 40;
        const N_TINY_DROPLETS: u32 = 100;
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
            misses: Some(5),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 5);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.01,
            "Expected ~{ACC}, got {actual_acc}"
        );

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn medium_accuracy() {
        const N_FRUITS: u32 = 60;
        const N_DROPLETS: u32 = 30;
        const N_TINY_DROPLETS: u32 = 150;
        const ACC: f64 = 0.75;

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
            misses: Some(15),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 15);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn low_accuracy() {
        const N_FRUITS: u32 = 40;
        const N_DROPLETS: u32 = 20;
        const N_TINY_DROPLETS: u32 = 80;
        const ACC: f64 = 0.50;

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
            misses: Some(20),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 20);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn fruits_provided() {
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

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        // Fruits will be adjusted to 47 to satisfy pool constraint
        assert_eq!(result.fruits, 47);
        assert_eq!(result.misses, 3);
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn droplets_provided() {
        const N_FRUITS: u32 = 50;
        const N_DROPLETS: u32 = 25;
        const N_TINY_DROPLETS: u32 = 100;
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
            droplets: Some(20),
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(8),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.droplets, 20);
        assert_eq!(result.misses, 8);
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn tiny_droplets_provided() {
        const N_FRUITS: u32 = 40;
        const N_DROPLETS: u32 = 20;
        const N_TINY_DROPLETS: u32 = 80;
        const ACC: f64 = 0.80;

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
            tiny_droplets: Some(60),
            tiny_droplet_misses: None,
            misses: Some(10),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.tiny_droplets, 60);
        assert_eq!(result.misses, 10);
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn all_values_provided() {
        const N_FRUITS: u32 = 30;
        const N_DROPLETS: u32 = 15;
        const N_TINY_DROPLETS: u32 = 60;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: Some(0.85),
            combo: None,
            fruits: Some(25),
            droplets: Some(12),
            tiny_droplets: Some(50),
            tiny_droplet_misses: Some(10),
            misses: Some(5),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        // When all values provided but don't sum correctly:
        // Priority: misses > fruits > droplets
        // fruits=25, droplets=12, misses=5 sum to 42, but pool needs 45
        // So: keep misses=5, keep fruits=25, adjust droplets to 15
        assert_eq!(result.misses, 5);
        assert_eq!(result.fruits, 25);
        assert_eq!(result.droplets, 15); // Adjusted from 12 to satisfy pool constraint
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn zero_accuracy() {
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
            misses: Some(30),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.fruits, 0);
        assert_eq!(result.droplets, 0);
        assert_eq!(result.tiny_droplets, 0);
        assert_eq!(result.tiny_droplet_misses, N_TINY_DROPLETS);
        assert_eq!(result.misses, 30);
        assert_eq!(result.accuracy(), 0.0);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn accuracy_very_close_to_one() {
        const N_FRUITS: u32 = 35;
        const N_DROPLETS: u32 = 18;
        const N_TINY_DROPLETS: u32 = 70;
        const ACC: f64 = 0.9878;

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
            misses: Some(1),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 1);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.01,
            "Expected ~{ACC}, got {actual_acc}"
        );

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn small_map() {
        const N_FRUITS: u32 = 10;
        const N_DROPLETS: u32 = 5;
        const N_TINY_DROPLETS: u32 = 20;
        const ACC: f64 = 0.70;

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
            misses: Some(3),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 3);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn large_map() {
        const N_FRUITS: u32 = 500;
        const N_DROPLETS: u32 = 250;
        const N_TINY_DROPLETS: u32 = 1000;
        const ACC: f64 = 0.87;

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
            misses: Some(42),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 42);

        let actual_acc = result.accuracy();
        assert!(
            (actual_acc - ACC).abs() < 0.01,
            "Expected ~{ACC}, got {actual_acc}"
        );

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn mostly_tiny_droplets() {
        const N_FRUITS: u32 = 20;
        const N_DROPLETS: u32 = 10;
        const N_TINY_DROPLETS: u32 = 200;
        const ACC: f64 = 0.65;

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
            misses: Some(8),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 8);

        verify_is_closest(&inspect, &result);
    }

    #[test]
    fn accuracy_requiring_rounding() {
        const N_FRUITS: u32 = 33;
        const N_DROPLETS: u32 = 17;
        const N_TINY_DROPLETS: u32 = 66;
        const ACC: f64 = 0.7241379; // Odd accuracy that requires careful rounding

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
            misses: Some(7),
        };

        let result = <Closest as HitResultGenerator<Catch>>::generate_hitresults(inspect.clone());

        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
        assert_eq!(result.misses, 7);

        verify_is_closest(&inspect, &result);
    }
}
