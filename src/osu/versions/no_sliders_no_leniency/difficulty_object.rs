use super::OsuObject;

pub(crate) struct DifficultyObject<'h> {
    pub(crate) base: &'h OsuObject,
    pub(crate) prev: Option<(f32, f32)>, // (jump_dist, strain_time)

    pub(crate) jump_dist: f32,
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
        scaling_factor: f32,
    ) -> Self {
        let delta = base.time - prev.time;

        // Capped to 25ms to prevent difficulty calculation breaking from simultaneous objects
        let strain_time = delta.max(25.0);

        // We don't need to calculate either angle or distance
        // when one of the last->curr objects is a spinner
        let (jump_dist, angle) = if base.is_spinner {
            (0.0, None)
        } else {
            let jump_dist = ((base.pos - prev.pos) * scaling_factor).length();

            let angle = prev_prev.map(|prev_prev| {
                let v1 = prev_prev.pos - prev.pos;
                let v2 = base.pos - prev.pos;

                let dot = v1.dot(v2);
                let det = v1.x * v2.y - v1.y * v2.x;

                det.atan2(dot).abs()
            });

            (jump_dist, angle)
        };

        Self {
            base,
            prev: prev_vals,

            jump_dist,
            angle,

            delta,
            strain_time,
        }
    }
}
