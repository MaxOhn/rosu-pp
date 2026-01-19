use crate::{
    any::hitresult_generator::HitResultParams,
    osu::{OsuHitResults, OsuScoreOrigin},
};

mod closest;
mod fast;

/// TODO: docs
pub struct OsuHitResultParams {
    // Guaranteed to be correct
    pub total_hits: u32,
    pub origin: OsuScoreOrigin,

    // Provided by user; may be infeasable
    pub acc: f64,
    pub large_tick_hits: Option<u32>,
    pub small_tick_hits: Option<u32>,
    pub slider_end_hits: Option<u32>,
    pub n300: Option<u32>,
    pub n100: Option<u32>,
    pub n50: Option<u32>,
    pub misses: u32,
}

impl HitResultParams for OsuHitResultParams {
    type HitResults = OsuHitResults;
}
