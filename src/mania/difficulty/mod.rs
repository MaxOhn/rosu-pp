use std::cmp;

use rosu_map::section::general::GameMode;

use crate::{
    any::difficulty::{skills::Skill, Difficulty},
    mania::{
        difficulty::{object::ManiaDifficultyObject, skills::strain::Strain},
        object::{ManiaObject, ObjectParams},
    },
    model::mode::ConvertError,
    Beatmap,
};

use super::attributes::ManiaDifficultyAttributes;

pub mod gradual;
mod object;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.018;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<ManiaDifficultyAttributes, ConvertError> {
    let map = map.convert_ref(GameMode::Mania, difficulty.get_mods())?;

    let n_objects = cmp::min(difficulty.get_passed_objects(), map.hit_objects.len()) as u32;

    let values = DifficultyValues::calculate(difficulty, &map);

    let hit_window = map
        .attributes()
        .difficulty(difficulty)
        .hit_windows()
        .od_great;

    Ok(ManiaDifficultyAttributes {
        stars: values.strain.difficulty_value() * DIFFICULTY_MULTIPLIER,
        hit_window,
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

        {
            let mut strain = Skill::new(&mut strain, &diff_objects);

            for curr in diff_objects.iter() {
                strain.process(curr);
            }
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
