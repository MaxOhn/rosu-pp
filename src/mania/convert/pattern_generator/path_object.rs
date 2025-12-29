use std::cmp;

use rosu_map::section::hit_objects::hit_samples::HitSoundType;

use crate::{
    mania::{
        convert::{pattern::Pattern, pattern_type::PatternType},
        object::ManiaObject,
    },
    model::{
        beatmap::Beatmap,
        control_point::{DifficultyPoint, EffectPoint, TimingPoint},
        hit_object::HitObject,
    },
    util::{get_precision_adjusted_beat_len, random::osu::Random},
};

use super::PatternGenerator;

pub struct PathObjectPatternGenerator<'h> {
    pub segment_duration: i32,
    pub sample: HitSoundType,
    pub inner: PatternGenerator<'h>,
    start_time: i32,
    end_time: i32,
    span_count: i32,
    prev_pattern: &'h Pattern,
    convert_type: PatternType,
    node_sounds: &'h [HitSoundType],
}

impl<'h> PathObjectPatternGenerator<'h> {
    #[expect(clippy::too_many_arguments, reason = "it is what it is /shrug")]
    pub fn new(
        random: &'h mut Random,
        hit_object: &'h HitObject,
        sample: HitSoundType,
        total_columns: i32,
        prev_pattern: &'h Pattern,
        orig: &'h Beatmap,
        repeats: usize,
        expected_dist: Option<f64>,
        node_sounds: &'h [HitSoundType],
    ) -> Self {
        let timing_beat_len = orig
            .timing_point_at(hit_object.start_time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

        let slider_velocity = orig
            .difficulty_point_at(hit_object.start_time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            });

        let kiai = orig
            .effect_point_at(hit_object.start_time)
            .map_or(EffectPoint::DEFAULT_KIAI, |point| point.kiai);

        let convert_type = if kiai {
            PatternType::default()
        } else {
            PatternType::LOW_PROBABILITY
        };

        let beat_len = get_precision_adjusted_beat_len(slider_velocity, timing_beat_len);

        let span_count = (repeats + 1) as i32;
        let start_time = hit_object.start_time.round_ties_even() as i32;

        let dist = expected_dist.unwrap_or(0.0);

        // * This matches stable's calculation.
        let end_time = (f64::from(start_time)
            + dist * beat_len * f64::from(span_count) * 0.01 / orig.slider_multiplier)
            .floor() as i32;

        let segment_duration = (end_time - start_time) / span_count;

        let inner = PatternGenerator::new(hit_object, total_columns, random, orig);

        Self {
            segment_duration,
            sample,
            inner,
            start_time,
            end_time,
            span_count,
            prev_pattern,
            convert_type,
            node_sounds,
        }
    }

    pub fn generate(&mut self) -> Vec<Pattern> {
        let orig_pattern = self.generate_();

        if orig_pattern.hit_objects.len() == 1 {
            return vec![orig_pattern];
        }

        // * We need to split the intermediate pattern into two new patterns:
        // * 1. A pattern containing all objects that do not end at our EndTime.
        // * 2. A pattern containing all objects that end at our EndTime. This will be used for further pattern generation.
        let mut intermediate_pattern = Pattern::default();
        let mut end_time_pattern = Pattern::default();

        for obj in orig_pattern.hit_objects {
            let col = ManiaObject::column(obj.pos.x, self.inner.total_columns as f32) as u8;

            #[expect(clippy::if_not_else, reason = "staying in-sync with lazer")]
            if self.end_time != obj.end_time().round_ties_even() as i32 {
                intermediate_pattern.add_object(obj, col);
            } else {
                end_time_pattern.add_object(obj, col);
            }
        }

        vec![intermediate_pattern, end_time_pattern]
    }

