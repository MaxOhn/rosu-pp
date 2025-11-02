use crate::{model::beatmap::BeatmapAttributesBuilder, osu::performance::OsuPerformance};

/// The result of a difficulty calculation on an osu!standard map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsuDifficultyAttributes {
    /// The difficulty of the aim skill.
    pub aim: f64,
    /// The number of sliders weighted by difficulty.
    pub aim_difficult_slider_count: f64,
    /// The difficulty of the speed skill.
    pub speed: f64,
    /// The difficulty of the flashlight skill.
    pub flashlight: f64,
    /// The ratio of the aim strain with and without considering sliders
    pub slider_factor: f64,
    /// Describes how much of aim's difficult strain count is contributed to by sliders
    pub aim_top_weighted_slider_factor: f64,
    /// Describes how much of speed's difficult strain count is contributed to by sliders
    pub speed_top_weighted_slider_factor: f64,
    /// The number of clickable objects weighted by difficulty.
    pub speed_note_count: f64,
    /// Weighted sum of aim strains.
    pub aim_difficult_strain_count: f64,
    /// Weighted sum of speed strains.
    pub speed_difficult_strain_count: f64,
    /// The amount of nested score per object.
    pub nested_score_per_object: f64,
    /// The legacy score base multiplier.
    pub legacy_score_base_multiplier: f64,
    /// The maximum legacy combo score.
    pub maximum_legacy_combo_score: f64,
    /// The approach rate.
    pub ar: f64,
    /// The great hit window.
    pub great_hit_window: f64,
    /// The ok hit window.
    pub ok_hit_window: f64,
    /// The meh hit window.
    pub meh_hit_window: f64,
    /// The health drain rate.
    pub hp: f64,
    /// The amount of circles.
    pub n_circles: u32,
    /// The amount of sliders.
    pub n_sliders: u32,
    /// The amount of "large ticks".
    ///
    /// The meaning depends on the kind of score:
    /// - if set on osu!stable, this value is irrelevant
    /// - if set on osu!lazer *with* slider accuracy, this value is the amount
    ///   of hit slider ticks and repeats
    /// - if set on osu!lazer *without* slider accuracy, this value is the
    ///   amount of hit slider heads, ticks, and repeats
    pub n_large_ticks: u32,
    /// The amount of spinners.
    pub n_spinners: u32,
    /// The final star rating
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: u32,
}

impl OsuDifficultyAttributes {
    /// Return the maximum combo.
    pub const fn max_combo(&self) -> u32 {
        self.max_combo
    }

    /// Return the amount of hitobjects.
    pub const fn n_objects(&self) -> u32 {
        self.n_circles + self.n_sliders + self.n_spinners
    }

    /// The overall difficulty
    pub const fn od(&self) -> f64 {
        BeatmapAttributesBuilder::osu_great_hit_window_to_od(self.great_hit_window)
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> OsuPerformance<'a> {
        self.into()
    }
}

/// The result of a performance calculation on an osu!standard map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsuPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: OsuDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
    /// The accuracy portion of the final pp.
    pub pp_acc: f64,
    /// The aim portion of the final pp.
    pub pp_aim: f64,
    /// The flashlight portion of the final pp.
    pub pp_flashlight: f64,
    /// The speed portion of the final pp.
    pub pp_speed: f64,
    /// Misses including an approximated amount of slider breaks
    pub effective_miss_count: f64,
    /// Approximated unstable-rate
    pub speed_deviation: Option<f64>,
}

impl OsuPerformanceAttributes {
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
        self.difficulty.n_objects()
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> OsuPerformance<'a> {
        self.difficulty.into()
    }
}

impl From<OsuPerformanceAttributes> for OsuDifficultyAttributes {
    fn from(attributes: OsuPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
