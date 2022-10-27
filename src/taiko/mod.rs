mod colours;
mod difficulty_object;
mod gradual_difficulty;
mod gradual_performance;
mod pp;
mod rim;
mod skills;
mod taiko_object;

use std::{borrow::Cow, cell::RefCell, rc::Rc};

pub use self::{gradual_difficulty::*, gradual_performance::*, pp::*};

use crate::{beatmap::BeatmapHitWindows, Beatmap, GameMode, Mods, OsuStars};

use self::{
    colours::ColourDifficultyPreprocessor,
    difficulty_object::{MonoIndex, ObjectLists, TaikoDifficultyObject},
    skills::{Peaks, PeaksDifficultyValues, PeaksRaw, Skill},
    taiko_object::IntoTaikoObjectIter,
};

const SECTION_LEN: usize = 400;

const DIFFICULTY_MULTIPLIER: f64 = 1.35;

/// Difficulty calculator on osu!taiko maps.
///
/// # Example
///
/// ```
/// use rosu_pp::{TaikoStars, Beatmap};
///
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
///
/// let difficulty_attrs = TaikoStars::new(&map)
///     .mods(8 + 64) // HDDT
///     .calculate();
///
/// println!("Stars: {}", difficulty_attrs.stars);
/// ```
#[derive(Clone, Debug)]
pub struct TaikoStars<'map> {
    map: Cow<'map, Beatmap>,
    mods: u32,
    passed_objects: Option<usize>,
    clock_rate: Option<f64>,
    is_convert: bool,
}

impl<'map> TaikoStars<'map> {
    /// Create a new difficulty calculator for osu!taiko maps.
    #[inline]
    pub fn new(map: &'map Beatmap) -> Self {
        let map = map.convert_mode(GameMode::Taiko);
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
    /// using [`TaikoStars`] multiple times with different `passed_objects`, you should use
    /// [`TaikoGradualDifficultyAttributes`](crate::taiko::TaikoGradualDifficultyAttributes).
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
    pub fn calculate(self) -> TaikoDifficultyAttributes {
        let clock_rate = self.clock_rate.unwrap_or_else(|| self.mods.clock_rate());

        let BeatmapHitWindows { od: hit_window, .. } = self
            .map
            .attributes()
            .mods(self.mods)
            .clock_rate(clock_rate)
            .hit_windows();

        let is_convert = self.is_convert || matches!(self.map, Cow::Owned(_));

        let (peaks, max_combo) = calculate_skills(self);

        let PeaksDifficultyValues {
            mut colour_rating,
            mut rhythm_rating,
            mut stamina_rating,
            mut combined_rating,
        } = peaks.difficulty_values();

        colour_rating *= DIFFICULTY_MULTIPLIER;
        rhythm_rating *= DIFFICULTY_MULTIPLIER;
        stamina_rating *= DIFFICULTY_MULTIPLIER;
        combined_rating *= DIFFICULTY_MULTIPLIER;

        let mut star_rating = rescale(combined_rating * 1.4);

        // * TODO: This is temporary measure as we don't detect abuse of multiple-input
        // * playstyles of converts within the current system.
        if is_convert {
            star_rating *= 0.925;

            // * For maps with low colour variance and high stamina requirement,
            // * multiple inputs are more likely to be abused.
            if colour_rating < 2.0 && stamina_rating > 8.0 {
                star_rating *= 0.8;
            }
        }

        TaikoDifficultyAttributes {
            stamina: stamina_rating,
            rhythm: rhythm_rating,
            colour: colour_rating,
            peak: combined_rating,
            hit_window,
            stars: star_rating,
            max_combo,
        }
    }

    /// Calculate the skill strains.
    ///
    /// Suitable to plot the difficulty of a map over time.
    #[inline]
    pub fn strains(self) -> TaikoStrains {
        let (peaks, _) = calculate_skills(self);

        let PeaksRaw {
            colour,
            rhythm,
            stamina,
        } = peaks.into_raw();

        TaikoStrains {
            section_len: SECTION_LEN as f64,
            color: colour,
            rhythm,
            stamina,
        }
    }
}

/// The result of calculating the strains on a osu!taiko map.
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug)]
pub struct TaikoStrains {
    /// Time in ms inbetween two strains.
    pub section_len: f64,
    /// Strain peaks of the color skill.
    pub color: Vec<f64>,
    /// Strain peaks of the rhythm skill.
    pub rhythm: Vec<f64>,
    /// Strain peaks of the stamina skill.
    pub stamina: Vec<f64>,
}

