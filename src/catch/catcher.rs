pub struct Catcher;

const AREA_CATCHER_SIZE: f32 = 106.75;

impl Catcher {
    pub const BASE_SPEED: f64 = 1.0;
    pub const ALLOWED_CATCH_RANGE: f32 = 0.8;

    pub fn calculate_catch_width(cs: f32) -> f32 {
        Self::calculate_catch_width_by_scale(Self::calculate_scale(cs))
    }

    fn calculate_catch_width_by_scale(scale: f32) -> f32 {
        AREA_CATCHER_SIZE * scale.abs() * Self::ALLOWED_CATCH_RANGE
    }

    fn calculate_scale(cs: f32) -> f32 {
        ((1.0 - 0.7 * ((f64::from(cs) - 5.0) / 5.0)) as f32 / 2.0 * 1.0) * 2.0
    }
}
