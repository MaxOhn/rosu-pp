use crate::{
    taiko::difficulty::{
        object::{TaikoDifficultyObject, TaikoDifficultyObjects},
        utils::interval_grouping::group_by_interval,
    },
    util::sync::RefCount,
};

use super::data::{
    same_patterns_grouped_hit_objects::SamePatternsGroupedHitObjects,
    same_rhythm_hit_object_grouping::SameRhythmHitObjectGrouping,
};

pub struct RhythmDifficultyPreprocessor;

impl RhythmDifficultyPreprocessor {
    pub fn process_and_assign(hit_objects: &TaikoDifficultyObjects) {
        let rhythm_groups = create_same_rhythm_grouped_hit_objects(&hit_objects.note_objects);

        for rhythm_group in rhythm_groups.iter() {
            for hit_object in rhythm_group.get().hit_objects.iter() {
                if let Some(hit_object) = hit_object.upgrade() {
                    hit_object
                        .get_mut()
                        .rhythm_data
                        .same_rhythm_grouped_hit_objects = Some(RefCount::clone(rhythm_group));
                }
            }
        }

        let pattern_groups = create_same_pattern_grouped_hit_objects(&rhythm_groups);

        for pattern_group in pattern_groups {
            for group in pattern_group.get().upgraded_groups() {
                for hit_object in group.get().upgraded_hit_objects() {
                    hit_object
                        .get_mut()
                        .rhythm_data
                        .same_patterns_grouped_hit_objects = Some(RefCount::clone(&pattern_group));
                }
            }
        }
    }
}

fn create_same_rhythm_grouped_hit_objects(
    hit_objects: &[RefCount<TaikoDifficultyObject>],
) -> Vec<RefCount<SameRhythmHitObjectGrouping>> {
    let mut rhythm_groups = Vec::new();

    for grouped in group_by_interval(hit_objects) {
        rhythm_groups.push(RefCount::new(SameRhythmHitObjectGrouping::new(
            rhythm_groups.last().map(RefCount::downgrade),
            grouped,
        )));
    }

    rhythm_groups
}

fn create_same_pattern_grouped_hit_objects(
    rhythm_groups: &[RefCount<SameRhythmHitObjectGrouping>],
) -> impl Iterator<Item = RefCount<SamePatternsGroupedHitObjects>> + use<'_> {
    group_by_interval(rhythm_groups).scan(None, |prev, grouped| {
        let curr = RefCount::new(SamePatternsGroupedHitObjects::new(prev.take(), grouped));
        *prev = Some(curr.downgrade());

        Some(curr)
    })
}
