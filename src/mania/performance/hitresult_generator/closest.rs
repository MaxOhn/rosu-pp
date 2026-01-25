use crate::{
    any::{HitResultGenerator, hitresult_generator::Closest},
    mania::{Mania, ManiaHitResults, performance::InspectManiaPerformance},
};

impl HitResultGenerator<Mania> for Closest {
    fn generate_hitresults(inspect: InspectManiaPerformance<'_>) -> ManiaHitResults {
        todo!()
    }
}
