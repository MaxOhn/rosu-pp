use rosu_map::util::Pos;

use crate::osu::object::OsuObject;

use super::object::OsuDifficultyObject;

/// Fields around the scaling of hit objects.
///
/// osu!lazer stores these in each hit object but since all objects share the
/// same scaling (w.r.t. difficulty & performance), we store them only once.
pub struct ScalingFactor {
    /// `NORMALIZED_RADIUS / Radius` and then adjusted if `Radius < 30`
    pub factor: f32,
    pub radius: f64,
    pub scale: f32,
}

impl ScalingFactor {
    pub fn new(cs: f64) -> Self {
        let scale = (1.0 - 0.7 * (cs as f32 - 5.0) / 5.0) / 2.0;

        let radius = f64::from(OsuObject::OBJECT_RADIUS * scale);
        let factor = OsuDifficultyObject::NORMALIZED_RADIUS / radius as f32;

        let factor_with_small_circle_bonus = if radius < 30.0 {
            factor * (1.0 + (30.0 - radius as f32).min(5.0) / 50.0)
        } else {
            factor
        };

        Self {
            factor: factor_with_small_circle_bonus,
            radius,
            scale,
        }
    }

    pub fn stack_offset(&self, stack_height: i32) -> Pos {
        let stack_offset = stack_height as f32 * self.scale * -6.4;

        Pos::new(stack_offset, stack_offset)
    }
}
