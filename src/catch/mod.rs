mod catch_object;
mod difficulty_object;
mod fruit_or_juice;
mod gradual_difficulty;
mod gradual_performance;
mod movement;
mod pp;

use catch_object::CatchObject;
use difficulty_object::DifficultyObject;
use fruit_or_juice::FruitOrJuice;
pub use gradual_difficulty::*;
pub use gradual_performance::*;
use movement::Movement;
pub use pp::*;

use crate::{catch::fruit_or_juice::FruitParams, curve::CurveBuffers, Beatmap, Mods, OsuStars};

const SECTION_LENGTH: f64 = 750.0;
const STAR_SCALING_FACTOR: f64 = 0.153;

const ALLOWED_CATCH_RANGE: f32 = 0.8;
const CATCHER_SIZE: f32 = 106.75;

/// Difficulty calculator on osu!catch maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{CatchStars, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let difficulty_attrs = CatchStars::new(&map)
///     .mods(8 + 64) // HDDT
///     .calculate();
///
/// println!("Stars: {}", difficulty_attrs.stars);
/// ```
#[derive(Clone, Debug)]
pub struct CatchStars<'map> {
    map: &'map Beatmap,
    mods: u32,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,
}

impl<'map> CatchStars<'map> {
    /// Create a new difficulty calculator for osu!catch maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map,
            mods: 0,
            passed_objects: None,
            clock_rate: None,
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(mut self, mods: u32) -> Self {
        self.mods = mods;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the difficulty after every few objects, instead of
    /// using [`CatchStars`] multiple times with different `passed_objects`, you should use
    /// [`CatchGradualDifficultyAttributes`](crate::catch::CatchGradualDifficultyAttributes).
    #[inline]
    pub fn passed_objects(mut self, passed_objects: usize) -> Self {
        self.passed_objects = Some(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    #[inline]
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.clock_rate = Some(clock_rate);

        self
    }

    /// Calculate all difficulty related values, including stars.
    #[inline]
    pub fn calculate(self) -> CatchDifficultyAttributes {
        let (mut movement, mut attributes) = calculate_movement(self);
        attributes.stars =
            Movement::difficulty_value(&mut movement.strain_peaks).sqrt() * STAR_SCALING_FACTOR;

        attributes
    }

    /// Calculate the skill strains.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> CatchStrains {
        let (movement, _) = calculate_movement(self);

        CatchStrains {
            section_len: SECTION_LENGTH,
            movement: movement.strain_peaks,
        }
    }
}

/// The result of calculating the strains on a osu!catch map.
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug)]
pub struct CatchStrains {
    /// Time in ms inbetween two strains.
    pub section_len: f64,
    /// Strain peaks of the movement skill.
    pub movement: Vec<f64>,
}

impl CatchStrains {
    /// Returns the number of strain peaks per skill.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.movement.len()
    }
}

fn calculate_movement(params: CatchStars<'_>) -> (Movement, CatchDifficultyAttributes) {
    let CatchStars {
        map,
        mods,
        passed_objects,
        clock_rate,
    } = params;

    let take = passed_objects.unwrap_or(usize::MAX);
    let clock_rate = clock_rate.unwrap_or_else(|| mods.clock_rate());
    let map_attributes = map.attributes().mods(mods).clock_rate(clock_rate).build();

    let attributes = CatchDifficultyAttributes {
        ar: map_attributes.ar,
        ..Default::default()
    };

    let mut params = FruitParams {
        attributes,
        curve_bufs: CurveBuffers::default(),
        last_pos: None,
        last_time: 0.0,
        map,
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

    let mut curr_section_end = (curr.time / clock_rate / SECTION_LENGTH).ceil() * SECTION_LENGTH;

    prev.init_hyper_dash(
        half_catcher_width,
        &curr,
        &mut last_direction,
        &mut last_excess,
    );

    // Handle first object distinctly
    let h = DifficultyObject::new(&curr, &prev, movement.half_catcher_width, clock_rate);

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

        let h = DifficultyObject::new(&curr, &prev, movement.half_catcher_width, clock_rate);

        let base_time = h.base.time / clock_rate;

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

/// The result of a difficulty calculation on an osu!catch map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatchDifficultyAttributes {
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

impl CatchDifficultyAttributes {
    /// Return the maximum combo.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.n_fruits + self.n_droplets
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

impl From<CatchPerformanceAttributes> for CatchDifficultyAttributes {
    #[inline]
    fn from(attributes: CatchPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}

impl<'map> From<OsuStars<'map>> for CatchStars<'map> {
    #[inline]
    fn from(osu: OsuStars<'map>) -> Self {
        let OsuStars {
            map,
            mods,
            passed_objects,
            clock_rate,
        } = osu;

        Self {
            map,
            mods,
            passed_objects,
            clock_rate,
        }
    }
}
