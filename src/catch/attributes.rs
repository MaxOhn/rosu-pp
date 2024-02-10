use crate::catch::performance::CatchPerformance;

/// The result of a difficulty calculation on an osu!catch map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatchDifficultyAttributes {
    /// The final star rating
    pub stars: f64,
    /// The approach rate.
    pub ar: f64,
    /// The amount of fruits.
    pub n_fruits: u32,
    /// The amount of droplets.
    pub n_droplets: u32,
    /// The amount of tiny droplets.
    pub n_tiny_droplets: u32,
    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub is_convert: bool,
}

impl CatchDifficultyAttributes {
    /// Return the maximum combo.
    pub const fn max_combo(&self) -> u32 {
        self.n_fruits + self.n_droplets
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> CatchPerformance<'a> {
        self.into()
    }
}

/// The result of a performance calculation on an osu!catch map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatchPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: CatchDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
}

impl CatchPerformanceAttributes {
    /// Return the star value.
    pub const fn stars(&self) -> f64 {
        self.difficulty.stars
    }

    /// Return the performance point value.
    pub const fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    pub const fn max_combo(&self) -> u32 {
        self.difficulty.max_combo()
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.difficulty.is_convert
    }
}

pub struct CatchDifficultyAttributesBuilder {
    inner: CatchDifficultyAttributes,
    take: usize,
}

impl CatchDifficultyAttributesBuilder {
    pub const fn new(attrs: CatchDifficultyAttributes, take: usize) -> Self {
        Self { inner: attrs, take }
    }

    pub const fn into_inner(self) -> CatchDifficultyAttributes {
        self.inner
    }

    pub const fn take_more(&self) -> bool {
        self.take > 0
    }

    pub fn inc_fruits(&mut self) {
        Self::inc_value(&mut self.inner.n_fruits, &mut self.take);
    }

    pub fn inc_droplets(&mut self) {
        Self::inc_value(&mut self.inner.n_droplets, &mut self.take);
    }

    /// Should only be used if [`take_more`] returns `true`.
    ///
    /// [`take_more`]: Self::take_more
    pub fn inc_tiny_droplets(&mut self) {
        self.inner.n_tiny_droplets += 1;
    }

    fn inc_value(value: &mut u32, take: &mut usize) {
        if *take > 0 {
            *value += 1;
            *take -= 1;
        }
    }
}
