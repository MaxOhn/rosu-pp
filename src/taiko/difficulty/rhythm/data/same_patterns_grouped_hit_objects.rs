use crate::{
    taiko::difficulty::object::TaikoDifficultyObject,
    util::sync::{RefCount, Weak},
};

use super::same_rhythm_hit_object_grouping::SameRhythmHitObjectGrouping;

#[derive(Debug)]
pub struct SamePatternsGroupedHitObjects {
    pub groups: Vec<Weak<SameRhythmHitObjectGrouping>>,
    pub previous: Option<Weak<Self>>,
}

impl SamePatternsGroupedHitObjects {
    pub const fn new(
        previous: Option<Weak<Self>>,
        groups: Vec<Weak<SameRhythmHitObjectGrouping>>,
    ) -> Self {
        Self { groups, previous }
    }

    pub fn group_interval(&self) -> Option<f64> {
        self.groups
            .get(1)
            .unwrap_or(&self.groups[0])
            .upgrade()
            .map(|grouped| grouped.get().interval)
    }

    pub fn interval_ratio(&self) -> f64 {
        self.group_interval()
            .zip(
                self.previous
                    .as_ref()
                    .and_then(Weak::upgrade)
                    .and_then(|prev| prev.get().group_interval()),
            )
            .map_or(1.0, |(this, prev)| this / prev)
    }

    pub fn first_hit_object(&self) -> Option<RefCount<TaikoDifficultyObject>> {
        self.groups
            .first()
            .and_then(Weak::upgrade)
            .and_then(|group| group.get().first_hit_object())
    }

    pub fn upgraded_groups(
        &self,
    ) -> impl Iterator<Item = RefCount<SameRhythmHitObjectGrouping>> + use<'_> {
        self.groups.iter().filter_map(Weak::upgrade)
    }
}
