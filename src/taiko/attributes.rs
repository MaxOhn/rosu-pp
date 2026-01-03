use crate::taiko::performance::TaikoPerformance;

/// The result of a difficulty calculation on an osu!taiko map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaikoDifficultyAttributes {
    /// The difficulty of the stamina skill.
    pub stamina: f64,
    /// The difficulty of the rhythm skill.
    pub rhythm: f64,
    /// The difficulty of the color skill.
    pub color: f64,
    /// The difficulty of the reading skill.
    pub reading: f64,
    /// The perceived hit window for an n300 inclusive of rate-adjusting mods (DT/HT/etc)
    pub great_hit_window: f64,
    /// The perceived hit window for an n100 inclusive of rate-adjusting mods (DT/HT/etc)
    pub ok_hit_window: f64,
    /// The ratio of stamina difficulty from mono-color (single color) streams to total
    /// stamina difficulty.
    pub mono_stamina_factor: f64,
    /// The difficulty corresponding to the mechanical skills.
    ///
    /// This includes colour and stamina combined.
    pub mechanical_difficulty: f64,
    /// The factor corresponding to the consistency of a map.
    pub consistency_factor: f64,
    /// The final star rating.
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: u32,
    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub is_convert: bool,
}

impl TaikoDifficultyAttributes {
    /// Return the maximum combo.
    pub const fn max_combo(&self) -> u32 {
        self.max_combo
    }

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> TaikoPerformance<'a> {
        self.into()
    }
}

/// The result of a performance calculation on an osu!taiko map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaikoPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: TaikoDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
    /// The accuracy portion of the final pp.
    pub pp_acc: f64,
    /// The strain portion of the final pp.
    pub pp_difficulty: f64,
    /// Upper bound on the player's tap deviation.
    pub estimated_unstable_rate: Option<f64>,
}

impl TaikoPerformanceAttributes {
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

    /// Whether the [`Beatmap`] was a convert i.e. an osu!standard map.
    ///
    /// [`Beatmap`]: crate::model::beatmap::Beatmap
    pub const fn is_convert(&self) -> bool {
        self.difficulty.is_convert
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> TaikoPerformance<'a> {
        self.difficulty.into()
    }
}

impl From<TaikoPerformanceAttributes> for TaikoDifficultyAttributes {
    fn from(attributes: TaikoPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
