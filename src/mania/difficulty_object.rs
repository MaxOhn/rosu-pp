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
    pub(crate) fn new(
        base: ManiaObject<'_>,
        last: ManiaObject<'_>,
        clock_rate: f64,
        total_columns: f32,
        idx: usize,
    ) -> Self {
        let delta_time = (base.start_time() - last.start_time()) / clock_rate;
        let start_time = base.start_time() / clock_rate;
        let end_time = base.end_time() / clock_rate;
        let base_column = base.column(total_columns);

        Self {
            idx,
            base_column,
            delta_time,
            start_time,
            end_time,
        }
    }
}
