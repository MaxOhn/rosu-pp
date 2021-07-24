use crate::parse::Pos2;

pub(crate) struct OsuObject {
    pub(crate) pos: Pos2,
    pub(crate) time: f32,
    pub(crate) is_spinner: bool,
}

impl OsuObject {
    #[inline]
    pub(crate) fn new(pos: Pos2, time: f32, is_spinner: bool, clock_rate: f32) -> Self {
        Self {
            pos,
            time: time / clock_rate,
            is_spinner,
        }
    }
}
