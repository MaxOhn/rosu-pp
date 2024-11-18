use rosu_map::section::general::GameMode;

use crate::{
    any::difficulty::{skills::Skill, Difficulty},
    catch::{
        catcher::Catcher, convert::convert_objects, difficulty::object::CatchDifficultyObject,
    },
    model::{beatmap::BeatmapAttributes, mode::ConvertError},
    Beatmap,
};

use self::skills::movement::Movement;

use super::{
    attributes::{CatchDifficultyAttributes, ObjectCountBuilder},
    object::palpable::PalpableObject,
};

pub mod gradual;
mod object;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 4.59;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<CatchDifficultyAttributes, ConvertError> {
    let map = map.convert_ref(GameMode::Catch, difficulty.get_mods())?;

    let DifficultyValues {
        movement,
        mut attrs,
    } = DifficultyValues::calculate(difficulty, &map);

    DifficultyValues::eval(&mut attrs, movement.difficulty_value());

    Ok(attrs)
}

pub struct CatchDifficultySetup {
    map_attrs: BeatmapAttributes,
    attrs: CatchDifficultyAttributes,
}

impl CatchDifficultySetup {
    pub fn new(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let map_attrs = map.attributes().difficulty(difficulty).build();

        let attrs = CatchDifficultyAttributes {
            ar: map_attrs.ar,
            is_convert: map.is_convert,
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
    pub fn calculate(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let CatchDifficultySetup {
            map_attrs,
            mut attrs,
        } = CatchDifficultySetup::new(difficulty, map);

        let hr_offsets = difficulty.get_hardrock_offsets();
        let reflection = difficulty.get_mods().reflection();
        let mut count = ObjectCountBuilder::new_regular(take);

        let palpable_objects =
            convert_objects(map, &mut count, reflection, hr_offsets, map_attrs.cs as f32);

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
        attrs.stars = movement_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
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
