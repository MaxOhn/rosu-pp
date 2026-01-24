use crate::{
    any::{HitResultPriority, hitresult_generator::HitResultParams},
    mania::score_state::ManiaHitResults,
};

mod closest;
mod fast;

/// TODO: docs
#[derive(Clone, Debug)]
pub struct ManiaHitResultParams {
    // Guaranteed to be correct
    pub total_hits: u32,
    pub is_classic: bool,
    pub priority: HitResultPriority,

    // Provided by user; may be infeasable
    pub acc: f64,
    pub n320: Option<u32>,
    pub n300: Option<u32>,
    pub n200: Option<u32>,
    pub n100: Option<u32>,
    pub n50: Option<u32>,
    pub misses: u32,
}

impl HitResultParams for ManiaHitResultParams {
    type HitResults = ManiaHitResults;
}
