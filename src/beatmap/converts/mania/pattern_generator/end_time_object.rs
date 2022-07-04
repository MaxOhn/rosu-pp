use crate::{
    beatmap::converts::mania::{
        legacy_random::Random, pattern::Pattern, pattern_type::PatternType,
    },
    parse::{HitObject, HitSound},
    Beatmap,
};

use super::PatternGenerator;

pub(crate) struct EndTimeObjectPatternGenerator<'h> {
    pub(crate) hit_object: &'h HitObject,
    pub(crate) end_time: f64,
    pub(crate) total_columns: i32,
    pub(crate) sample: u8,
    convert_type: PatternType,
    prev_pattern: &'h Pattern,
    random: &'h mut Random,
}

impl<'h> EndTimeObjectPatternGenerator<'h> {
    pub(crate) fn new(
        random: &'h mut Random,
        hit_object: &'h HitObject,
        end_time: f64,
        sample: u8,
        total_columns: i32,
        prev_pattern: &'h Pattern,
    ) -> Self {
        let convert_type = if prev_pattern.column_with_objs() == total_columns {
            PatternType::default()
        } else {
            PatternType::FORCE_NOT_STACK
        };

        Self {
            hit_object,
            end_time,
            total_columns,
            sample,
            convert_type,
            prev_pattern,
            random,
        }
    }

    pub(crate) fn generate(&mut self) -> Pattern {
        let generate_hold = self.end_time - self.hit_object.start_time >= 100.0;

        match self.total_columns {
            8 if self.sample.finish() && self.end_time - self.hit_object.start_time < 1000.0 => {
                Pattern::new_end_time_note(self, 0, generate_hold)
            }
            8 => {
                let column = self.get_random_column(self.random_start());

                Pattern::new_end_time_note(self, column, generate_hold)
            }
            _ => {
                let column = self.get_random_column(0);

                Pattern::new_end_time_note(self, column, generate_hold)
            }
        }
    }

    fn get_random_column(&mut self, lower: i32) -> u8 {
        let column = PatternGenerator::get_random_column(self, Some(lower), None);

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK) {
            self.find_available_column(column, Some(lower), None, None, None, &[self.prev_pattern])
        } else {
            self.find_available_column(column, Some(lower), None, None, None, &[])
        }
    }
}

impl PatternGenerator for EndTimeObjectPatternGenerator<'_> {
    #[inline]
    fn hit_object(&self) -> &HitObject {
        self.hit_object
    }

    #[inline]
    fn total_columns(&self) -> i32 {
        self.total_columns
    }

    #[inline]
    fn random(&mut self) -> &mut Random {
        self.random
    }

    fn original_map(&self) -> &Beatmap {
        panic!("trait method is not used")
    }
}
