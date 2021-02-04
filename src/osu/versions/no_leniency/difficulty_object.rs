use super::OsuObject;

pub(crate) struct DifficultyObject<'h> {
    pub(crate) base: &'h OsuObject,
    pub(crate) prev: Option<(f32, f32)>, // (jump_dist, strain_time)

    pub(crate) jump_dist: f32,
    pub(crate) travel_dist: f32,
    pub(crate) angle: Option<f32>,

    pub(crate) delta: f32,
    pub(crate) strain_time: f32,
}

impl<'h> DifficultyObject<'h> {
    pub(crate) fn new(
        base: &'h OsuObject,
        prev: &OsuObject,
        prev_vals: Option<(f32, f32)>, // (jump_dist, strain_time)
        prev_prev: Option<OsuObject>,
        clock_rate: f32,
        scaling_factor: f32,
    ) -> Self {
        let delta = (base.time - prev.time) / clock_rate;
        let strain_time = delta.max(50.0);

        let pos = base.pos;
        let travel_dist = prev.travel_dist.unwrap_or(0.0);
        let prev_cursor_pos = prev.end_pos;

        let jump_dist = if base.is_spinner() {
            0.0
        } else {
            ((pos - prev_cursor_pos) * scaling_factor).length()
        };

        let angle = prev_prev.map(|prev_prev| {
            let prev_prev_cursor_pos = prev_prev.end_pos;

            let v1 = prev_prev_cursor_pos - prev.pos;
            let v2 = pos - prev_cursor_pos;

            let dot = v1.dot(v2);
            let det = v1.x * v2.y - v1.y * v2.x;

            det.atan2(dot).abs()
        });

        Self {
            base,
            prev: prev_vals,

            jump_dist,
            travel_dist,
            angle,

            delta,
            strain_time,
        }
    }
}
