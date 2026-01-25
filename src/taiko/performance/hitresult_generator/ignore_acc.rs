use std::cmp;

use crate::{
    any::{HitResultGenerator, HitResultPriority, hitresult_generator::IgnoreAccuracy},
    taiko::{InspectTaikoPerformance, Taiko, TaikoHitResults},
};

impl HitResultGenerator<Taiko> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectTaikoPerformance<'_>) -> TaikoHitResults {
        let total_hits = inspect.total_hits();
        let misses = inspect.misses();

        let remain = total_hits - misses;

        let (n300, n100) = match (inspect.n300, inspect.n100) {
            (Some(n300), Some(n100)) => match inspect.hitresult_priority {
                HitResultPriority::BestCase => {
                    let n300 = cmp::min(n300, remain);
                    let n100 = cmp::min(n100, remain - n300);

                    (n300, n100)
                }
                HitResultPriority::WorstCase => {
                    let n100 = cmp::min(n100, remain);
                    let n300 = cmp::min(n300, remain - n100);

                    (n300, n100)
                }
                HitResultPriority::Fastest => todo!(),
            },
            (Some(n300), None) => {
                let n300 = cmp::min(n300, remain);
                let n100 = remain - n300;

                (n300, n100)
            }
            (None, Some(n100)) => {
                let n100 = cmp::min(n100, remain);
                let n300 = remain - n100;

                (n300, n100)
            }
            (None, None) => match inspect.hitresult_priority {
                HitResultPriority::BestCase => (remain, 0),
                HitResultPriority::WorstCase => (0, remain),
                HitResultPriority::Fastest => todo!(),
            },
        };

        TaikoHitResults { n300, n100, misses }
    }
}
