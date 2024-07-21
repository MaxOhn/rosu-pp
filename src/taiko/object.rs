use rosu_map::section::hit_objects::hit_samples::HitSoundType;

use crate::model::hit_object::HitObject;

pub struct TaikoObject {
    pub start_time: f64,
    pub hit_type: HitType,
}

impl TaikoObject {
    pub const fn new(h: &HitObject, sound: HitSoundType) -> Self {
        Self {
            start_time: h.start_time,
            hit_type: if !h.is_circle() {
                HitType::NonHit
            } else if sound.has_flag(HitSoundType::CLAP | HitSoundType::WHISTLE) {
                HitType::Rim
            } else {
                HitType::Center
            },
        }
    }

    pub const fn is_hit(&self) -> bool {
        self.hit_type.is_hit()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HitType {
    Center,
    Rim,
    NonHit,
}

impl HitType {
    pub const fn is_hit(self) -> bool {
        !matches!(self, Self::NonHit)
    }
}
