use crate::util::sync::RefCount;

use super::data::{
    same_patterns_grouped_hit_objects::SamePatternsGroupedHitObjects,
    same_rhythm_hit_object_grouping::SameRhythmHitObjectGrouping,
};

#[derive(Debug)]
pub struct RhythmData {
    pub same_rhythm_grouped_hit_objects: Option<RefCount<SameRhythmHitObjectGrouping>>,
    pub same_patterns_grouped_hit_objects: Option<RefCount<SamePatternsGroupedHitObjects>>,
    pub ratio: f64,
}

impl RhythmData {
    pub fn new(delta_time: f64, prev_delta_time: Option<f64>) -> Self {
        let Some(prev_delta_time) = prev_delta_time else {
            return Self {
                same_rhythm_grouped_hit_objects: None,
                same_patterns_grouped_hit_objects: None,
                ratio: 1.0,
            };
        };

        let actual_ratio = delta_time / prev_delta_time;

        let actual_diff = |r| f64::abs(r - actual_ratio);

        let closest_ratio = COMMON_RATIOS
            .iter()
            .min_by(|r1, r2| actual_diff(*r1).total_cmp(&actual_diff(*r2)))
            .unwrap();

        Self {
            same_rhythm_grouped_hit_objects: None,
            same_patterns_grouped_hit_objects: None,
            ratio: *closest_ratio,
        }
    }
}

#[expect(clippy::eq_op, reason = "staying in-sync with lazer")]
static COMMON_RATIOS: [f64; 9] = [
    1.0 / 1.0,
    2.0 / 1.0,
    1.0 / 2.0,
    3.0 / 1.0,
    1.0 / 3.0,
    3.0 / 2.0,
    2.0 / 3.0,
    5.0 / 4.0,
    4.0 / 5.0,
];
