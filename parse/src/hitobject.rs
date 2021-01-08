use super::{HitObjectKind, Pos2};

use std::cmp::Ordering;

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
            HitObjectKind::Slider { .. } => self.start_time, // wrong but should be unreachable
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
