use std::cmp;

use crate::{
    any::difficulty::{skills::Skill, Difficulty},
    mania::{
        difficulty::{object::ManiaDifficultyObject, skills::strain::Strain},
        object::{ManiaObject, ObjectParams},
    },
};

use super::{attributes::ManiaDifficultyAttributes, convert::ManiaBeatmap};

pub mod gradual;
mod object;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.018;

pub fn difficulty(
    difficulty: &Difficulty,
    converted: &ManiaBeatmap<'_>,
) -> ManiaDifficultyAttributes {
    let n_objects = cmp::min(difficulty.get_passed_objects(), converted.hit_objects.len()) as u32;

    let values = DifficultyValues::calculate(difficulty, converted);

    let hit_window = converted
        .attributes()
        .difficulty(difficulty)
        .hit_windows()
        .od;

    ManiaDifficultyAttributes {
        stars: values.strain.difficulty_value() * DIFFICULTY_MULTIPLIER,
        hit_window,
        max_combo: values.max_combo,
        n_objects,
        is_convert: converted.is_convert,
    }
}

pub struct DifficultyValues {
    pub strain: Strain,
    pub max_combo: u32,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, converted: &ManiaBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let total_columns = converted.cs.round_ties_even().max(1.0);
        let clock_rate = difficulty.get_clock_rate();
        let mut params = ObjectParams::new(converted);

        let mania_objects = converted
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
            max_combo: params.into_max_combo(),
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
