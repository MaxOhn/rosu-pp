use super::CatchObject;

const NORMALIZED_HITOBJECT_RADIUS: f32 = 41.0;

pub(crate) struct DifficultyObject<'o> {
    pub(crate) base: &'o CatchObject,
    pub(crate) last: &'o CatchObject,

    pub(crate) delta: f64,
    pub(crate) start_time: f64,

    pub(crate) normalized_pos: f32,
    pub(crate) last_normalized_pos: f32,

    pub(crate) strain_time: f64,
    pub(crate) clock_rate: f64,
}

impl<'o> DifficultyObject<'o> {
    #[inline]
    pub(crate) fn new(
        base: &'o CatchObject,
        last: &'o CatchObject,
        half_catcher_width: f32,
        clock_rate: f64,
    ) -> Self {
        let delta = (base.time - last.time) / clock_rate;
        let start_time = base.time / clock_rate;
        let strain_time = delta.max(40.0);

        let scaling_factor = NORMALIZED_HITOBJECT_RADIUS / half_catcher_width;
        let normalized_pos = base.pos * scaling_factor;
        let last_normalized_pos = last.pos * scaling_factor;

        Self {
            base,
            last,
            delta,
            start_time,
            normalized_pos,
            last_normalized_pos,
            strain_time,
            clock_rate,
        }
    }
}
