/// A break point of a [`Beatmap`](crate::beatmap::Beatmap).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Break {
    /// Start timestamp of the break.
    pub start_time: f64,
    /// End timestamp of the break.
    pub end_time: f64,
}

impl Break {
    /// Duration of the break.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }
}
