use crate::{
    any::{HitResultGenerator, hitresult_generator::Closest},
    mania::{ManiaHitResults, performance::hitresult_generator::ManiaHitResultParams},
};

impl HitResultGenerator<ManiaHitResultParams> for Closest {
    fn generate_hitresults(params: &ManiaHitResultParams) -> ManiaHitResults {
        todo!()
    }
}
