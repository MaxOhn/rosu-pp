use super::mania_object::ManiaObject;

pub(crate) struct ManiaDifficultyObject<'h> {
    pub(crate) idx: usize,
    pub(crate) base: ManiaObject<'h>,
    pub(crate) delta_time: f64,
    pub(crate) start_time: f64,
    pub(crate) end_time: f64,
}

impl<'h> ManiaDifficultyObject<'h> {
    pub(crate) fn new(
        base: ManiaObject<'h>,
        last: ManiaObject<'h>,
        clock_rate: f64,
        idx: usize,
    ) -> Self {
        let delta_time = (base.start_time() - last.start_time()) / clock_rate;
        let start_time = base.start_time() / clock_rate;
        let end_time = base.end_time() / clock_rate;

        Self {
            idx,
            base,
            delta_time,
            start_time,
            end_time,
        }
    }
}
