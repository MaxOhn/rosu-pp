use std::cmp;

use rosu_map::section::general::GameMode;

use crate::{
    Beatmap,
    any::difficulty::{Difficulty, skills::StrainSkill},
    mania::{
        difficulty::{object::ManiaDifficultyObject, skills::strain::Strain},
        object::{ManiaObject, ObjectParams},
    },
    model::mode::ConvertError,
};

use super::{attributes::ManiaDifficultyAttributes, convert};

pub mod gradual;
mod object;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.018;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<ManiaDifficultyAttributes, ConvertError> {
    let mut map = map.convert_ref(GameMode::Mania, difficulty.get_mods())?;

    if difficulty.get_mods().ho() {
        convert::apply_hold_off_to_beatmap(map.to_mut());
    }

    if difficulty.get_mods().invert() {
        convert::apply_invert_to_beatmap(map.to_mut());
    }

    if let Some(seed) = difficulty.get_mods().random_seed() {
        convert::apply_random_to_beatmap(map.to_mut(), seed);
    }

    let n_objects = cmp::min(difficulty.get_passed_objects(), map.hit_objects.len()) as u32;

    let values = DifficultyValues::calculate(difficulty, &map);

    Ok(ManiaDifficultyAttributes {
        stars: values.strain.into_difficulty_value() * DIFFICULTY_MULTIPLIER,
        max_combo: values.max_combo,
        n_objects,
        n_hold_notes: values.n_hold_notes,
        is_convert: map.is_convert,
    })
}

pub struct DifficultyValues {
    pub strain: Strain,
    pub max_combo: u32,
    pub n_hold_notes: u32,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let take = difficulty.get_passed_objects();
        let total_columns = map.cs.round_ties_even().max(1.0);
        let clock_rate = difficulty.get_clock_rate();
        let mut params = ObjectParams::new(map);

        let mania_objects = map
            .hit_objects
            .iter()
            .map(|h| ManiaObject::new(h, total_columns, &mut params))
            .take(take);

        let diff_objects = Self::create_difficulty_objects(clock_rate, mania_objects);

        let mut strain = Strain::new(total_columns as usize);

        for curr in diff_objects.iter() {
            strain.process(curr, &diff_objects);
        }

        Self {
            strain,
            max_combo: params.max_combo(),
            n_hold_notes: params.n_hold_notes(),
        }
    }

    pub fn create_difficulty_objects(
        clock_rate: f64,
        mut mania_objects: impl ExactSizeIterator<Item = ManiaObject>,
    ) -> Box<[ManiaDifficultyObject]> {
        let Some(first) = mania_objects.next() else {
            return Box::default();
        };

        let n_diff_objects = mania_objects.len();

        let diff_objects_iter = mania_objects.enumerate().scan(first, |last, (i, base)| {
            let diff_object = ManiaDifficultyObject::new(&base, last, clock_rate, i);
            *last = base;

            Some(diff_object)
        });

        let mut diff_objects = Vec::with_capacity(n_diff_objects);
        diff_objects.extend(diff_objects_iter);

        debug_assert_eq!(n_diff_objects, diff_objects.len());

        diff_objects.into_boxed_slice()
    }
}
