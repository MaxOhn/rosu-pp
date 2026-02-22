use std::{cmp, mem};

use crate::{
    any::{HitResultGenerator, HitResultPriority, hitresult_generator::IgnoreAccuracy},
    mania::{InspectManiaPerformance, Mania, ManiaHitResults},
};

impl HitResultGenerator<Mania> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectManiaPerformance<'_>) -> ManiaHitResults {
        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let mut remain = total_hits - misses;

        // Helper to assign a specified value
        let mut assign_specified = |specified: Option<u32>| -> Option<u32> {
            let assigned = cmp::min(specified?, remain);
            remain -= assigned;

            Some(assigned)
        };

        let (n320, n300, n200, n100, n50) = match inspect.hitresult_priority {
            HitResultPriority::BestCase => {
                // First pass: assign specified values in priority order
                let n320 = assign_specified(inspect.n320);
                let n300 = assign_specified(inspect.n300);
                let n200 = assign_specified(inspect.n200);
                let n100 = assign_specified(inspect.n100);
                let n50 = assign_specified(inspect.n50);

                // Second pass: fill first unspecified with remainder
                let mut n320 = n320.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n300 = n300.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n200 = n200.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n100 = n100.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n50 = n50.unwrap_or_else(|| mem::replace(&mut remain, 0));

                if remain > 0 {
                    n320 += remain;
                }

                (n320, n300, n200, n100, n50)
            }
            HitResultPriority::WorstCase => {
                // First pass: assign specified values in priority order (worst to best)
                let n50 = assign_specified(inspect.n50);
                let n100 = assign_specified(inspect.n100);
                let n200 = assign_specified(inspect.n200);
                let n300 = assign_specified(inspect.n300);
                let n320 = assign_specified(inspect.n320);

                // Second pass: fill first unspecified with remainder
                let mut n50 = n50.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n100 = n100.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n200 = n200.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n300 = n300.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n320 = n320.unwrap_or_else(|| mem::replace(&mut remain, 0));

                if remain > 0 {
                    n50 += remain;
                }

                (n320, n300, n200, n100, n50)
            }
        };

        ManiaHitResults {
            n320,
            n300,
            n200,
            n100,
            n50,
            misses,
        }
    }
}
