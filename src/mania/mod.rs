mod gradual_difficulty;
mod gradual_performance;
mod pp;
mod strain;

pub use gradual_difficulty::*;
pub use gradual_performance::*;
pub use pp::*;
use strain::Strain;

use crate::{parse::HitObject, Beatmap, GameMode, Mods, Strains};

const SECTION_LEN: f64 = 400.0;
const STAR_SCALING_FACTOR: f64 = 0.018;

/// Difficulty calculator on osu!mania maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{ManiaStars, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let difficulty_attrs = ManiaStars::new(&map)
///     .mods(8 + 64) // HDDT
///     .calculate();
///
/// println!("Stars: {}", difficulty_attrs.stars);
/// ```
#[derive(Clone, Debug)]
pub struct ManiaStars<'map> {
    map: &'map Beatmap,
    mods: u32,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,
}

impl<'map> ManiaStars<'map> {
    /// Create a new difficulty calculator for osu!mania maps.
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
    /// using [`ManiaStars`] multiple times with different `passed_objects`, you should use
    /// [`ManiaGradualDifficultyAttributes`](crate::mania::ManiaGradualDifficultyAttributes).
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
    pub fn calculate(self) -> ManiaDifficultyAttributes {
        let mut strain = calculate_strain(self);

        ManiaDifficultyAttributes {
            stars: Strain::difficulty_value(&mut strain.strain_peaks) * STAR_SCALING_FACTOR,
        }
    }

    /// Calculate the skill strains.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> Strains {
        let clock_rate = self.clock_rate.unwrap_or_else(|| self.mods.speed());
        let strain = calculate_strain(self);

        Strains {
            section_length: SECTION_LEN * clock_rate,
            strains: strain.strain_peaks,
        }
    }
}

fn calculate_strain(params: ManiaStars<'_>) -> Strain {
    let ManiaStars {
        map,
        mods,
        passed_objects,
        clock_rate,
    } = params;

    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());
    let rounded_cs = map.cs.round();

    let columns = match map.mode {
        GameMode::MNA => rounded_cs.max(1.0) as u8,
        GameMode::STD => {
            let rounded_od = map.od.round();

            let n_objects = map.n_circles + map.n_sliders + map.n_spinners;
            let slider_or_spinner_ratio = (n_objects - map.n_circles) as f32 / n_objects as f32;

            if slider_or_spinner_ratio < 0.2 {
                7
            } else if slider_or_spinner_ratio < 0.3 || rounded_cs >= 5.0 {
                6 + (rounded_od > 5.0) as u8
            } else if slider_or_spinner_ratio > 0.6 {
                4 + (rounded_od > 4.0) as u8
            } else {
                (rounded_od as u8 + 1).max(4).min(7)
            }
        }
        other => panic!("can not calculate mania difficulty on a {:?} map", other),
    };

    let clock_rate = clock_rate.unwrap_or_else(|| mods.speed());
    let mut strain = Strain::new(columns);
    let columns = columns as f32;

    let mut hit_objects = map
        .hit_objects
        .iter()
        .take(take)
        .skip(1)
        .zip(map.hit_objects.iter())
        .map(|(base, prev)| DifficultyHitObject::new(base, prev, columns, clock_rate));

    // Handle first object distinctly
    let h = match hit_objects.next() {
        Some(h) => h,
        None => return strain,
    };

    // No strain for first object
    let mut curr_section_end = (h.start_time / SECTION_LEN).ceil() * SECTION_LEN;
    strain.process(&h);

    // Handle all other objects
    for h in hit_objects {
        while h.start_time > curr_section_end {
            strain.save_current_peak();
            strain.start_new_section_from(curr_section_end);
            curr_section_end += SECTION_LEN;
        }

        strain.process(&h);
    }

    strain.save_current_peak();

    strain
}

#[derive(Debug)]
pub(crate) struct DifficultyHitObject<'o> {
    base: &'o HitObject,
    column: usize,
    delta: f64,
    start_time: f64,
}

impl<'o> DifficultyHitObject<'o> {
    #[inline]
    fn new(base: &'o HitObject, prev: &'o HitObject, columns: f32, clock_rate: f64) -> Self {
        let x_divisor = 512.0 / columns;
        let column = (base.pos.x / x_divisor).floor().min(columns - 1.0) as usize;

        Self {
            base,
            column,
            delta: (base.start_time - prev.start_time) / clock_rate,
            start_time: base.start_time / clock_rate,
        }
    }
}

/// The result of a difficulty calculation on an osu!mania map.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ManiaDifficultyAttributes {
    /// The final star rating.
    pub stars: f64,
}

/// The result of a performance calculation on an osu!mania map.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ManiaPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: ManiaDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
    /// The accuracy portion of the final pp.
    pub pp_acc: f64,
    /// The strain portion of the final pp.
    pub pp_strain: f64,
}

impl ManiaPerformanceAttributes {
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
}

impl From<ManiaPerformanceAttributes> for ManiaDifficultyAttributes {
    fn from(attributes: ManiaPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}
