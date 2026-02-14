use crate::{
    any::{HitResultGenerator, hitresult_generator::IgnoreAccuracy},
    catch::{Catch, CatchHitResults, performance::inspect::InspectCatchPerformance},
};

impl HitResultGenerator<Catch> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectCatchPerformance<'_>) -> CatchHitResults {
        todo!()
    }
}
