mod difficulty_object;
mod gradual_difficulty;
mod gradual_performance;
mod mania_object;
mod pp;
mod skills;

use std::borrow::Cow;

use crate::{beatmap::BeatmapHitWindows, util::FloatExt, Beatmap, GameMode, Mods, OsuStars};

pub use self::{gradual_difficulty::*, gradual_performance::*, pp::*};

pub(crate) use self::mania_object::ManiaObject;

use self::{
    difficulty_object::ManiaDifficultyObject,
    mania_object::ObjectParameters,
    skills::{Skill, Strain},
};

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
    map: Cow<'map, Beatmap>,
    mods: u32,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,
    is_convert: bool,
}

impl<'map> ManiaStars<'map> {
    /// Create a new difficulty calculator for osu!mania maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        let map = map.convert_mode(GameMode::Mania);
        let is_convert = matches!(map, Cow::Owned(_));

        Self {
            map,
            mods: 0,
            passed_objects: None,
            clock_rate: None,
            is_convert,
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

    /// Specify whether the map is a convert i.e. an osu!standard map.
    #[inline]
    pub fn is_convert(mut self, is_convert: bool) -> Self {
        self.is_convert = is_convert;

        self
    }

    /// Calculate all difficulty related values, including stars.
    #[inline]
    pub fn calculate(self) -> ManiaDifficultyAttributes {
        let is_convert = self.is_convert || matches!(self.map, Cow::Owned(_));

        let clock_rate = self.clock_rate.unwrap_or_else(|| self.mods.clock_rate());

        let BeatmapHitWindows { od: hit_window, .. } = self
            .map
            .attributes()
            .mods(self.mods)
            .converted(is_convert)
            .clock_rate(clock_rate)
            .hit_windows();

        let ManiaResult { strain, max_combo } = calculate_result(self);

        ManiaDifficultyAttributes {
            stars: strain.difficulty_value() * STAR_SCALING_FACTOR,
            hit_window,
            max_combo,
        }
    }

    /// Calculate the skill strains.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> ManiaStrains {
        let ManiaResult { strain, .. } = calculate_result(self);

        ManiaStrains {
            section_len: SECTION_LEN,
            strains: strain.strain_peaks,
        }
    }
}

/// The result of calculating the strains on a osu!taiko map.
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug)]
pub struct ManiaStrains {
    /// Time in ms inbetween two strains.
    pub section_len: f64,
    /// Strain peaks of the strain skill.
    pub strains: Vec<f64>,
}

impl ManiaStrains {
    /// Returns the number of strain peaks per skill.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.strains.len()
    }
}

fn calculate_result(params: ManiaStars<'_>) -> ManiaResult {
    let ManiaStars {
        map,
        mods,
        passed_objects,
        clock_rate,
        is_convert: _,
    } = params;

    let take = passed_objects.unwrap_or(map.hit_objects.len());
    let total_columns = map.cs.round_even().max(1.0);

    let clock_rate = clock_rate.unwrap_or_else(|| mods.clock_rate());
    let mut strain = Strain::new(total_columns as usize);
    let mut params = ObjectParameters::new(map.as_ref());
    let mut hit_objects = map.hit_objects.iter().take(take);

    let first = match hit_objects.next() {
        Some(h) => ManiaObject::new(h, total_columns, &mut params),
        None => {
            return ManiaResult {
                strain,
                max_combo: 0,
            }
        }
    };

    let diff_objects_iter = hit_objects.enumerate().scan(first, |last, (i, h)| {
        let base = ManiaObject::new(h, total_columns, &mut params);
        let diff_object = ManiaDifficultyObject::new(&base, &*last, clock_rate, i);
        *last = base;

        Some(diff_object)
    });

    let mut diff_objects = Vec::with_capacity(map.hit_objects.len().min(take).saturating_sub(1));
    diff_objects.extend(diff_objects_iter);

    for curr in diff_objects.iter() {
        strain.process(curr, &diff_objects);
    }

    ManiaResult {
        strain,
        max_combo: params.max_combo,
    }
}

struct ManiaResult {
    strain: Strain,
    max_combo: usize,
}

/// The result of a difficulty calculation on an osu!mania map.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ManiaDifficultyAttributes {
    /// The final star rating.
    pub stars: f64,
    /// The perceived hit window for an n300 inclusive of rate-adjusting mods (DT/HT/etc).
    pub hit_window: f64,
    /// The maximum achievable combo.
    pub max_combo: usize,
}

impl ManiaDifficultyAttributes {
    /// Return the maximum combo.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.max_combo
    }
}

/// The result of a performance calculation on an osu!mania map.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
        self.difficulty.max_combo
    }
}

impl From<ManiaPerformanceAttributes> for ManiaDifficultyAttributes {
    #[inline]
    fn from(attributes: ManiaPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}

impl<'map> From<OsuStars<'map>> for ManiaStars<'map> {
    #[inline]
    fn from(osu: OsuStars<'map>) -> Self {
        let OsuStars {
            map,
            mods,
            passed_objects,
            clock_rate,
        } = osu;

        Self {
            map: map.convert_mode(GameMode::Mania),
            mods,
            passed_objects,
            clock_rate,
            is_convert: true,
        }
    }
}
