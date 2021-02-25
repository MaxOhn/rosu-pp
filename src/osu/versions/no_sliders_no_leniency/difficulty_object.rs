use crate::parse::HitObject;

use std::borrow::Cow;

pub(crate) struct DifficultyObject<'h> {
    pub(crate) base: &'h HitObject,
    pub(crate) prev: Option<(f32, f32)>, // (jump_dist, strain_time)

    pub(crate) jump_dist: f32,
    pub(crate) angle: Option<f32>,

    pub(crate) delta: f32,
    pub(crate) strain_time: f32,
}

impl<'h> DifficultyObject<'h> {
    pub(crate) fn new(
        base: &'h HitObject,
        prev: &HitObject,
        prev_vals: Option<(f32, f32)>, // (jump_dist, strain_time)
        prev_prev: Option<Cow<HitObject>>,
        clock_rate: f32,
        scaling_factor: f32,
    ) -> Self {
        let delta = (base.start_time - prev.start_time) / clock_rate;
        let strain_time = delta.max(50.0);

        let jump_dist = if base.is_spinner() {
            0.0
        } else {
            ((base.pos - prev.pos) * scaling_factor).length()
        };

        let angle = prev_prev.map(|prev_prev| {
            let v1 = prev_prev.pos - prev.pos;
            let v2 = base.pos - prev.pos;

            let dot = v1.dot(v2);
            let det = v1.x * v2.y - v1.y * v2.x;

            det.atan2(dot).abs()
        });

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
