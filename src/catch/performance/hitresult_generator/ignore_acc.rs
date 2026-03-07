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

        // Enforce pool constraints with priority
        // Fruit/droplet pool: misses > fruits > droplets
        let pool_total = n_fruits + n_droplets;
        let current_sum = fruits + droplets + misses;

        let (fruits, droplets) = match current_sum.cmp(&pool_total) {
            cmp::Ordering::Less => {
                // Need to add more - prioritize droplets (lower priority)
                let needed = pool_total - current_sum;
                let new_droplets = cmp::min(droplets + needed, n_droplets);
                let still_needed = pool_total.saturating_sub(fruits + new_droplets + misses);
                let new_fruits = cmp::min(fruits + still_needed, n_fruits);

                (new_fruits, new_droplets)
            }
            cmp::Ordering::Equal => (fruits, droplets),
            cmp::Ordering::Greater => {
                // Have too many - reduce droplets first (lower priority)
                let excess = current_sum - pool_total;
                let new_droplets = droplets.saturating_sub(excess);
                let still_excess = (fruits + new_droplets + misses).saturating_sub(pool_total);
                let new_fruits = fruits.saturating_sub(still_excess);

                (new_fruits, new_droplets)
            }
        };

        // Tiny droplet pool: tiny_droplets > tiny_droplet_misses
        let tiny_pool_total = n_tiny_droplets;
        let tiny_current_sum = tiny_droplets + tiny_droplet_misses;

        let (tiny_droplets, tiny_droplet_misses) = match tiny_current_sum.cmp(&tiny_pool_total) {
            cmp::Ordering::Less => {
                // Need to add more - prioritize tiny_droplets (higher priority)
                let needed = tiny_pool_total - tiny_current_sum;
                let new_tiny_droplets = cmp::min(tiny_droplets + needed, n_tiny_droplets);
                let still_needed = tiny_pool_total.saturating_sub(new_tiny_droplets);

                (new_tiny_droplets, still_needed)
            }
            cmp::Ordering::Equal => (tiny_droplets, tiny_droplet_misses),
            cmp::Ordering::Greater => {
                // Have too many - reduce tiny_droplet_misses first (lower priority)
                let excess = tiny_current_sum - tiny_pool_total;
                let new_tiny_droplet_misses = tiny_droplet_misses.saturating_sub(excess);
                let still_excess =
                    (tiny_droplets + new_tiny_droplet_misses).saturating_sub(tiny_pool_total);
                let new_tiny_droplets = tiny_droplets.saturating_sub(still_excess);

                (new_tiny_droplets, new_tiny_droplet_misses)
            }
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
        // fruits=30 is unsatisfiable
        // -> filling up 25 droplets (its max) still leaves 10
        // -> increment fruits to 40
        assert_eq!(result.fruits, 40);
        assert_eq!(result.droplets, 25);
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
        // droplets=15 is unsatisfiable
        // -> filling up 50 fruits (its max) still leaves 2
        // -> increment droplets to 17
        assert_eq!(result.fruits, 50);
        assert_eq!(result.droplets, 17);
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

        // Pool constraints will be enforced:
        // Fruit/droplet pool: 40 + 20 = 60, provided sum: 35 + 18 + 3 = 56 (missing 4)
        // Priority: misses > fruits > droplets, so adjust droplets: 18 + 4 = 22
        // But n_droplets = 20, so droplets = 20, still need 2 more
        // So adjust fruits: 35 + 2 = 37
        assert_eq!(result.fruits, 37);
        assert_eq!(result.droplets, 20);
        assert_eq!(result.misses, 3);

        // Tiny droplet pool: 80, provided sum: 70 + 10 = 80 (correct)
        assert_eq!(result.tiny_droplets, 70);
        assert_eq!(result.tiny_droplet_misses, 10);
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

    #[test]
    fn missing_objects() {
        const N_FRUITS: u32 = 728;
        const N_DROPLETS: u32 = 2;
        const N_TINY_DROPLETS: u32 = 263;

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
            fruits: Some(N_FRUITS - 10),
            droplets: Some(N_DROPLETS - 1),
            tiny_droplets: Some(N_TINY_DROPLETS - 50),
            tiny_droplet_misses: Some(20),
            misses: Some(2),
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Catch>>::generate_hitresults(inspect);

        // Enforcing pool constraints:
        // Fruit/droplet pool: 728 + 2 = 730, provided sum: 718 + 1 + 2 = 721 (missing 9)
        // Priority: misses > fruits > droplets
        // - droplets are capped to 2 so we assign 1 + 1 but have 8 left
        // - fruits are capped to 728 so we can assign 8 + 718 = 726
        assert_eq!(result.fruits, 726);
        assert_eq!(result.droplets, 2);

        // Tiny droplet pool: 263, provided sum: 213 + 20 = 233 (missing 30)
        // Priority: tiny_droplets_misses > tiny_droplet, so adjust tiny_droplets:
        //   213 + 30 = N_TINY_DROPLETS - 20
        assert_eq!(result.tiny_droplets, 243);
        assert_eq!(result.tiny_droplet_misses, 20);
        assert_eq!(result.misses, 2);
    }
}
