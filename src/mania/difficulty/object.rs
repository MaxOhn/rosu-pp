use crate::{any::difficulty::object::IDifficultyObject, mania::object::ManiaObject};

pub struct ManiaDifficultyObject {
    pub idx: usize,
    pub base_column: usize,
    pub delta_time: f64,
    pub start_time: f64,
    pub end_time: f64,
}

impl ManiaDifficultyObject {
    pub fn new(base: &ManiaObject, last: &ManiaObject, clock_rate: f64, idx: usize) -> Self {
        Self {
            idx,
            base_column: base.column,
            delta_time: (base.start_time - last.start_time) / clock_rate,
            start_time: base.start_time / clock_rate,
            end_time: base.end_time / clock_rate,
        }
    }
}

impl IDifficultyObject for ManiaDifficultyObject {
    fn idx(&self) -> usize {
        self.idx
    }
}
