use std::cmp;

use crate::{
    Difficulty,
    any::InspectablePerformance,
    catch::{Catch, CatchDifficultyAttributes},
};

/// Inspectable [`CatchPerformance`] to expose all of its internal details.
///
/// [`CatchPerformance`]: crate::catch::performance::CatchPerformance
#[derive(Clone, Debug)]
pub struct InspectCatchPerformance<'a> {
    pub attrs: &'a CatchDifficultyAttributes,
    pub difficulty: &'a Difficulty,
    pub acc: Option<f64>,
    pub combo: Option<u32>,
    pub fruits: Option<u32>,
    pub droplets: Option<u32>,
    pub tiny_droplets: Option<u32>,
    pub tiny_droplet_misses: Option<u32>,
    pub misses: Option<u32>,
}

impl InspectCatchPerformance<'_> {
    /// Returns the clamped number of fruit and droplet misses.
    pub fn misses(&self) -> u32 {
        self.misses.map_or(0, |n| {
            cmp::min(n, self.attrs.n_fruits + self.attrs.n_droplets)
        })
    }

    /// Returns the total number of objects in the map.
    pub const fn total_objects(&self) -> u32 {
        self.attrs.n_fruits + self.attrs.n_droplets + self.attrs.n_tiny_droplets
    }
}

impl InspectablePerformance for Catch {
    type InspectPerformance<'a> = InspectCatchPerformance<'a>;

    fn inspect_performance<'a>(
        perf: &'a Self::Performance<'_>,
        attrs: &'a Self::DifficultyAttributes,
    ) -> Self::InspectPerformance<'a> {
        InspectCatchPerformance {
            attrs,
            difficulty: &perf.difficulty,
            acc: perf.acc,
            combo: perf.combo,
            fruits: perf.fruits,
            droplets: perf.droplets,
            tiny_droplets: perf.tiny_droplets,
            tiny_droplet_misses: perf.tiny_droplet_misses,
            misses: perf.misses,
        }
    }
}
