use super::{closest_rhythm, HitObjectRhythm};
use crate::parse::HitObject;

#[derive(Clone, Debug)]
pub(crate) struct DifficultyObject<'o> {
    pub(crate) idx: usize,
    pub(crate) base: &'o HitObject,
    pub(crate) prev: &'o HitObject,
    pub(crate) delta: f64,
    pub(crate) rhythm: &'static HitObjectRhythm,
    pub(crate) start_time: f64,
}

impl<'o> DifficultyObject<'o> {
    #[inline]
    pub(crate) fn new(
        idx: usize,
        base: &'o HitObject,
        prev: &'o HitObject,
        prev_prev: &HitObject,
        clock_rate: f64,
    ) -> Self {
        let delta = (base.start_time - prev.start_time) / clock_rate;
        let rhythm = closest_rhythm(delta, prev, prev_prev, clock_rate);

        Self {
            idx,
            base,
            prev,
            delta,
            rhythm,
            start_time: base.start_time / clock_rate,
        }
    }
}