    fn generate_(&mut self) -> Pattern {
        let conversion_diff = self.inner.conversion_difficulty();

        if self.inner.total_columns == 1 {
            Pattern::new_slider_note(self, 0, self.start_time, self.end_time)
        } else if self.span_count > 1 {
            if self.segment_duration <= 90 {
                self.generate_random_hold_notes(self.start_time, 1)
            } else if self.segment_duration <= 120 {
                self.convert_type |= PatternType::FORCE_NOT_STACK;

                self.generate_random_notes(self.start_time, self.span_count + 1)
            } else if self.segment_duration <= 160 {
                self.generate_stair(self.start_time)
            } else if self.segment_duration <= 200 && conversion_diff > 3.0 {
                self.generate_random_multiple_notes(self.start_time)
            } else if self.end_time - self.start_time >= 4000 {
                self.generate_n_random_notes(self.start_time, 0.23, 0.0, 0.0)
            } else if self.segment_duration > 400
                && self.span_count < self.inner.total_columns - 1 - self.inner.random_start()
            {
                self.generate_tiled_hold_notes(self.start_time)
            } else {
                self.generate_hold_and_normal_notes(self.start_time, conversion_diff)
            }
        } else if self.segment_duration <= 110 {
            if self.prev_pattern.column_with_objs() < self.inner.total_columns {
                self.convert_type |= PatternType::FORCE_NOT_STACK;
            } else {
                self.convert_type &= !PatternType::FORCE_NOT_STACK;
            }

            let note_count = 1 + i32::from(self.segment_duration >= 80);

            self.generate_random_notes(self.start_time, note_count)
        } else if conversion_diff > 6.5 {
            if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
                self.generate_n_random_notes(self.start_time, 0.78, 0.3, 0.0)
            } else {
                self.generate_n_random_notes(self.start_time, 0.85, 0.36, 0.03)
            }
        } else if conversion_diff > 4.0 {
            if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
                self.generate_n_random_notes(self.start_time, 0.43, 0.08, 0.0)
            } else {
                self.generate_n_random_notes(self.start_time, 0.56, 0.18, 0.0)
            }
        } else if conversion_diff > 2.5 {
            if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
                self.generate_n_random_notes(self.start_time, 0.3, 0.0, 0.0)
            } else {
                self.generate_n_random_notes(self.start_time, 0.37, 0.08, 0.0)
            }
        } else if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
            self.generate_n_random_notes(self.start_time, 0.17, 0.0, 0.0)
        } else {
            self.generate_n_random_notes(self.start_time, 0.27, 0.0, 0.0)
        }
    }

    fn generate_random_hold_notes(&mut self, start_time: i32, note_count: i32) -> Pattern {
        // * - - - -
        // * ■ - ■ ■
        // * □ - □ □
        // * ■ - ■ ■

        let mut pattern = Pattern::default();

        let random_start = self.inner.random_start();
        let usable_columns =
            self.inner.total_columns - random_start - self.prev_pattern.column_with_objs();
        let mut next_column = self.inner.get_random_column(None, None);

        for _ in 0..cmp::min(usable_columns, note_count) {
            // * Find available column
            next_column =
                self.find_available_column(next_column, None, &[&pattern, self.prev_pattern]);

            pattern.add_slider_note(self, next_column, start_time, self.end_time);
        }

        // * This is can't be combined with the above loop due to RNG
        for _ in 0..note_count.saturating_sub(usable_columns) {
            next_column = self.find_available_column(next_column, None, &[&pattern]);

            pattern.add_slider_note(self, next_column, start_time, self.end_time);
        }

        pattern
    }

    fn generate_random_notes(&mut self, mut start_time: i32, note_count: i32) -> Pattern {
        // * - - - -
        // * x - - -
        // * - - x -
        // * - - - x
        // * x - - -

        let mut next_column = self.inner.get_column(Some(true));

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK)
            && self.prev_pattern.column_with_objs() < self.inner.total_columns
        {
            next_column = self.find_available_column(next_column, None, &[self.prev_pattern]);
        }

        let mut last_column = next_column;
        let mut pattern = Pattern::with_capacity(note_count as usize);

        for _ in 0..note_count {
            pattern.add_slider_note(self, next_column, start_time, start_time);

            next_column = self.find_available_column(
                next_column,
                Some(&|c| c != i32::from(last_column)),
                &[],
            );

            last_column = next_column;
            start_time += self.segment_duration;
        }

        pattern
    }

    fn generate_stair(&mut self, mut start_time: i32) -> Pattern {
        // * - - - -
        // * x - - -
        // * - x - -
        // * - - x -
        // * - - - x
        // * - - x -
        // * - x - -
        // * x - - -

        let mut column = i32::from(self.inner.get_column(Some(true)));
        let mut increasing = self.inner.random.next_double() > 0.5;
        let mut pattern = Pattern::with_capacity(self.span_count as usize + 1);

        for _ in 0..=self.span_count as usize {
            pattern.add_slider_note(self, column as u8, start_time, start_time);
            start_time += self.segment_duration;

            // * Check if we're at the borders of the stage, and invert the pattern if so
            if increasing {
                if column >= self.inner.total_columns - 1 {
                    increasing = false;
                    column -= 1;
                } else {
                    column += 1;
                }
            } else if column <= self.inner.random_start() {
                increasing = true;
                column += 1;
            } else {
                column -= 1;
            }
        }

        pattern
    }

    fn generate_random_multiple_notes(&mut self, mut start_time: i32) -> Pattern {
        // * - - - -
        // * x - - -
        // * - x x -
        // * - - - x
        // * x - x -

        let legacy = (4..=8).contains(&self.inner.total_columns);
        let interval = self
            .inner
            .random
            .next_int_range(1, self.inner.total_columns - i32::from(legacy));

        let mut next_column = i32::from(self.inner.get_column(Some(true)));
        let random_start = self.inner.random_start();
        let not_2k = self.inner.total_columns > 2;
        let mut pattern =
            Pattern::with_capacity((self.span_count as usize + 1) * (1 + usize::from(not_2k)));

        for _ in 0..=self.span_count as usize {
            pattern.add_slider_note(self, next_column as u8, start_time, start_time);

            next_column += interval;

            if next_column >= self.inner.total_columns - random_start {
                next_column =
                    next_column - self.inner.total_columns - random_start + i32::from(legacy);
            }

            next_column += random_start;

            // * If we're in 2K, let's not add many consecutive doubles
            if not_2k {
                pattern.add_slider_note(self, next_column as u8, start_time, start_time);
            }

            next_column = i32::from(self.inner.get_random_column(None, None));
            start_time += self.segment_duration;
        }

        pattern
    }

    fn generate_n_random_notes(
        &mut self,
        start_time: i32,
        mut p2: f64,
        mut p3: f64,
        mut p4: f64,
    ) -> Pattern {
        // * - - - -
        // * ■ - ■ ■
        // * □ - □ □
        // * ■ - ■ ■

        match self.inner.total_columns {
            2 => {
                p2 = 0.0;
                p3 = 0.0;
                p4 = 0.0;
            }
            3 => {
                p2 = p2.min(0.1);
                p3 = 0.0;
                p4 = 0.0;
            }
            4 => {
                p2 = p2.min(0.3);
                p3 = p3.min(0.04);
                p4 = 0.0;
            }
            5 => {
                p2 = p2.min(0.34);
                p3 = p3.min(0.1);
                p4 = p4.min(0.03);
            }
            _ => {}
        }

        let is_double_sample =
            |sample: HitSoundType| sample.has_flag(HitSoundType::CLAP | HitSoundType::FINISH);

        let can_generate_two_notes = !self.convert_type.contains(PatternType::LOW_PROBABILITY)
            && (is_double_sample(self.sample)
                || is_double_sample(self.sample_info_list_at(self.start_time)));

        if can_generate_two_notes {
            p2 = 1.0;
        }

        let note_count = self
            .inner
            .get_random_note_count(p2, p3, Some(p4), None, None);

        self.generate_random_hold_notes(start_time, note_count)
    }

    fn generate_tiled_hold_notes(&mut self, mut start_time: i32) -> Pattern {
        // * - - - -
        // * ■ ■ ■ ■
        // * □ □ □ □
        // * □ □ □ □
        // * □ □ □ ■
        // * □ □ ■ -
        // * □ ■ - -
        // * ■ - - -

        let column_repeat = cmp::min(self.span_count, self.inner.total_columns) as usize;

        // * Due to integer rounding, this is not guaranteed to be the same as EndTime (the class-level variable).
        let end_time = start_time + self.segment_duration * self.span_count;

        let mut next_column = self.inner.get_column(Some(true));

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK)
            && self.prev_pattern.column_with_objs() < self.inner.total_columns
        {
            next_column = self.find_available_column(next_column, None, &[self.prev_pattern]);
        }

        let mut pattern = Pattern::with_capacity(column_repeat);

        for _ in 0..column_repeat {
            next_column = self.find_available_column(next_column, None, &[&pattern]);
            pattern.add_slider_note(self, next_column, start_time, end_time);
            start_time += self.segment_duration;
        }

        pattern
    }

    fn generate_hold_and_normal_notes(
        &mut self,
        mut start_time: i32,
        conversion_diff: f64,
    ) -> Pattern {
        // * - - - -
        // * ■ x x -
        // * ■ - x x
        // * ■ x - x
        // * ■ - x x

        let mut pattern = Pattern::default();

        let mut hold_column = self.inner.get_column(Some(true));

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK)
            && self.prev_pattern.column_with_objs() < self.inner.total_columns
        {
            hold_column = self.find_available_column(hold_column, None, &[self.prev_pattern]);
        }

        // * Create the hold note
        pattern.add_slider_note(self, hold_column, start_time, self.end_time);

        let mut next_column = self.inner.get_random_column(None, None);

        let mut note_count = if conversion_diff > 6.5 {
            self.inner
                .get_random_note_count(0.63, 0.0, None, None, None)
        } else if conversion_diff > 4.0 {
            let p2 = if self.inner.total_columns < 6 {
                0.12
            } else {
                0.45
            };

            self.inner.get_random_note_count(p2, 0.0, None, None, None)
        } else if conversion_diff > 2.5 {
            let p2 = if self.inner.total_columns < 6 {
                0.0
            } else {
                0.24
            };

            self.inner.get_random_note_count(p2, 0.0, None, None, None)
        } else {
            0
        };

        note_count = cmp::min(note_count, self.inner.total_columns - 1);

        let sample = self.sample_info_list_at(start_time);
        let ignore_head =
            !sample.has_flag(HitSoundType::WHISTLE | HitSoundType::FINISH | HitSoundType::CLAP);

        let mut row_pattern = Pattern::default();
        let hold_column = i32::from(hold_column);

        for _ in 0..=self.span_count as usize {
            if !(ignore_head && start_time == self.start_time) {
                for _ in 0..note_count {
                    next_column = self.find_available_column(
                        next_column,
                        Some(&|c| c != hold_column),
                        &[&row_pattern],
                    );

                    row_pattern.add_slider_note(self, next_column, start_time, start_time);
                }
            }

            pattern.append(&mut row_pattern);
            start_time += self.segment_duration;
        }

        pattern
    }

    fn sample_info_list_at(&self, time: i32) -> HitSoundType {
        self.note_samples_at(time)
            .first()
            .map_or(self.sample, |sample| *sample)
    }

    fn note_samples_at(&self, time: i32) -> &[HitSoundType] {
        let idx = if self.segment_duration == 0 {
            0
        } else {
            ((time - self.start_time) / self.segment_duration) as usize
        };

        &self.node_sounds[idx..]
    }

    fn find_available_column(
        &mut self,
        mut initial_column: u8,
        validation: Option<&dyn Fn(i32) -> bool>,
        patterns: &[&Pattern],
    ) -> u8 {
        let lower = self.inner.random_start();
        let upper = self.inner.total_columns;

        let is_valid = |column: i32| {
            if let Some(fun) = validation
                && !(fun)(column)
            {
                return false;
            }

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
