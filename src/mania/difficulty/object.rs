use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    mania::object::ManiaObject,
    util::sync::RefCount,
};

pub struct ManiaDifficultyObject {
    pub idx: usize,
    pub delta_time: f64,
    pub start_time: f64,
    pub end_time: f64,
    pub column: usize,
    pub prev_hit_objects: Box<[Option<RefCount<Self>>]>,
    pub column_strain_time: f64,
}

impl ManiaDifficultyObject {
    pub fn new(
        base: &ManiaObject,
        last_object: &ManiaObject,
        clock_rate: f64,
        objects: &[RefCount<Self>],
        per_column_objects: &[Vec<RefCount<Self>>],
    ) -> Self {
        let idx = objects.len();

        let start_time = base.start_time / clock_rate;
        let delta_time = (base.start_time - last_object.start_time) / clock_rate;
        let end_time = base.end_time / clock_rate;

        let total_columns = per_column_objects.len();
        let column = base.column;
        let column_idx = per_column_objects[column].len();
        let mut prev_hit_objects = vec![None; total_columns].into_boxed_slice();

        let column_strain_time = start_time
            - Self::prev_in_column(0, column_idx, column, per_column_objects)
                .map_or(start_time, |h| h.get().start_time);

        if idx > 0 {
            let prev_note = &objects[idx - 1];
            let prev_note_ref = prev_note.get();

            prev_hit_objects.clone_from(&prev_note_ref.prev_hit_objects);

            // * intentionally depends on processing order to match live.
            prev_hit_objects[prev_note_ref.column] = Some(RefCount::clone(prev_note));
        }

        Self {
            idx,
            delta_time,
            start_time,
            end_time,
            column,
            prev_hit_objects,
            column_strain_time,
        }
    }

    fn prev_in_column(
        backwards_idx: usize,
        column_idx: usize,
        column: usize,
        per_column_objects: &[Vec<RefCount<Self>>],
    ) -> Option<&RefCount<Self>> {
        column_idx
            .checked_sub(backwards_idx + 1)
            .and_then(|idx| per_column_objects[column].get(idx))
    }
}

impl IDifficultyObject for ManiaDifficultyObject {
    type DifficultyObjects = [RefCount<Self>];

    fn idx(&self) -> usize {
        self.idx
    }
}

impl HasStartTime for RefCount<ManiaDifficultyObject> {
    fn start_time(&self) -> f64 {
        self.get().start_time
    }
}
