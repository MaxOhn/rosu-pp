use crate::{
    any::difficulty::{mode::ModeDifficulty, skills::Skill},
    catch::{
        catcher::Catcher, convert::convert_objects, difficulty::object::CatchDifficultyObject,
    },
    model::beatmap::BeatmapAttributes,
    util::mods::Mods,
};

use self::skills::movement::Movement;

use super::{
    attributes::{CatchDifficultyAttributes, ObjectCountBuilder},
    convert::CatchBeatmap,
    object::palpable::PalpableObject,
};

pub mod gradual;
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

    DifficultyValues::eval(&mut attrs, movement.difficulty_value());

    attrs
}

pub struct CatchDifficultySetup {
    map_attrs: BeatmapAttributes,
    attrs: CatchDifficultyAttributes,
}

impl CatchDifficultySetup {
    pub fn new(difficulty: &ModeDifficulty, converted: &CatchBeatmap<'_>) -> Self {
        let mods = difficulty.get_mods();
        let clock_rate = difficulty.get_clock_rate();

        let map_attrs = converted
            .attributes()
            .mods(mods)
            .clock_rate(clock_rate)
            .build();

        let attrs = CatchDifficultyAttributes {
            ar: map_attrs.ar,
            is_convert: converted.is_convert,
            ..Default::default()
        };

        Self { map_attrs, attrs }
    }
}

pub struct DifficultyValues {
    pub movement: Movement,
    pub attrs: CatchDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &ModeDifficulty, converted: &CatchBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let mods = difficulty.get_mods();
        let clock_rate = difficulty.get_clock_rate();

        let CatchDifficultySetup {
            map_attrs,
            mut attrs,
        } = CatchDifficultySetup::new(difficulty, converted);

        let hr = mods.hr();
        let mut count = ObjectCountBuilder::new_regular(take);

        let palpable_objects = convert_objects(converted, &mut count, hr, map_attrs.cs as f32);

        let diff_objects = Self::create_difficulty_objects(
            &map_attrs,
            clock_rate,
            palpable_objects.iter().take(take),
        );

        let mut movement = Movement::new(clock_rate);

        {
            let mut movement = Skill::new(&mut movement, &diff_objects);

            for curr in diff_objects.iter() {
                movement.process(curr);
            }
        }

        attrs.set_object_count(&count.into_regular());

        Self { movement, attrs }
    }

    pub fn eval(attrs: &mut CatchDifficultyAttributes, movement_difficulty_value: f64) {
        attrs.stars = movement_difficulty_value.sqrt() * STAR_SCALING_FACTOR;
    }

    pub fn create_difficulty_objects<'a>(
        map_attrs: &BeatmapAttributes,
        clock_rate: f64,
        mut palpable_objects: impl ExactSizeIterator<Item = &'a PalpableObject>,
    ) -> Box<[CatchDifficultyObject]> {
        let Some(mut last_object) = palpable_objects.next() else {
            return Box::default();
        };

        let mut half_catcher_width = Catcher::calculate_catch_width(map_attrs.cs as f32) * 0.5;
        half_catcher_width *= 1.0 - ((map_attrs.cs as f32 - 5.5).max(0.0) * 0.0625);
        let scaling_factor =
            CatchDifficultyObject::NORMALIZED_HITOBJECT_RADIUS / half_catcher_width;

        palpable_objects
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
            .collect()
    }
}
