use crate::mania::performance::ManiaPerformance;

/// The result of a difficulty calculation on an osu!mania map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ManiaDifficultyAttributes {
    /// The final star rating.
    pub stars: f64,
    /// The amount of hitobjects in the map.
    pub n_objects: u32,
    /// The amount of hold notes in the map.
    pub n_hold_notes: u32,
    /// The maximum achievable combo.
    pub max_combo: u32,
    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub is_convert: bool,
}

impl ManiaDifficultyAttributes {
    /// Return the maximum combo.
    pub const fn max_combo(&self) -> u32 {
        self.max_combo
    }

    /// Return the amount of hitobjects.
    pub const fn n_objects(&self) -> u32 {
        self.n_objects
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> ManiaPerformance<'a> {
        self.into()
    }
}

/// The result of a performance calculation on an osu!mania map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ManiaPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation.
    pub difficulty: ManiaDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
    /// The difficulty portion of the final pp.
    pub pp_difficulty: f64,
}

impl ManiaPerformanceAttributes {
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
        self.difficulty.max_combo
    }

    /// Return the amount of hitobjects.
    pub const fn n_objects(&self) -> u32 {
        self.difficulty.n_objects
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.difficulty.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> ManiaPerformance<'a> {
        self.difficulty.into()
    }
}

impl From<ManiaPerformanceAttributes> for ManiaDifficultyAttributes {
    fn from(attributes: ManiaPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
