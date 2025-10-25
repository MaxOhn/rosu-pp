use rosu_map::section::hit_objects::hit_samples::HitSoundType;

use crate::{
    Beatmap,
    mania::convert::{pattern::Pattern, pattern_type::PatternType},
    model::hit_object::HitObject,
    util::random::osu::Random,
};

use super::PatternGenerator;

pub struct EndTimeObjectPatternGenerator<'h> {
    pub end_time: f64,
    pub sample: HitSoundType,
    pub inner: PatternGenerator<'h>,
    convert_type: PatternType,
    prev_pattern: &'h Pattern,
}

impl<'h> EndTimeObjectPatternGenerator<'h> {
    pub fn new(
        random: &'h mut Random,
        hit_object: &'h HitObject,
        end_time: f64,
        sample: HitSoundType,
        total_columns: i32,
        prev_pattern: &'h Pattern,
        orig: &'h Beatmap,
    ) -> Self {
        let convert_type = if prev_pattern.column_with_objs() == total_columns {
            PatternType::default()
        } else {
            PatternType::FORCE_NOT_STACK
        };

        let inner = PatternGenerator::new(hit_object, total_columns, random, orig);

        Self {
            end_time,
            sample,
            inner,
            convert_type,
            prev_pattern,
        }
    }

    pub fn generate(&mut self) -> Pattern {
        let generate_hold = self.end_time - self.inner.hit_object.start_time >= 100.0;

        match self.inner.total_columns {
            8 if self.sample.has_flag(HitSoundType::FINISH)
                && self.end_time - self.inner.hit_object.start_time < 1000.0 =>
            {
                Pattern::new_end_time_note(self, 0, generate_hold)
            }
            8 => {
                let column = self.get_random_column(self.inner.random_start());

                Pattern::new_end_time_note(self, column, generate_hold)
            }
            _ => {
                let column = self.get_random_column(0);

                Pattern::new_end_time_note(self, column, generate_hold)
            }
        }
    }

    fn get_random_column(&mut self, lower: i32) -> u8 {
        let column = self.inner.get_random_column(Some(lower), None);

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK) {
            self.find_available_column(column, Some(lower), &[self.prev_pattern])
        } else {
            self.find_available_column(column, Some(lower), &[])
        }
    }

    fn find_available_column(
        &mut self,
        mut initial_column: u8,
        lower: Option<i32>,
        patterns: &[&Pattern],
    ) -> u8 {
        let lower = lower.unwrap_or_else(|| self.inner.random_start());
        let upper = self.inner.total_columns;

        let is_valid = |column: i32| {
            let column = column as u8;

            patterns
                .iter()
                .all(|pattern| !pattern.column_has_obj(column))
        };

        // * Check for the initial column
        if is_valid(i32::from(initial_column)) {
            return initial_column;
        }

        // * Ensure that we have at least one free column, so that an endless loop is avoided
        let has_valid_column = (lower..upper).any(is_valid);
        assert!(has_valid_column);

        // * Iterate until a valid column is found. This is a random iteration in the default case.
        while {
            initial_column = self.inner.get_random_column(Some(lower), Some(upper));

            !is_valid(i32::from(initial_column))
        } {}

        initial_column
    }
}
