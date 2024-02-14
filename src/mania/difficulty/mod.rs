use crate::{
    any::difficulty::{skills::Skill, ModeDifficulty},
    mania::{
        difficulty::{object::ManiaDifficultyObject, skills::strain::Strain},
        object::{ManiaObject, ObjectParams},
    },
    util::{float_ext::FloatExt, mods::Mods},
};

use super::{attributes::ManiaDifficultyAttributes, convert::ManiaBeatmap};

mod object;
mod skills;

const STAR_SCALING_FACTOR: f64 = 0.018;

pub fn difficulty(
    difficulty: &ModeDifficulty,
    converted: &ManiaBeatmap<'_>,
) -> ManiaDifficultyAttributes {
    let n_objects = converted.map.hit_objects.len() as u32;
    let values = DifficultyValues::calculate(difficulty, converted);

    let mods = difficulty.get_mods();
    let clock_rate = difficulty.get_clock_rate();
    let hit_window =
        (f64::from((get_hit_window(converted, mods) * clock_rate) as i32) / clock_rate).ceil();

    ManiaDifficultyAttributes {
        stars: values.strain.difficulty_value() * STAR_SCALING_FACTOR,
        hit_window,
        max_combo: values.max_combo,
        n_objects,
        is_convert: converted.is_convert,
    }
}

fn get_hit_window(converted: &ManiaBeatmap<'_>, mods: u32) -> f64 {
    fn apply_mod_adjustments(mut value: f64, mods: u32) -> f64 {
        if mods.hr() {
            value /= 1.4;
        } else if mods.ez() {
            value *= 1.4;
        }

        value
    }

    let od = f64::from(converted.map.od);

    if !converted.is_convert {
        let od = ((10.0 - od).max(0.0)).min(10.0);

        return apply_mod_adjustments(34.0 + 3.0 * od, mods);
    }

    if od.round_even() > 4.0 {
        return apply_mod_adjustments(34.0, mods);
    }

    apply_mod_adjustments(47.0, mods)
}

pub struct DifficultyValues {
    pub strain: Strain,
    pub max_combo: u32,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &ModeDifficulty, converted: &ManiaBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let total_columns = converted.map.cs.round_even().max(1.0);
        let clock_rate = difficulty.get_clock_rate();
        let mut params = ObjectParams::new(converted.map.as_ref());

        let mut mania_objects = converted
            .map
            .hit_objects
            .iter()
            .map(|h| ManiaObject::new(h, total_columns, &mut params))
            .take(take);

        let Some(first) = mania_objects.next() else {
            return DifficultyValues {
                strain: Strain::new(total_columns as usize),
                max_combo: 0,
            };
        };

        let n_diff_objects = mania_objects.len();

        let diff_objects_iter = mania_objects.enumerate().scan(first, |last, (i, base)| {
            let diff_object = ManiaDifficultyObject::new(&base, last, clock_rate, i);
            *last = base;

            Some(diff_object)
        });

        let mut diff_objects = Vec::with_capacity(n_diff_objects);
        diff_objects.extend(diff_objects_iter);

        let mut strain = Skill::new(Strain::new(total_columns as usize), &diff_objects);

        for curr in diff_objects.iter() {
            strain.process(curr);
        }

        Self {
            strain: strain.inner,
            max_combo: params.into_max_combo(),
        }
    }
}
