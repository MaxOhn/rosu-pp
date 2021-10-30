use crate::parse::{HitObject, HitObjectKind, Pos2};

pub(crate) struct OsuObject {
    pub(crate) pos: Pos2,
    pub(crate) time: f32,
    pub(crate) is_spinner: bool,
    pub(crate) is_slider: bool,
}

impl OsuObject {
    #[inline]
    pub(crate) fn from(h: &HitObject, clock_rate: f32) -> Option<Self> {
        match &h.kind {
            HitObjectKind::Circle => Some(Self {
                pos: h.pos,
                time: h.start_time / clock_rate,
                is_spinner: false,
                is_slider: false,
            }),
            HitObjectKind::Slider { .. } => Some(Self {
                pos: h.pos,
                time: h.start_time / clock_rate,
                is_spinner: false,
                is_slider: true,
            }),
            HitObjectKind::Spinner { .. } => Some(Self {
                pos: h.pos,
                time: h.start_time / clock_rate,
                is_spinner: true,
                is_slider: false,
            }),
            HitObjectKind::Hold { .. } => None,
        }
    }
}
