use crate::{
    any::{HitResultGenerator, hitresult_generator::IgnoreAccuracy},
    mania::{InspectManiaPerformance, Mania, ManiaHitResults},
};

impl HitResultGenerator<Mania> for IgnoreAccuracy {
    fn generate_hitresults(inspect: InspectManiaPerformance<'_>) -> ManiaHitResults {
        todo!()
    }
}
