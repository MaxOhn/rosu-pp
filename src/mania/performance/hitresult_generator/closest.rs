use crate::{
    any::{
        HitResultGenerator,
        hitresult_generator::{Closest, IgnoreAccuracy},
    },
    mania::{Mania, ManiaHitResults, performance::InspectManiaPerformance},
};

impl HitResultGenerator<Mania> for Closest {
    fn generate_hitresults(inspect: InspectManiaPerformance<'_>) -> ManiaHitResults {
        let Some(acc) = inspect.acc else {
            return <IgnoreAccuracy as HitResultGenerator<Mania>>::generate_hitresults(inspect);
        };

        todo!()
    }
}
