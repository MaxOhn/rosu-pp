use std::mem;

use rosu_map::util::Pos;

use crate::model::hit_object::{HitObject, HitObjectKind, HoldNote};

use super::pattern_generator::{
    end_time_object::EndTimeObjectPatternGenerator, hit_object::HitObjectPatternGenerator,
    path_object::PathObjectPatternGenerator,
};

#[derive(Default)]
pub struct Pattern {
    pub hit_objects: Vec<HitObject>,
    contained_columns: ContainedColumns,
}

impl Pattern {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            hit_objects: Vec::with_capacity(capacity),
            contained_columns: ContainedColumns::default(),
        }
    }

    fn new_single(hit_object: HitObject, column: u8) -> Self {
        let mut contained_columns = ContainedColumns::default();
        contained_columns.insert(column);
        let hit_objects = vec![hit_object];

        Self {
            hit_objects,
            contained_columns,
        }
    }

    pub fn new_note(generator: &HitObjectPatternGenerator<'_>, column: u8) -> Self {
        let pos = column_to_pos(column, generator.inner.total_columns);

        let hit_object = HitObject {
            pos: Pos::new(pos, pos),
            start_time: generator.inner.hit_object.start_time,
            kind: HitObjectKind::Circle,
        };

        Self::new_single(hit_object, column)
    }

    pub fn add_note(&mut self, generator: &HitObjectPatternGenerator<'_>, column: u8) {
        let pos = column_to_pos(column, generator.inner.total_columns);

        let hit_object = HitObject {
            pos: Pos::new(pos, pos),
            start_time: generator.inner.hit_object.start_time,
            kind: HitObjectKind::Circle,
        };

        self.contained_columns.insert(column);
        self.hit_objects.push(hit_object);
    }

    pub fn new_end_time_note(
        generator: &EndTimeObjectPatternGenerator<'_>,
        column: u8,
        hold_note: bool,
    ) -> Self {
        let pos = column_to_pos(column, generator.inner.total_columns);
        let pos = Pos::new(pos, pos);

        let hit_object = if hold_note {
            HitObject {
                pos,
                start_time: generator.inner.hit_object.start_time,
                kind: HitObjectKind::Hold(HoldNote {
                    duration: generator.end_time - generator.inner.hit_object.start_time,
                }),
            }
        } else {
            HitObject {
                pos,
                start_time: generator.inner.hit_object.start_time,
                kind: HitObjectKind::Circle,
            }
        };

        Self::new_single(hit_object, column)
    }

    pub fn new_slider_note(
        generator: &PathObjectPatternGenerator<'_>,
        column: u8,
        start_time: i32,
        end_time: i32,
    ) -> Self {
        let pos = column_to_pos(column, generator.inner.total_columns);
        let pos = Pos::new(pos, pos);

        let hit_object = if start_time == end_time {
            HitObject {
                pos,
                start_time: f64::from(start_time),
                kind: HitObjectKind::Circle,
            }
        } else {
            let start_time = f64::from(start_time);

            HitObject {
                pos,
                start_time,
                kind: HitObjectKind::Hold(HoldNote {
                    duration: f64::from(end_time) - start_time,
                }),
            }
        };

        Self::new_single(hit_object, column)
    }

    pub fn add_slider_note(
        &mut self,
        generator: &PathObjectPatternGenerator<'_>,
        column: u8,
        start_time: i32,
        end_time: i32,
    ) {
        let pos = column_to_pos(column, generator.inner.total_columns);
        let pos = Pos::new(pos, pos);

        let hit_object = if start_time == end_time {
            HitObject {
                pos,
                start_time: f64::from(start_time),
                kind: HitObjectKind::Circle,
            }
        } else {
            let start_time = f64::from(start_time);

            HitObject {
                pos,
                start_time,
                kind: HitObjectKind::Hold(HoldNote {
                    duration: f64::from(end_time) - start_time,
                }),
            }
        };

        self.contained_columns.insert(column);
        self.hit_objects.push(hit_object);
    }

    pub fn add_object(&mut self, obj: HitObject, column: u8) {
        self.hit_objects.push(obj);
        self.contained_columns.insert(column);
    }

    pub const fn column_has_obj(&self, column: u8) -> bool {
        self.contained_columns.contains(column)
    }

    pub const fn column_with_objs(&self) -> i32 {
        self.contained_columns.len() as i32
    }

    /// Moves all values of `other` into `self`, leaving `other` empty but
    /// keeps the capacity.
    pub fn append(&mut self, other: &mut Self) {
        self.hit_objects.append(&mut other.hit_objects);
        self.contained_columns.append(&mut other.contained_columns);
    }
}

fn column_to_pos(column: u8, total_columns: i32) -> f32 {
    let divisor = 512.0 / total_columns as f32;

    (f32::from(column) * divisor).ceil()
}

#[derive(Copy, Clone, Default)]
pub struct ContainedColumns(u16);

impl ContainedColumns {
    pub const fn insert(&mut self, column: u8) {
        self.0 |= 1 << column;
    }

    pub fn append(&mut self, other: &mut Self) {
        self.0 |= mem::take(&mut other.0);
    }

    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn contains(self, column: u8) -> bool {
        self.0 & (1 << column) != 0
    }
}
