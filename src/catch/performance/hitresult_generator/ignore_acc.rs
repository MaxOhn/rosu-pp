use std::cmp;

use crate::{
    any::{HitResultGenerator, hitresult_generator::IgnoreAccuracy},
    catch::{Catch, CatchHitResults, performance::inspect::InspectCatchPerformance},
};

impl HitResultGenerator<Catch> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectCatchPerformance<'_>) -> CatchHitResults {
        let n_fruits = inspect.attrs.n_fruits;
        let n_droplets = inspect.attrs.n_droplets;
        let n_tiny_droplets = inspect.attrs.n_tiny_droplets;

        // Get misses (clamped to fruits+droplets pool)
        let misses = inspect.misses();

        // Available catches from fruit/droplet pool
        let mut fruit_droplet_remain = (n_fruits + n_droplets).saturating_sub(misses);

        // Available from tiny droplet pool
        let mut tiny_droplet_remain = n_tiny_droplets;

        // Helper to assign a specified value from the fruit/droplet pool
        let mut assign_fruit_droplet = |specified: Option<u32>, max: u32| -> Option<u32> {
            let value = specified?;
            let assigned = cmp::min(cmp::min(value, max), fruit_droplet_remain);
            fruit_droplet_remain = fruit_droplet_remain.saturating_sub(assigned);

            Some(assigned)
        };

        // Helper to assign from tiny droplet pool
        let mut assign_tiny_droplet = |specified: Option<u32>| -> Option<u32> {
            let value = specified?;
            let assigned = cmp::min(value, tiny_droplet_remain);
            tiny_droplet_remain = tiny_droplet_remain.saturating_sub(assigned);

            Some(assigned)
        };

        // First pass: assign specified values in priority order (fruits > droplets > tiny_droplets)
        let fruits = assign_fruit_droplet(inspect.fruits, n_fruits);
        let droplets = assign_fruit_droplet(inspect.droplets, n_droplets);
        let tiny_droplets = assign_tiny_droplet(inspect.tiny_droplets);
        let tiny_droplet_misses = assign_tiny_droplet(inspect.tiny_droplet_misses);

        // Second pass: fill first unspecified with remainder
        let fruits = fruits.unwrap_or_else(|| {
            let take = cmp::min(fruit_droplet_remain, n_fruits);
            fruit_droplet_remain = fruit_droplet_remain.saturating_sub(take);

            take
        });

        let droplets = droplets.unwrap_or_else(|| {
            let take = cmp::min(fruit_droplet_remain, n_droplets);
            fruit_droplet_remain = fruit_droplet_remain.saturating_sub(take);

            take
        });

        let tiny_droplets = tiny_droplets.unwrap_or_else(|| {
            let take = tiny_droplet_remain;
            tiny_droplet_remain = 0;

            take
        });

        let tiny_droplet_misses = tiny_droplet_misses.unwrap_or(tiny_droplet_remain);

        CatchHitResults {
            fruits,
            droplets,
            tiny_droplets,
            tiny_droplet_misses,
            misses,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Difficulty, catch::CatchDifficultyAttributes};

    use super::*;

    #[test]
    fn all_missing() {
        const N_FRUITS: u32 = 50;
        const N_DROPLETS: u32 = 25;
        const N_TINY_DROPLETS: u32 = 100;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(5),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // Priority: fruits, then droplets, then tiny_droplets
        // With 5 misses, we have 70 slots for fruits+droplets
        assert_eq!(result.fruits, N_FRUITS);
        assert_eq!(result.droplets, 20); // 70 - 50 = 20
        assert_eq!(result.tiny_droplets, N_TINY_DROPLETS);
        assert_eq!(result.tiny_droplet_misses, 0);
        assert_eq!(result.misses, 5);
        assert_eq!(result.total_hits(), N_FRUITS + N_DROPLETS + N_TINY_DROPLETS);
    }

    #[test]
    fn some_provided() {
        const N_FRUITS: u32 = 50;
        const N_DROPLETS: u32 = 25;
        const N_TINY_DROPLETS: u32 = 100;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            fruits: Some(30),
            droplets: None,
            tiny_droplets: Some(50),
            tiny_droplet_misses: None,
            misses: Some(10),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // fruits=30, tiny_droplets=50 provided
        // Fruit/droplet pool: 75 - 10 misses = 65 available
        // fruits=30 takes 30, leaving 35 for droplets
        assert_eq!(result.fruits, 30);
        assert_eq!(result.droplets, 25); // min(35, 25) = 25
        assert_eq!(result.tiny_droplets, 50);
        assert_eq!(result.tiny_droplet_misses, 50); // 100 - 50 = 50
        assert_eq!(result.misses, 10);
    }

    #[test]
    fn droplets_provided() {
        const N_FRUITS: u32 = 50;
        const N_DROPLETS: u32 = 25;
        const N_TINY_DROPLETS: u32 = 100;

        let inspect = InspectCatchPerformance {
            attrs: &CatchDifficultyAttributes {
                n_fruits: N_FRUITS,
                n_droplets: N_DROPLETS,
                n_tiny_droplets: N_TINY_DROPLETS,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            fruits: None,
            droplets: Some(15),
            tiny_droplets: None,
            tiny_droplet_misses: Some(80),
            misses: Some(8),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // droplets=15, tiny_droplet_misses=80 provided
        // Fruit/droplet pool: 75 - 8 misses = 67 available
        // droplets=15 takes 15, leaving 52 for fruits
        assert_eq!(result.fruits, 50); // min(52, 50) = 50
        assert_eq!(result.droplets, 15);
        assert_eq!(result.tiny_droplets, 20); // 100 - 80 = 20
        assert_eq!(result.tiny_droplet_misses, 80);
        assert_eq!(result.misses, 8);
    }

    #[test]
    fn all_provided() {
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
            acc: None,
            combo: None,
            fruits: Some(35),
            droplets: Some(18),
            tiny_droplets: Some(70),
            tiny_droplet_misses: Some(10),
            misses: Some(3),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // All values provided, just clamp them
        assert_eq!(result.fruits, 35);
        assert_eq!(result.droplets, 18);
        assert_eq!(result.tiny_droplets, 70);
        assert_eq!(result.tiny_droplet_misses, 10);
        assert_eq!(result.misses, 3);
    }

    #[test]
    fn no_misses() {
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
            acc: None,
            combo: None,
            fruits: None,
            droplets: None,
            tiny_droplets: None,
            tiny_droplet_misses: None,
            misses: Some(0),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // No misses: all fruits, all droplets, all tiny_droplets
        assert_eq!(result.fruits, N_FRUITS);
        assert_eq!(result.droplets, N_DROPLETS);
        assert_eq!(result.tiny_droplets, N_TINY_DROPLETS);
        assert_eq!(result.tiny_droplet_misses, 0);
        assert_eq!(result.misses, 0);
    }

    #[test]
    fn excess_values_clamped() {
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
            acc: None,
            combo: None,
            fruits: Some(100), // Way more than available
            droplets: Some(50),
            tiny_droplets: Some(200),
            tiny_droplet_misses: Some(100),
            misses: Some(5),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // Values should be clamped to available space
        // Fruit/droplet pool: 30 - 5 = 25 available
        // fruits tries to take 100, gets clamped to min(100, 20, 25) = 20
        // That leaves 5 for droplets, so droplets gets min(50, 10, 5) = 5
        assert_eq!(result.fruits, 20);
        assert_eq!(result.droplets, 5);
        assert_eq!(result.tiny_droplets, 40); // min(200, 40)
        assert_eq!(result.tiny_droplet_misses, 0); // No space left in tiny pool
        assert_eq!(result.misses, 5);
    }
}
