/// Effect-related info about this control point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EffectPoint {
    pub time: f64,
    pub kiai: bool,
}

impl EffectPoint {
    pub const DEFAULT_KIAI: bool = rosu_map::section::timing_points::EffectPoint::DEFAULT_KIAI;

    pub const fn new(time: f64, kiai: bool) -> Self {
        Self { time, kiai }
    }

    pub const fn is_redundant(&self, existing: &Self) -> bool {
        self.kiai == existing.kiai
    }
}

impl Default for EffectPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            kiai: Self::DEFAULT_KIAI,
        }
    }
}

pub fn effect_point_at(points: &[EffectPoint], time: f64) -> Option<&EffectPoint> {
    points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .map_or_else(|i| i.checked_sub(1), Some)
        .map(|i| &points[i])
}
