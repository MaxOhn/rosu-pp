use crate::{any::hitresult_generator::HitResultParams, taiko::TaikoHitResults};

mod closest;
mod fast;

/// TODO: docs
pub struct TaikoHitResultParams {
    // Guaranteed to be correct
    pub total_hits: u32,

    // Provided by user; may be infeasable
    pub acc: f64,
    pub n300: Option<u32>,
    pub n100: Option<u32>,
    pub misses: u32,
}

impl HitResultParams for TaikoHitResultParams {
    type HitResults = TaikoHitResults;
}
