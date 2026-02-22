use std::{cmp, mem};

use crate::{
    any::{HitResultGenerator, HitResultPriority, hitresult_generator::IgnoreAccuracy},
    taiko::{InspectTaikoPerformance, Taiko, TaikoHitResults},
};

impl HitResultGenerator<Taiko> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectTaikoPerformance<'_>) -> TaikoHitResults {
        let total_hits = inspect.total_hits();
        let misses = inspect.misses();
        let mut remain = total_hits - misses;

        // Helper to assign a specified value
        let mut assign_specified = |specified: Option<u32>| -> Option<u32> {
            let assigned = cmp::min(specified?, remain);
            remain -= assigned;

            Some(assigned)
        };

        let (n300, n100) = match inspect.hitresult_priority {
            HitResultPriority::BestCase => {
                // First pass: assign specified values in priority order
                let n300 = assign_specified(inspect.n300);
                let n100 = assign_specified(inspect.n100);

                // Second pass: fill first unspecified with remainder
                let mut n300 = n300.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n100 = n100.unwrap_or_else(|| mem::replace(&mut remain, 0));

                if remain > 0 {
                    n300 += remain;
                }

                (n300, n100)
            }
            HitResultPriority::WorstCase => {
                // First pass: assign specified values in priority order (worst to best)
                let n100 = assign_specified(inspect.n100);
                let n300 = assign_specified(inspect.n300);

                // Second pass: fill first unspecified with remainder
                let mut n100 = n100.unwrap_or_else(|| mem::replace(&mut remain, 0));
                let n300 = n300.unwrap_or_else(|| mem::replace(&mut remain, 0));

                if remain > 0 {
                    n100 += remain;
                }

                (n300, n100)
            }
        };

        TaikoHitResults { n300, n100, misses }
    }
}
