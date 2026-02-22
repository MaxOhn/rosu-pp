use std::{cmp, mem};

use crate::{
    any::{HitResultGenerator, HitResultPriority, hitresult_generator::IgnoreAccuracy},
    osu::{InspectOsuPerformance, Osu, OsuHitResults},
};

impl HitResultGenerator<Osu> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectOsuPerformance<'_>) -> OsuHitResults {
        let (slider_end_hits, large_tick_hits, small_tick_hits) = inspect.tick_hits();

        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let mut remain = total_hits - misses;

        // Helper to assign a specified value
        let mut assign_specified = |specified: Option<u32>| -> Option<u32> {
            let assigned = cmp::min(specified?, remain);
            remain -= assigned;

            Some(assigned)
        };

        let (n300, n100, n50) = match inspect.hitresult_priority {
            HitResultPriority::BestCase => {
                // First pass: assign specified values in priority order
                let n300 = assign_specified(inspect.n300);
                let n100 = assign_specified(inspect.n100);
                let n50 = assign_specified(inspect.n50);

                // Second pass: fill first unspecified with remainder
                let mut n300 = n300.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n100 = n100.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n50 = n50.unwrap_or_else(|| mem::replace(&mut remain, 0));

                if remain > 0 {
                    n300 += remain;
                }

                (n300, n100, n50)
            }
            HitResultPriority::WorstCase => {
                // First pass: assign specified values in priority order (worst to best)
                let n50 = assign_specified(inspect.n50);
                let n100 = assign_specified(inspect.n100);
                let n300 = assign_specified(inspect.n300);

                // Second pass: fill first unspecified with remainder
                let mut n50 = n50.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n100 = n100.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n300 = n300.unwrap_or_else(|| mem::replace(&mut remain, 0));

                if remain > 0 {
                    n50 += remain;
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
    use crate::{Difficulty, osu::OsuDifficultyAttributes};

    use super::*;

    #[test]
    fn all_specified_exact() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(40),
            n100: Some(30),
            n50: Some(20),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn all_specified_too_few_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(40),
            n100: Some(30),
            n50: Some(10),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300,
            N_CIRCLES - inspect.n100.unwrap() - inspect.n50.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn all_specified_too_few_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(40),
            n100: Some(30),
            n50: Some(10),
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(
            result.n50,
            N_CIRCLES - inspect.n300.unwrap() - inspect.n100.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn all_specified_too_many_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(40),
            n100: Some(30),
            n50: Some(30),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, 20); // Best case so we subtract from n50s
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn all_specified_too_many_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(40),
            n100: Some(30),
            n50: Some(30),
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 30); // Worst case so we subtract from n300s
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn none_specified_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 90);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn none_specified_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 90);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn only_n300_specified_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(50),
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n100,
            N_CIRCLES - inspect.n300.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn only_n300_specified_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(50),
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, 0);
        assert_eq!(
            result.n50,
            N_CIRCLES - inspect.n300.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn only_n100_specified_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: Some(30),
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300,
            N_CIRCLES - inspect.n100.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn only_n100_specified_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: Some(30),
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(
            result.n50,
            N_CIRCLES - inspect.n100.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn only_n50_specified_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: Some(20),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300,
            N_CIRCLES - inspect.n50.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn only_n50_specified_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: Some(20),
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, 0);
        assert_eq!(
            result.n100,
            N_CIRCLES - inspect.n50.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn n300_and_n100_specified_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(50),
            n100: Some(20),
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(
            result.n50,
            N_CIRCLES - inspect.n300.unwrap() - inspect.n100.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn n300_and_n50_specified_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(50),
            n100: None,
            n50: Some(20),
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n100,
            N_CIRCLES - inspect.n300.unwrap() - inspect.n50.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn n100_and_n50_specified_worst() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: Some(20),
            n50: Some(30),
            misses: Some(10),
            hitresult_priority: HitResultPriority::WorstCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(
            result.n300,
            N_CIRCLES - inspect.n100.unwrap() - inspect.n50.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n100, inspect.n100.unwrap());
        assert_eq!(result.n50, inspect.n50.unwrap());
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn specified_value_exceeds_total_best() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(150),
            n100: None,
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, N_CIRCLES - inspect.misses.unwrap());
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn multiple_specified_exceed_total() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(60),
            n100: Some(50),
            n50: None,
            misses: Some(10),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(
            result.n100,
            N_CIRCLES - inspect.n300.unwrap() - inspect.misses.unwrap()
        );
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, inspect.misses.unwrap());
    }

    #[test]
    fn no_misses() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: Some(80),
            n100: None,
            n50: None,
            misses: None,
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result =
            <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect.clone());

        assert_eq!(result.n300, inspect.n300.unwrap());
        assert_eq!(result.n100, N_CIRCLES - inspect.n300.unwrap());
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 0);
    }

    #[test]
    fn all_misses() {
        const N_CIRCLES: u32 = 100;

        let inspect = InspectOsuPerformance {
            attrs: &OsuDifficultyAttributes {
                n_circles: N_CIRCLES,
                ..Default::default()
            },
            difficulty: &Difficulty::new(),
            acc: None,
            combo: None,
            large_tick_hits: None,
            small_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: None,
            misses: Some(100),
            hitresult_priority: HitResultPriority::BestCase,
        };

        let result = <IgnoreAccuracy as HitResultGenerator<Osu>>::generate_hitresults(inspect);

        assert_eq!(result.n300, 0);
        assert_eq!(result.n100, 0);
        assert_eq!(result.n50, 0);
        assert_eq!(result.misses, 100);
    }
}
