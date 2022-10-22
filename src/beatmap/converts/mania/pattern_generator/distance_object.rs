use crate::{
    beatmap::{
        converts::mania::{legacy_random::Random, pattern::Pattern, pattern_type::PatternType},
        EffectPoint,
    },
    curve::Curve,
    mania::ManiaObject,
    parse::{HitObject, HitSound},
    util::FloatExt,
    Beatmap,
};

use super::PatternGenerator;

pub(crate) struct DistanceObjectPatternGenerator<'h> {
    pub(crate) hit_object: &'h HitObject,
    pub(crate) segment_duration: i32,
    pub(crate) total_columns: i32,
    pub(crate) sample: u8,
    start_time: i32,
    end_time: i32,
    span_count: i32,
    orig: &'h Beatmap,
    prev_pattern: &'h Pattern,
    convert_type: PatternType,
    random: &'h mut Random,
    edge_sounds: &'h [u8],
}

impl<'h> DistanceObjectPatternGenerator<'h> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        random: &'h mut Random,
        hit_object: &'h HitObject,
        sample: u8,
        total_columns: i32,
        prev_pattern: &'h Pattern,
        orig: &'h Beatmap,
        repeats: usize,
        curve: &Curve<'_>,
        edge_sounds: &'h [u8],
    ) -> Self {
        let timing_point = orig.timing_point_at(hit_object.start_time);

        let difficulty_point = orig
            .difficulty_point_at(hit_object.start_time)
            .unwrap_or_default();

        let kiai = orig
            .effect_point_at(hit_object.start_time)
            .map_or(EffectPoint::DEFAULT_KIAI, |point| point.kiai);

        let convert_type = if kiai {
            PatternType::default()
        } else {
            PatternType::LOW_PROBABILITY
        };

        let beat_len = timing_point.beat_len * difficulty_point.bpm_mult;

        let span_count = (repeats + 1) as i32;
        let start_time = hit_object.start_time.round_even() as i32;

        // * This matches stable's calculation.
        let end_time = (start_time as f64
            + curve.dist() * beat_len * span_count as f64 * 0.01 / orig.slider_mult)
            .floor() as i32;

        let segment_duration = (end_time - start_time) / span_count;

        Self {
            hit_object,
            segment_duration,
            total_columns,
            sample,
            start_time,
            end_time,
            span_count,
            orig,
            prev_pattern,
            convert_type,
            random,
            edge_sounds,
        }
    }

    pub(crate) fn generate(&mut self) -> Vec<Pattern> {
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
            let col = ManiaObject::column(obj.pos.x, self.total_columns as f32) as u8;

            if self.end_time != obj.end_time().round_even() as i32 {
                intermediate_pattern.add_object(obj, col);
            } else {
                end_time_pattern.add_object(obj, col);
            }
        }

        vec![intermediate_pattern, end_time_pattern]
    }

    fn generate_(&mut self) -> Pattern {
        let conversion_diff = self.conversion_difficulty();

        if self.total_columns == 1 {
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
                && self.span_count < self.total_columns - 1 - self.random_start()
            {
                self.generate_tiled_hold_notes(self.start_time)
            } else {
                self.generate_hold_and_normal_notes(self.start_time, conversion_diff)
            }
        } else if self.segment_duration <= 110 {
            if self.prev_pattern.column_with_objs() < self.total_columns {
                self.convert_type |= PatternType::FORCE_NOT_STACK;
            } else {
                self.convert_type &= !PatternType::FORCE_NOT_STACK;
            }

            let note_count = 1 + (self.segment_duration >= 80) as i32;

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

        let random_start = self.random_start();
        let usable_columns =
            self.total_columns - random_start - self.prev_pattern.column_with_objs();
        let mut next_column = PatternGenerator::get_random_column(self, None, None);

        for _ in 0..usable_columns.min(note_count) {
            // * Find available column
            next_column = self.find_available_column(
                next_column,
                None,
                None,
                None,
                None,
                &[&pattern, self.prev_pattern],
            );

            pattern.add_slider_note(self, next_column, start_time, self.end_time);
        }

        // * This is can't be combined with the above loop due to RNG
        for _ in 0..note_count.saturating_sub(usable_columns) {
            next_column =
                self.find_available_column(next_column, None, None, None, None, &[&pattern]);

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

        let mut next_column = self.get_column(Some(true));

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK)
            && self.prev_pattern.column_with_objs() < self.total_columns
        {
            next_column = self.find_available_column(
                next_column,
                None,
                None,
                None,
                None,
                &[self.prev_pattern],
            );
        }

        let mut last_column = next_column;
        let mut pattern = Pattern::with_capacity(note_count as usize);

        for _ in 0..note_count {
            pattern.add_slider_note(self, next_column, start_time, start_time);

            next_column = self.find_available_column(
                next_column,
                None,
                None,
                None,
                Some(&|c| c != last_column as i32),
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

        let mut column = self.get_column(Some(true)) as i32;
        let mut increasing = self.random.gen_double() > 0.5;
        let mut pattern = Pattern::with_capacity(self.span_count as usize + 1);

        for _ in 0..=self.span_count as usize {
            pattern.add_slider_note(self, column as u8, start_time, start_time);
            start_time += self.segment_duration;

            // * Check if we're at the borders of the stage, and invert the pattern if so
            if increasing {
                if column >= self.total_columns - 1 {
                    increasing = false;
                    column -= 1;
                } else {
                    column += 1;
                }
            } else if column <= self.random_start() {
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

        let legacy = (4..=8).contains(&self.total_columns);
        let interval = self
            .random
            .gen_int_range(1, self.total_columns as i32 - (legacy as i32));

        let mut next_column = self.get_column(Some(true)) as i32;
        let random_start = self.random_start();
        let not_2k = self.total_columns > 2;
        let mut pattern =
            Pattern::with_capacity((self.span_count as usize + 1) * (1 + not_2k as usize));

        for _ in 0..=self.span_count as usize {
            pattern.add_slider_note(self, next_column as u8, start_time, start_time);

            next_column += interval;

            if next_column >= self.total_columns - random_start {
                next_column = next_column - self.total_columns - random_start + (legacy as i32);
            }

            next_column += random_start;

            // * If we're in 2K, let's not add many consecutive doubles
            if not_2k {
                pattern.add_slider_note(self, next_column as u8, start_time, start_time);
            }

            next_column = PatternGenerator::get_random_column(self, None, None) as i32;
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

        match self.total_columns {
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

        let is_double_sample = |sample: u8| sample.clap() || sample.finish();

        let can_generate_two_notes = !self.convert_type.contains(PatternType::LOW_PROBABILITY)
            && (is_double_sample(self.sample)
                || is_double_sample(self.sample_info_list_at(self.start_time)));

        if can_generate_two_notes {
            p2 = 1.0;
        }

        let note_count = self.get_random_note_count(p2, p3, Some(p4), None, None);

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

        let column_repeat = self.span_count.min(self.total_columns) as usize;

        // * Due to integer rounding, this is not guaranteed to be the same as EndTime (the class-level variable).
        let end_time = start_time + self.segment_duration * self.span_count;

        let mut next_column = self.get_column(Some(true));

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK)
            && self.prev_pattern.column_with_objs() < self.total_columns
        {
            next_column = self.find_available_column(
                next_column,
                None,
                None,
                None,
                None,
                &[self.prev_pattern],
            );
        }

        let mut pattern = Pattern::with_capacity(column_repeat);

        for _ in 0..column_repeat {
            next_column =
                self.find_available_column(next_column, None, None, None, None, &[&pattern]);
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

        let mut hold_column = self.get_column(Some(true));

        if self.convert_type.contains(PatternType::FORCE_NOT_STACK)
            && self.prev_pattern.column_with_objs() < self.total_columns
        {
            hold_column = self.find_available_column(
                hold_column,
                None,
                None,
                None,
                None,
                &[self.prev_pattern],
            );
        }

        // * Create the hold note
        pattern.add_slider_note(self, hold_column, start_time, self.end_time);

        let mut next_column = PatternGenerator::get_random_column(self, None, None);

        let mut note_count = if conversion_diff > 6.5 {
            self.get_random_note_count(0.63, 0.0, None, None, None)
        } else if conversion_diff > 4.0 {
            let p2 = if self.total_columns < 6 { 0.12 } else { 0.45 };

            self.get_random_note_count(p2, 0.0, None, None, None)
        } else if conversion_diff > 2.5 {
            let p2 = if self.total_columns < 6 { 0.0 } else { 0.24 };

            self.get_random_note_count(p2, 0.0, None, None, None)
        } else {
            0
        };

        note_count = note_count.min(self.total_columns - 1);

        let sample = self.sample_info_list_at(start_time);
        let ignore_head = !(sample.whistle() || sample.finish() || sample.clap());

        let mut row_pattern = Pattern::default();
        let hold_column = hold_column as i32;

        for _ in 0..=self.span_count as usize {
            if !(ignore_head && start_time == self.start_time) {
                for _ in 0..note_count {
                    next_column = self.find_available_column(
                        next_column,
                        None,
                        None,
                        None,
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

    fn sample_info_list_at(&self, time: i32) -> u8 {
        self.note_samples_at(time)
            .first()
            .map_or(self.sample, |sample| *sample)
    }

    fn note_samples_at(&self, time: i32) -> &[u8] {
        let idx = if self.segment_duration == 0 {
            0
        } else {
            ((time - self.start_time) / self.segment_duration) as usize
        };

        &self.edge_sounds[idx..]
    }
}

impl PatternGenerator for DistanceObjectPatternGenerator<'_> {
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

    #[inline]
    fn original_map(&self) -> &Beatmap {
        self.orig
    }
}