impl TaikoStrains {
    /// Returns the number of strain peaks per skill.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.color.len()
    }
}

fn calculate_skills(params: TaikoStars<'_>) -> (Peaks, usize) {
    let TaikoStars {
        map,
        mods,
        passed_objects,
        clock_rate,
        is_convert: _,
    } = params;

    let mut take = passed_objects.unwrap_or(map.hit_objects.len());
    let clock_rate = clock_rate.unwrap_or_else(|| mods.clock_rate());

    let mut peaks = Peaks::new();
    let mut max_combo = 0;

    let mut diff_objects = map
        .taiko_objects()
        .take_while(|(h, _)| {
            if h.is_hit {
                if take == 0 {
                    return false;
                }

                max_combo += 1;
                take -= 1;
            }

            true
        })
        .skip(2)
        .zip(map.hit_objects.iter().skip(1))
        .zip(map.hit_objects.iter())
        .enumerate()
        .fold(
            ObjectLists::default(),
            |mut lists, (idx, (((base, base_start_time), last), last_last))| {
                let diff_obj = TaikoDifficultyObject::new(
                    base,
                    base_start_time,
                    last.start_time,
                    last_last.start_time,
                    clock_rate,
                    &lists,
                    idx,
                );

                match &diff_obj.mono_idx {
                    MonoIndex::Centre(_) => lists.centres.push(idx),
                    MonoIndex::Rim(_) => lists.rims.push(idx),
                    MonoIndex::None => {}
                }

                if diff_obj.note_idx.is_some() {
                    lists.notes.push(idx);
                }

                lists.all.push(Rc::new(RefCell::new(diff_obj)));

                lists
            },
        );

    ColourDifficultyPreprocessor::process_and_assign(&mut diff_objects);

    for hit_object in diff_objects.all.iter() {
        peaks.process(&hit_object.borrow(), &diff_objects);
    }

    (peaks, max_combo)
}

#[inline]
fn rescale(stars: f64) -> f64 {
    if stars < 0.0 {
        stars
    } else {
        10.43 * (stars / 8.0 + 1.0).ln()
    }
}

/// The result of a difficulty calculation on an osu!taiko map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaikoDifficultyAttributes {
    /// The difficulty corresponding to the stamina skill.
    pub stamina: f64,
    /// The difficulty corresponding to the rhythm skill.
    pub rhythm: f64,
    /// The difficulty corresponding to the colour skill.
    pub colour: f64,
    /// The difficulty corresponding to the hardest parts of the map.
    pub peak: f64,
    /// The perceived hit window for an n300 inclusive of rate-adjusting mods (DT/HT/etc)
    pub hit_window: f64,
    /// The final star rating.
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: usize,
}

impl TaikoDifficultyAttributes {
    /// Return the maximum combo.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.max_combo
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
    /// Scaled miss count based on total hits.
    pub effective_miss_count: f64,
}

impl TaikoPerformanceAttributes {
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

impl From<TaikoPerformanceAttributes> for TaikoDifficultyAttributes {
    #[inline]
    fn from(attributes: TaikoPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}

impl<'map> From<OsuStars<'map>> for TaikoStars<'map> {
    #[inline]
    fn from(osu: OsuStars<'map>) -> Self {
        let OsuStars {
            map,
            mods,
            passed_objects,
            clock_rate,
        } = osu;

        Self {
            map: map.convert_mode(GameMode::Taiko),
            mods,
            passed_objects,
            clock_rate,
            is_convert: true,
        }
    }
}
