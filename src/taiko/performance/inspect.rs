use std::cmp;

use crate::{
    Difficulty,
    any::{HitResultPriority, InspectablePerformance},
    taiko::{Taiko, TaikoDifficultyAttributes},
};

/// Inspectable [`TaikoPerformance`] to expose all of its internal details.
///
/// [`TaikoPerformance`]: crate::taiko::performance::TaikoPerformance
#[derive(Clone, Debug)]
pub struct InspectTaikoPerformance<'a> {
    pub attrs: &'a TaikoDifficultyAttributes,
    pub difficulty: &'a Difficulty,
    pub combo: Option<u32>,
    pub acc: Option<f64>,
    pub n300: Option<u32>,
    pub n100: Option<u32>,
    pub misses: Option<u32>,
    pub hitresult_priority: HitResultPriority,
}

impl InspectTaikoPerformance<'_> {
    pub const fn max_combo(&self) -> u32 {
        self.attrs.max_combo()
    }

    pub fn total_hits(&self) -> u32 {
        cmp::min(
            self.difficulty.get_passed_objects() as u32,
            self.max_combo(),
        )
    }

    pub fn misses(&self) -> u32 {
        self.misses.map_or(0, |n| cmp::min(n, self.total_hits()))
    }
}

impl InspectablePerformance for Taiko {
    type InspectPerformance<'a> = InspectTaikoPerformance<'a>;

    fn inspect_performance<'a>(
        perf: &'a Self::Performance<'_>,
        attrs: &'a Self::DifficultyAttributes,
    ) -> Self::InspectPerformance<'a> {
        InspectTaikoPerformance {
            attrs,
            difficulty: &perf.difficulty,
            combo: perf.combo,
            acc: perf.acc,
            n300: perf.n300,
            n100: perf.n100,
            misses: perf.misses,
            hitresult_priority: perf.hitresult_priority,
        }
    }
}
