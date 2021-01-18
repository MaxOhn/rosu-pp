use super::{PathType, Pos2};

use std::cmp::Ordering;

/// "Intermediate" hitobject created through parsing.
/// Each mode will handle them differently.
#[derive(Clone, Debug, PartialEq)]
pub struct HitObject {
    pub pos: Pos2,
    pub start_time: f32,
    pub kind: HitObjectKind,
    pub sound: u8,
}

impl HitObject {
    #[inline]
    pub fn end_time(&self) -> f32 {
        match &self.kind {
            HitObjectKind::Circle { .. } => self.start_time,
            // incorrect, only called in mania which has no sliders though
            HitObjectKind::Slider { .. } => self.start_time,
            HitObjectKind::Spinner { end_time } => *end_time,
            HitObjectKind::Hold { end_time, .. } => *end_time,
        }
    }

    #[inline]
    pub fn is_circle(&self) -> bool {
        matches!(self.kind, HitObjectKind::Circle { .. })
    }

    #[inline]
    pub fn is_slider(&self) -> bool {
        matches!(self.kind, HitObjectKind::Slider { .. })
    }

    #[inline]
    pub fn is_spinner(&self) -> bool {
        matches!(self.kind, HitObjectKind::Spinner { .. })
    }
}

impl PartialOrd for HitObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_time.partial_cmp(&other.start_time)
    }
}

/// Further data related to specific object types.
#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    Circle,
    Slider {
        pixel_len: f32,
        repeats: usize,
        curve_points: Vec<Pos2>,
        path_type: PathType,
    },
    Spinner {
        end_time: f32,
    },
    Hold {
        end_time: f32,
    },
}
