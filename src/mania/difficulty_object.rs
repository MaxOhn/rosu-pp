use super::mania_object::ManiaObject;

#[derive(Clone, Debug)]
pub(crate) struct ManiaDifficultyObject {
    pub(crate) idx: usize,
    pub(crate) base_column: usize,
    pub(crate) delta_time: f64,
    pub(crate) start_time: f64,
    pub(crate) end_time: f64,
}

impl ManiaDifficultyObject {
    pub(crate) fn new(base: &ManiaObject, last: &ManiaObject, clock_rate: f64, idx: usize) -> Self {
        Self {
            idx,
            base_column: base.column,
            delta_time: (base.start_time - last.start_time) / clock_rate,
            start_time: base.start_time / clock_rate,
            end_time: base.end_time / clock_rate,
        }
    }
}
