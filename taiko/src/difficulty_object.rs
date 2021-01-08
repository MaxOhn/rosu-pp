use super::{closest_rhythm, HitObjectRhythm};

use parse::HitObject;

#[derive(Clone, Debug)]
pub(crate) struct DifficultyObject<'o> {
    pub(crate) idx: usize,
    pub(crate) base: &'o HitObject,
    pub(crate) prev: &'o HitObject,
    pub(crate) delta: f32,
    pub(crate) rhythm: &'static HitObjectRhythm,
}

impl<'o> DifficultyObject<'o> {
    #[inline]
    pub(crate) fn new(
        idx: usize,
        base: &'o HitObject,
        prev: &'o HitObject,
        prev_prev: &HitObject,
        clock_rate: f32,
    ) -> Self {
        let delta = (base.start_time - prev.start_time) / clock_rate;
        let rhythm = closest_rhythm(delta, prev, prev_prev, clock_rate);

        Self {
            idx,
            base,
            prev,
            delta,
            rhythm,
        }
    }
}
