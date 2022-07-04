use super::{closest_rhythm, taiko_object::TaikoObject, HitObjectRhythm};

#[derive(Clone, Debug)]
pub(crate) struct DifficultyObject<'o> {
    pub(crate) idx: usize,
    pub(crate) base: TaikoObject<'o>,
    pub(crate) prev: TaikoObject<'o>,
    pub(crate) delta: f64,
    pub(crate) rhythm: &'static HitObjectRhythm,
    pub(crate) start_time: f64,
}

impl<'o> DifficultyObject<'o> {
    #[inline]
    pub(crate) fn new(
        idx: usize,
        base: TaikoObject<'o>,
        prev: TaikoObject<'o>,
        prev_prev: TaikoObject<'o>,
        clock_rate: f64,
    ) -> Self {
        let delta = (base.h.start_time - prev.h.start_time) / clock_rate;
        let rhythm = closest_rhythm(delta, prev.h, prev_prev.h, clock_rate);

        Self {
            idx,
            base,
            prev,
            delta,
            rhythm,
            start_time: base.h.start_time / clock_rate,
        }
    }
}
