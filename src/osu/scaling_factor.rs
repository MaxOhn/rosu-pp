use crate::parse::Pos2;

use super::NORMALIZED_RADIUS;

const OBJECT_RADIUS: f32 = 64.0;

pub(crate) struct ScalingFactor {
    adjusted_factor: f32,
    factor: f32,
    radius: f32,
    scale: f32,
}

impl ScalingFactor {
    pub(crate) fn new(cs: f64) -> Self {
        let scale = (1.0 - 0.7 * (cs as f32 - 5.0) / 5.0) / 2.0;

        let radius = OBJECT_RADIUS * scale;
        let factor = NORMALIZED_RADIUS / radius;

        let adjusted_factor = if radius < 30.0 {
            factor * (1.0 + (30.0 - radius).min(5.0) / 50.0)
        } else {
            factor
        };

        Self {
            adjusted_factor,
            factor,
            radius,
            scale: scale * -6.4,
        }
    }

    pub(crate) fn raw(&self) -> f64 {
        self.factor as f64
    }

    pub(crate) fn adjusted(&self) -> f32 {
        self.adjusted_factor
    }

    pub(crate) fn radius(&self) -> f32 {
        self.radius
    }

    pub(crate) fn stack_offset(&self, stack_height: f32) -> Pos2 {
        Pos2::new(stack_height * self.scale)
    }
}
