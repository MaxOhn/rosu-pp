#![cfg(feature = "mania")]

mod pp;
mod strain;

pub use pp::*;
use strain::Strain;

use crate::{parse::HitObject, Beatmap, GameMode, Mods, Strains};

const SECTION_LEN: f64 = 400.0;
const STAR_SCALING_FACTOR: f64 = 0.018;

/// Star calculation for osu!mania maps
///
/// In case of a partial play, e.g. a fail, one can specify the amount of passed objects.
pub fn stars(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> DifficultyAttributes {
    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());

    if take < 2 {
        return DifficultyAttributes::default();
    }

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

    let clock_rate = mods.speed();
    let section_len = SECTION_LEN * clock_rate;
    let mut strain = Strain::new(columns);
    let columns = columns as f32;

    let mut hit_objects = map
        .hit_objects
        .iter()
        .take(take)
        .skip(1)
        .zip(map.hit_objects.iter())
        .map(|(base, prev)| DifficultyHitObject::new(base, prev, columns, clock_rate));

    // No strain for first object
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    // Handle second object separately to remove later if-branching
    let h = hit_objects.next().unwrap();

    while h.base.start_time > current_section_end {
        current_section_end += section_len;
    }

    strain.process(&h);

    // Handle all other objects
    for h in hit_objects {
        while h.base.start_time > current_section_end {
            strain.save_current_peak();
            strain.start_new_section_from(current_section_end / clock_rate);

            current_section_end += section_len;
        }

        strain.process(&h);
    }

    strain.save_current_peak();

    let stars = strain.difficulty_value() * STAR_SCALING_FACTOR;

    DifficultyAttributes { stars }
}

/// Essentially the same as the `stars` function but instead of
/// evaluating the final strains, it just returns them as is.
///
/// Suitable to plot the difficulty of a map over time.
pub fn strains(map: &Beatmap, mods: impl Mods) -> Strains {
    if map.hit_objects.len() < 2 {
        return Strains::default();
    }

    let clock_rate = mods.speed();
    let section_len = SECTION_LEN * clock_rate;
    let mut strain = Strain::new(map.cs as u8);

    let mut hit_objects = map
        .hit_objects
        .iter()
        .skip(1)
        .zip(map.hit_objects.iter())
        .map(|(base, prev)| DifficultyHitObject::new(base, prev, map.cs, clock_rate));

    // No strain for first object
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    // Handle second object separately to remove later if-branching
    let h = hit_objects.next().unwrap();

    while h.base.start_time > current_section_end {
        current_section_end += section_len;
    }

    strain.process(&h);

    // Handle all other objects
    for h in hit_objects {
        while h.base.start_time > current_section_end {
            strain.save_current_peak();
            strain.start_new_section_from(current_section_end / clock_rate);

            current_section_end += section_len;
        }

        strain.process(&h);
    }

    strain.save_current_peak();

    Strains {
        section_length: section_len,
        strains: strain.strain_peaks,
    }
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

/// Various data created through the star calculation.
/// This data is necessary to calculate PP.
#[derive(Copy, Clone, Debug, Default)]
pub struct DifficultyAttributes {
    pub stars: f64,
}

/// Various data created through the pp calculation.
#[derive(Copy, Clone, Debug, Default)]
pub struct PerformanceAttributes {
    pub attributes: DifficultyAttributes,
    pub pp_acc: f64,
    pub pp_strain: f64,
    pub pp: f64,
}

impl PerformanceAttributes {
    /// Return the star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        self.attributes.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f64 {
        self.pp
    }
}
