#![cfg(feature = "fruits")]

mod catch_object;
mod difficulty_object;
mod fruit_or_juice;
mod gradual_difficulty;
mod movement;
mod pp;
mod slider_state;

use catch_object::CatchObject;
use difficulty_object::DifficultyObject;
use fruit_or_juice::FruitOrJuice;
pub use gradual_difficulty::*;
use movement::Movement;
pub use pp::*;
use slider_state::SliderState;

use crate::{curve::CurveBuffers, fruits::fruit_or_juice::FruitParams, Beatmap, Mods, Strains};

const SECTION_LENGTH: f64 = 750.0;
const STAR_SCALING_FACTOR: f64 = 0.153;

const ALLOWED_CATCH_RANGE: f32 = 0.8;
const CATCHER_SIZE: f32 = 106.75;

/// Difficulty calculation for osu!ctb maps.
///
/// In case of a partial play, e.g. a fail, one can specify the amount of passed objects.
pub fn stars(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> FruitsDifficultyAttributes {
    let (mut movement, mut attributes) = calculate_movement(map, mods, passed_objects);
    attributes.stars =
        Movement::difficulty_value(&mut movement.strain_peaks).sqrt() * STAR_SCALING_FACTOR;

    attributes
}

/// Essentially the same as the [`stars`] function but instead of
/// evaluating the final strains, it just returns them as is.
///
/// Suitable to plot the difficulty of a map over time.
pub fn strains(map: &Beatmap, mods: impl Mods) -> Strains {
    let (movement, _) = calculate_movement(map, mods, None);

    Strains {
        section_length: SECTION_LENGTH * mods.speed(),
        strains: movement.strain_peaks,
    }
}

fn calculate_movement(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> (Movement, FruitsDifficultyAttributes) {
    let take = passed_objects.unwrap_or(usize::MAX);

    let map_attributes = map.attributes().mods(mods);

    let attributes = FruitsDifficultyAttributes {
        ar: map_attributes.ar,
        ..Default::default()
    };

    let mut params = FruitParams {
        attributes,
        curve_bufs: CurveBuffers::default(),
        last_pos: None,
        last_time: 0.0,
        map,
        slider_state: SliderState::new(map),
        ticks: Vec::new(), // using the same buffer for all sliders
        with_hr: mods.hr(),
    };

    // BUG: Incorrect object order on 2B maps that have fruits within sliders
    let mut hit_objects = map
        .hit_objects
        .iter()
        .filter_map(|h| FruitOrJuice::new(h, &mut params))
        .flatten()
        .take(take);

    // Hyper dash business
    let half_catcher_width =
        (calculate_catch_width(map_attributes.cs as f32) / 2.0 / ALLOWED_CATCH_RANGE) as f64;
    let mut last_direction = 0;
    let mut last_excess = half_catcher_width;

    // Strain business
    let mut movement = Movement::new(map_attributes.cs as f32);

    let (mut prev, curr) = match (hit_objects.next(), hit_objects.next()) {
        (Some(prev), Some(curr)) => (prev, curr),
        (Some(_), None) | (None, None) => return (movement, params.attributes),
        (None, Some(_)) => unreachable!(),
    };

    let mut curr_section_end =
        (curr.time / map_attributes.clock_rate / SECTION_LENGTH).ceil() * SECTION_LENGTH;

    prev.init_hyper_dash(
        half_catcher_width,
        &curr,
        &mut last_direction,
        &mut last_excess,
    );

    // Handle first object distinctly
    let h = DifficultyObject::new(
        &curr,
        &prev,
        movement.half_catcher_width,
        map_attributes.clock_rate,
    );

    movement.process(&h);
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        prev.init_hyper_dash(
            half_catcher_width,
            &curr,
            &mut last_direction,
            &mut last_excess,
        );

        let h = DifficultyObject::new(
            &curr,
            &prev,
            movement.half_catcher_width,
            map_attributes.clock_rate,
        );

        let base_time = h.base.time / map_attributes.clock_rate;

        while base_time > curr_section_end {
            movement.save_current_peak();
            movement.start_new_section_from(curr_section_end);
            curr_section_end += SECTION_LENGTH;
        }

        movement.process(&h);
        prev = curr;
    }

    movement.save_current_peak();

    (movement, params.attributes)
}

#[inline]
pub(crate) fn calculate_catch_width(cs: f32) -> f32 {
    let scale = 1.0 - 0.7 * (cs - 5.0) / 5.0;

    CATCHER_SIZE * scale.abs() * ALLOWED_CATCH_RANGE
}

/// The result of a difficulty calculation on an osu!ctb map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FruitsDifficultyAttributes {
    /// The final star rating
    pub stars: f64,
    /// The approach rate.
    pub ar: f64,
    /// The amount of fruits.
    pub n_fruits: usize,
    /// The amount of droplets.
    pub n_droplets: usize,
    /// The amount of tiny droplets.
    pub n_tiny_droplets: usize,
}

impl FruitsDifficultyAttributes {
    /// Return the maximum combo.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.n_fruits + self.n_droplets
    }
}

/// The result of a performance calculation on an osu!ctb map.
#[derive(Clone, Debug, Default)]
pub struct FruitsPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: FruitsDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
}

impl FruitsPerformanceAttributes {
    /// Return the star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        self.difficulty.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.difficulty.max_combo()
    }
}

impl From<FruitsPerformanceAttributes> for FruitsDifficultyAttributes {
    fn from(attributes: FruitsPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
