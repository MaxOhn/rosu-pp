use super::OsuObject;

const NORMALIZED_RADIUS: f32 = 52.0;

pub(crate) struct DifficultyObject {
    pub(crate) base: OsuObject,
    pub(crate) prev: Option<(f32, f32)>, // (jump_dist, strain_time)

    pub(crate) jump_dist: f32,
    pub(crate) travel_dist: f32,
    pub(crate) angle: Option<f32>,

    pub(crate) delta: f32,
    pub(crate) strain_time: f32,
}

impl DifficultyObject {
    pub(crate) fn new(
        base: OsuObject,
        prev: OsuObject,
        prev_diff: Option<DifficultyObject>,
        prev_prev: Option<OsuObject>,
        clock_rate: f32,
        radius: f32,
    ) -> Self {
        let delta = (base.time() - prev.time()) / clock_rate;
        let strain_time = delta.max(50.0);

        let mut scaling_factor = NORMALIZED_RADIUS / radius;

        if radius < 30.0 {
            let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
            scaling_factor *= 1.0 + small_circle_bonus;
        }

        let pos = base.pos();
        let travel_dist = prev.travel_dist();
        let prev_cursor_pos = prev.pos();

        let jump_dist = if base.is_spinner() {
            0.0
        } else {
            (pos * scaling_factor - prev_cursor_pos * scaling_factor).length()
        };

        let angle = prev_prev.map(|prev_prev| {
            let prev_prev_cursor_pos = prev_prev.pos();

            let v1 = prev_prev_cursor_pos - prev_cursor_pos;
            let v2 = pos - prev_cursor_pos;

            let dot = v1.dot(v2);
            let det = v1.x * v2.y - v1.y * v2.x;

            det.atan2(dot).abs()
        });

        let prev = prev_diff.map(|o| (o.jump_dist, o.strain_time));

        Self {
            base,
            prev,

            jump_dist,
            travel_dist,
            angle,

            delta,
            strain_time,
        }
    }
}
