use std::collections::HashSet;

use crate::{
    parse::{HitObject, HitObjectKind, Pos2},
    util::ByteHasher,
};

use super::pattern_generator::{
    distance_object::DistanceObjectPatternGenerator,
    end_time_object::EndTimeObjectPatternGenerator, hit_object::HitObjectPatternGenerator,
};

#[derive(Default)]
pub(crate) struct Pattern {
    pub(crate) hit_objects: Vec<HitObject>,
    contained_columns: HashSet<u8, ByteHasher>,
}

impl Pattern {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            hit_objects: Vec::with_capacity(capacity),
            contained_columns: HashSet::with_hasher(ByteHasher),
        }
    }

    fn new_single(hit_object: HitObject, column: u8) -> Self {
        let mut contained_columns = HashSet::with_capacity_and_hasher(1, ByteHasher);
        contained_columns.insert(column);
        let hit_objects = vec![hit_object];

        Self {
            hit_objects,
            contained_columns,
        }
    }

    pub(crate) fn new_note(generator: &HitObjectPatternGenerator<'_>, column: u8) -> Self {
        let hit_object = HitObject {
            pos: Pos2::new(column_to_pos(column, generator.total_columns)),
            start_time: generator.hit_object.start_time,
            kind: HitObjectKind::Circle,
        };

        Self::new_single(hit_object, column)
    }

    pub(crate) fn add_note(&mut self, generator: &HitObjectPatternGenerator<'_>, column: u8) {
        let hit_object = HitObject {
            pos: Pos2::new(column_to_pos(column, generator.total_columns)),
            start_time: generator.hit_object.start_time,
            kind: HitObjectKind::Circle,
        };

        self.contained_columns.insert(column);
        self.hit_objects.push(hit_object);
    }

    pub(crate) fn new_end_time_note(
        generator: &EndTimeObjectPatternGenerator<'_>,
        column: u8,
        hold_note: bool,
    ) -> Self {
        let pos = Pos2::new(column_to_pos(column, generator.total_columns));

        let hit_object = if hold_note {
            HitObject {
                pos,
                start_time: generator.hit_object.start_time,
                kind: HitObjectKind::Hold {
                    end_time: generator.end_time,
                },
            }
        } else {
            HitObject {
                pos,
                start_time: generator.hit_object.start_time,
                kind: HitObjectKind::Circle,
            }
        };

        Self::new_single(hit_object, column)
    }

    pub(crate) fn new_slider_note(
        generator: &DistanceObjectPatternGenerator<'_>,
        column: u8,
        start_time: i32,
        end_time: i32,
    ) -> Self {
        let pos = Pos2::new(column_to_pos(column, generator.total_columns));

        let hit_object = if start_time == end_time {
            HitObject {
                pos,
                start_time: start_time as f64,
                kind: HitObjectKind::Circle,
            }
        } else {
            HitObject {
                pos,
                start_time: start_time as f64,
                kind: HitObjectKind::Hold {
                    end_time: end_time as f64,
                },
            }
        };

        Self::new_single(hit_object, column)
    }

    pub(crate) fn add_slider_note(
        &mut self,
        generator: &DistanceObjectPatternGenerator<'_>,
        column: u8,
        start_time: i32,
        end_time: i32,
    ) {
        let pos = Pos2::new(column_to_pos(column, generator.total_columns));

        let hit_object = if start_time == end_time {
            HitObject {
                pos,
                start_time: start_time as f64,
                kind: HitObjectKind::Circle,
            }
        } else {
            HitObject {
                pos,
                start_time: start_time as f64,
                kind: HitObjectKind::Hold {
                    end_time: end_time as f64,
                },
            }
        };

        self.contained_columns.insert(column);
        self.hit_objects.push(hit_object);
    }

    pub(crate) fn add_object(&mut self, obj: HitObject, column: u8) {
        self.hit_objects.push(obj);
        self.contained_columns.insert(column);
    }

    pub(crate) fn column_has_obj(&self, column: u8) -> bool {
        self.contained_columns.contains(&column)
    }

    pub(crate) fn column_with_objs(&self) -> i32 {
        self.contained_columns.len() as i32
    }

    /// Moves all values of `other` into `self`,
    /// leaving `other` empty but keeps the capacities.
    pub(crate) fn append(&mut self, other: &mut Self) {
        self.hit_objects.append(&mut other.hit_objects);
        self.contained_columns
            .extend(other.contained_columns.drain());
    }
}

fn column_to_pos(column: u8, total_columns: i32) -> f32 {
    let divisor = 512.0 / total_columns as f32;

    (column as f32 * divisor).ceil()
}
