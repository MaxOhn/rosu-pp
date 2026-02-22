use std::cmp;

use crate::{
    Difficulty,
    any::{HitResultPriority, InspectablePerformance},
    mania::{Mania, ManiaDifficultyAttributes},
};

/// Inspectable [`ManiaPerformance`] to expose all of its internal details.
///
/// [`ManiaPerformance`]: crate::mania::performance::ManiaPerformance
#[derive(Clone, Debug)]
pub struct InspectManiaPerformance<'a> {
    pub attrs: &'a ManiaDifficultyAttributes,
    pub difficulty: &'a Difficulty,
    pub n320: Option<u32>,
    pub n300: Option<u32>,
    pub n200: Option<u32>,
    pub n100: Option<u32>,
    pub n50: Option<u32>,
    pub misses: Option<u32>,
    pub acc: Option<f64>,
    pub hitresult_priority: HitResultPriority,
}

impl InspectManiaPerformance<'_> {
    pub fn total_hits(&self) -> u32 {
        let passed_objects = self.difficulty.get_passed_objects() as u32;
        let total_hits = cmp::min(passed_objects, self.attrs.n_objects);

        if self.is_classic() {
            total_hits
        } else {
            // Note that we don't consider `passed_objects` here. Unsure if
            // that's the correct behavior.
            total_hits + self.attrs.n_hold_notes
        }
    }

    pub fn misses(&self) -> u32 {
        self.misses.map_or(0, |n| cmp::min(n, self.total_hits()))
    }

    pub fn is_classic(&self) -> bool {
        !self.difficulty.get_lazer() || self.difficulty.get_mods().cl()
    }
}

impl InspectablePerformance for Mania {
    type InspectPerformance<'a> = InspectManiaPerformance<'a>;

    fn inspect_performance<'a>(
        perf: &'a Self::Performance<'_>,
        attrs: &'a Self::DifficultyAttributes,
    ) -> Self::InspectPerformance<'a> {
        InspectManiaPerformance {
            attrs,
            difficulty: &perf.difficulty,
            n320: perf.n320,
            n300: perf.n300,
            n200: perf.n200,
            n100: perf.n100,
            n50: perf.n50,
            misses: perf.misses,
            acc: perf.acc,
            hitresult_priority: perf.hitresult_priority,
        }
    }
}
