use crate::{
    any::difficulty::{skills::Skill, ModeDifficulty},
    catch::{
        catcher::Catcher, convert::convert_objects, difficulty::object::CatchDifficultyObject,
    },
    util::mods::Mods,
};

use self::skills::movement::Movement;

use super::{
    attributes::{CatchDifficultyAttributes, CatchDifficultyAttributesBuilder},
    convert::CatchBeatmap,
};

mod object;
mod skills;

const STAR_SCALING_FACTOR: f64 = 0.153;

pub fn difficulty(
    difficulty: &ModeDifficulty,
    converted: &CatchBeatmap<'_>,
) -> CatchDifficultyAttributes {
    let DifficultyValues {
        movement,
        mut attrs,
    } = DifficultyValues::calculate(difficulty, converted);

    attrs.stars = movement.difficulty_value().sqrt() * STAR_SCALING_FACTOR;

    attrs
}

pub struct DifficultyValues {
    pub movement: Movement,
    pub attrs: CatchDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &ModeDifficulty, converted: &CatchBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let map_attrs = converted
            .attributes()
            .mods(difficulty.get_mods())
            .clock_rate(clock_rate)
            .build();

        let attrs = CatchDifficultyAttributes {
            ar: map_attrs.ar,
            is_convert: converted.is_convert,
            ..Default::default()
        };

        let mut attrs = CatchDifficultyAttributesBuilder::new(attrs, take);

        let hr = difficulty.get_mods().hr();
        let movement = Movement::new(clock_rate);

        let palpable_objects = convert_objects(converted, &mut attrs, hr, map_attrs.cs as f32);
        let mut palpable_objects_iter = palpable_objects.iter().take(take);

        let Some(mut last_object) = palpable_objects_iter.next() else {
            return Self {
                movement,
                attrs: attrs.into_inner(),
            };
        };

        let mut half_catcher_width = Catcher::calculate_catch_width(map_attrs.cs as f32) * 0.5;
        half_catcher_width *= 1.0 - ((map_attrs.cs as f32 - 5.5).max(0.0) * 0.0625);
        let scaling_factor =
            CatchDifficultyObject::NORMALIZED_HITOBJECT_RADIUS / half_catcher_width;

        let diff_objects: Vec<_> = palpable_objects_iter
            .enumerate()
            .map(|(i, hit_object)| {
                let diff_object = CatchDifficultyObject::new(
                    hit_object,
                    last_object,
                    clock_rate,
                    scaling_factor,
                    i,
                );
                last_object = hit_object;

                diff_object
            })
            .collect();

        let mut movement = Skill::new(movement, &diff_objects);

        for curr in diff_objects.iter() {
            movement.process(curr);
        }

        Self {
            movement: movement.inner,
            attrs: attrs.into_inner(),
        }
    }
}
