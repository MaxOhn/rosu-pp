use crate::{
    any::{HitResultGenerator, hitresult_generator::Fast},
    catch::{Catch, CatchHitResults, performance::inspect::InspectCatchPerformance},
};

impl HitResultGenerator<Catch> for Fast {
    fn generate_hitresults(inspect: InspectCatchPerformance<'_>) -> CatchHitResults {
        todo!()
    }
}
