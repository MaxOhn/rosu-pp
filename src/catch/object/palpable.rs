pub struct PalpableObject {
    pub x: f32,
    pub x_offset: f32,
    pub start_time: f64,
    pub dist_to_hyper_dash: f32,
    pub hyper_dash: bool,
}

impl PalpableObject {
    pub const fn new(x: f32, x_offset: f32, start_time: f64) -> Self {
        Self {
            x,
            x_offset,
            start_time,
            dist_to_hyper_dash: 0.0,
            hyper_dash: false,
        }
    }

    pub fn effective_x(&self) -> f32 {
        self.x + self.x_offset
    }
}
