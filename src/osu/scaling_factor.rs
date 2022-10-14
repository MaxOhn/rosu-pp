use crate::parse::Pos2;

use super::NORMALIZED_RADIUS;

const OBJECT_RADIUS: f32 = 64.0;

#[derive(Copy, Clone, Debug)]
pub(crate) struct ScalingFactor {
    pub(crate) factor: f32,
    pub(crate) radius: f32,
    scale: f32,
}

impl ScalingFactor {
    pub(crate) fn new(cs: f64) -> Self {
        let scale = (1.0 - 0.7 * (cs as f32 - 5.0) / 5.0) / 2.0;

        let radius = OBJECT_RADIUS * scale;
        let factor = NORMALIZED_RADIUS / radius;

        let factor = if radius < 30.0 {
            factor * (1.0 + (30.0 - radius).min(5.0) / 50.0)
        } else {
            factor
        };

        Self {
            factor,
            radius,
            scale,
        }
    }

    pub(crate) fn stack_offset(&self, stack_height: f32) -> Pos2 {
        Pos2::new(stack_height * self.scale * -6.4)
    }
}
