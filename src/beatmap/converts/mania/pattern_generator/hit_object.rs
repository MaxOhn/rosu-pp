use crate::{
    beatmap::{
        converts::mania::{
            legacy_random::Random, pattern::Pattern, pattern_type::PatternType, PrevValues,
        },
        EffectPoint,
    },
    mania::ManiaObject,
    parse::{HitObject, HitSound},
    Beatmap,
};

use super::PatternGenerator;

pub(crate) struct HitObjectPatternGenerator<'h> {
    pub(crate) hit_object: &'h HitObject,
    pub(crate) total_columns: i32,
    pub(crate) sample: u8,
    pub(crate) stair_type: PatternType,
    convert_type: PatternType,
    prev_pattern: &'h Pattern,
    random: &'h mut Random,
    orig: &'h Beatmap,
}

impl<'h> HitObjectPatternGenerator<'h> {
    pub(crate) fn new(
        random: &'h mut Random,
        hit_object: &'h HitObject,
        sample: u8,
        total_columns: i32,
        prev: &'h PrevValues,
        density: f64,
        orig: &'h Beatmap,
    ) -> Self {
        let timing_point = orig.timing_point_at(hit_object.start_time);

        let pos_separation = (hit_object.pos - prev.pos).length();
        let time_separation = hit_object.start_time - prev.time;

        let mut convert_type = PatternType::default();

        if time_separation <= 80.0 {
            // * More than 187 BPM
            convert_type |= PatternType::FORCE_NOT_STACK | PatternType::KEEP_SINGLE;
        } else if time_separation <= 95.0 {
            // * More than 157 BPM
            convert_type |= PatternType::FORCE_NOT_STACK | PatternType::KEEP_SINGLE | prev.stair;
        } else if time_separation <= 105.0 {
            // * More than 140 BPM
            convert_type |= PatternType::FORCE_NOT_STACK | PatternType::LOW_PROBABILITY;
        } else if time_separation <= 125.0 {
            // * More than 120 BPM
            convert_type |= PatternType::FORCE_NOT_STACK;
        } else if time_separation <= 135.0 && pos_separation < 20.0 {
            // * More than 111 BPM stream
            convert_type |= PatternType::CYCLE | PatternType::KEEP_SINGLE;
        } else if time_separation <= 150.0 && pos_separation < 20.0 {
            // * More than 100 BPM stream
            convert_type |= PatternType::FORCE_STACK | PatternType::LOW_PROBABILITY;
        } else if pos_separation < 20.0 && density >= timing_point.beat_len / 2.5 {
            // * Low density stream
            convert_type |= PatternType::REVERSE | PatternType::LOW_PROBABILITY;
        } else if density < timing_point.beat_len / 2.5 {
            // * High density
        } else {
            let kiai = orig
                .effect_point_at(hit_object.start_time)
                .map_or(EffectPoint::DEFAULT_KIAI, |point| point.kiai);

            if kiai {
                // * High density
            } else {
                convert_type |= PatternType::LOW_PROBABILITY;
            }
        }

        if !convert_type.contains(PatternType::KEEP_SINGLE) {
            if sample.finish() && total_columns != 8 {
                convert_type |= PatternType::MIRROR;
            } else if sample.clap() {
                convert_type |= PatternType::GATHERED;
            }
        }

        Self {
            hit_object,
            stair_type: prev.stair,
            convert_type,
            total_columns,
            sample,
            prev_pattern: &prev.pattern,
            random,
            orig,
        }
    }

    pub(crate) fn generate(&mut self) -> Pattern {
        let pattern = self.generate_core();

        for obj in pattern.hit_objects.iter() {
            let col = ManiaObject::column(obj.pos.x, self.total_columns as f32) as i32;

            if self.convert_type.contains(PatternType::STAIR) && col == self.total_columns - 1 {
                self.stair_type = PatternType::REVERSE_STAIR;
            }

            if self.convert_type.contains(PatternType::REVERSE_STAIR) && col == self.random_start()
            {
                self.stair_type = PatternType::STAIR;
            }
        }

        pattern
    }

    fn generate_core(&mut self) -> Pattern {
        if self.total_columns == 1 {
            return Pattern::new_note(self, 0);
        }

        let last_column = self.prev_pattern.hit_objects.last().map_or(0, |h| {
            ManiaObject::column(h.pos.x, self.total_columns as f32) as u8
        });

        let random_start = self.random_start() as u8;

        if self.convert_type.contains(PatternType::REVERSE)
            && !self.prev_pattern.hit_objects.is_empty()
        {
            let mut pattern = Pattern::default();

            for i in random_start..self.total_columns as u8 {
                if self.prev_pattern.column_has_obj(i) {
                    pattern.add_note(self, random_start + self.total_columns as u8 - i - 1);
                }
            }

            return pattern;
        }

        if self.convert_type.contains(PatternType::CYCLE)
            && self.prev_pattern.hit_objects.len() == 1
            // * If we convert to 7K + 1, let's not overload the special key
            && (self.total_columns != 8 || last_column != 0)
            // * Make sure the last column was not the centre column
            && (self.total_columns % 2 == 0 || last_column != self.total_columns as u8 / 2)
        {
            // * Generate a new pattern by cycling backwards (similar to Reverse but for only one hit object)
            let column = random_start + self.total_columns as u8 - last_column - 1;

            return Pattern::new_note(self, column);
        }

        if self.convert_type.contains(PatternType::FORCE_STACK)
            && !self.prev_pattern.hit_objects.is_empty()
        {
            let mut pattern = Pattern::default();

            // * Generate a new pattern by placing on the already filled columns
            for i in random_start..self.total_columns as u8 {
                if self.prev_pattern.column_has_obj(i) {
                    pattern.add_note(self, i);
                }
            }

            return pattern;
        }

        if self.prev_pattern.hit_objects.len() == 1 {
            if self.convert_type.contains(PatternType::STAIR) {
                // * Generate a new pattern by placing on the next column,
                // * cycling back to the start if there is no "next"
                let mut target_column = last_column + 1;

                if target_column == self.total_columns as u8 {
                    target_column = random_start;
                }

                return Pattern::new_note(self, target_column);
            }

            if self.convert_type.contains(PatternType::REVERSE_STAIR) {
                // * Generate a new pattern by placing on the previous column,
                // * cycling back to the end if there is no "previous"
                let mut target_column = last_column as i8 - 1;

                if target_column == random_start as i8 - 1 {
                    target_column = self.total_columns as i8 - 1;
                }

                return Pattern::new_note(self, target_column as u8);
            }
        }

        if self.convert_type.contains(PatternType::KEEP_SINGLE) {
            return self.generate_random_notes(1);
        }

        let conversion_diff = self.conversion_difficulty();

        if self.convert_type.contains(PatternType::MIRROR) {
            if conversion_diff > 6.5 {
                self.generate_random_pattern_with_mirrored(0.12, 0.38, 0.12)
            } else if conversion_diff > 4.0 {
                self.generate_random_pattern_with_mirrored(0.12, 0.17, 0.0)
            } else {
                self.generate_random_pattern_with_mirrored(0.12, 0.0, 0.0)
            }
        } else if conversion_diff > 6.5 {
            if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
                self.generate_random_pattern(0.78, 0.42, 0.0, 0.0)
            } else {
                self.generate_random_pattern(1.0, 0.62, 0.0, 0.0)
            }
        } else if conversion_diff > 4.0 {
            if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
                self.generate_random_pattern(0.35, 0.08, 0.0, 0.0)
            } else {
                self.generate_random_pattern(0.52, 0.15, 0.0, 0.0)
            }
        } else if conversion_diff > 2.0 {
            if self.convert_type.contains(PatternType::LOW_PROBABILITY) {
                self.generate_random_pattern(0.18, 0.0, 0.0, 0.0)
            } else {
                self.generate_random_pattern(0.45, 0.0, 0.0, 0.0)
            }
        } else {
            self.generate_random_pattern(0.0, 0.0, 0.0, 0.0)
        }
    }

    fn generate_random_notes(&mut self, mut note_count: i32) -> Pattern {
        let mut pattern = Pattern::default();

        let allow_stacking = !self.convert_type.contains(PatternType::FORCE_NOT_STACK);

        if !allow_stacking {
            note_count =
                (self.total_columns - self.random_start() - self.prev_pattern.column_with_objs())
                    .min(note_count);
        }

        let mut next_column = self.get_column(Some(true));

        for _ in 0..note_count {
            next_column = if allow_stacking {
                self.find_available_column(
                    next_column,
                    None,
                    None,
                    Some(Self::get_next_column),
                    None,
                    &[&pattern],
                )
            } else {
                self.find_available_column(
                    next_column,
                    None,
                    None,
                    Some(Self::get_next_column),
                    None,
                    &[&pattern, self.prev_pattern],
                )
            };

            pattern.add_note(self, next_column);
        }

        pattern
    }

    fn get_next_column(&mut self, mut last: u8) -> u8 {
        if self.convert_type.contains(PatternType::GATHERED) {
            last += 1;

            if last == self.total_columns as u8 {
                last = self.random_start() as u8;
            }
        } else {
            last = PatternGenerator::get_random_column(self, None, None);
        }

        last
    }

    fn has_special_column(&self) -> bool {
        self.sample.clap() && self.sample.finish()
    }

    fn generate_random_pattern(&mut self, p2: f64, p3: f64, p4: f64, p5: f64) -> Pattern {
        let random_note_count = self.get_random_note_count(p2, p3, p4, p5);
        let mut pattern = self.generate_random_notes(random_note_count);

        if self.random_start() > 0 && self.has_special_column() {
            pattern.add_note(self, 0);
        }

        pattern
    }

    fn get_random_note_count(&mut self, mut p2: f64, mut p3: f64, mut p4: f64, mut p5: f64) -> i32 {
        match self.total_columns {
            2 => {
                p2 = 0.0;
                p3 = 0.0;
                p4 = 0.0;
                p5 = 0.0;
            }
            3 => {
                p2 = p2.min(0.1);
                p3 = 0.0;
                p4 = 0.0;
                p5 = 0.0;
            }
            4 => {
                p2 = p2.min(0.23);
                p3 = p3.min(0.04);
                p4 = 0.0;
                p5 = 0.0;
            }
            5 => {
                p3 = p3.min(0.15);
                p4 = p4.min(0.03);
                p5 = 0.0;
            }
            _ => {}
        }

        if self.sample.clap() {
            p2 = 1.0;
        }

        PatternGenerator::get_random_note_count(self, p2, p3, Some(p4), Some(p5), None)
    }

    fn generate_random_pattern_with_mirrored(
        &mut self,
        centre_probability: f64,
        p2: f64,
        p3: f64,
    ) -> Pattern {
        if self.convert_type.contains(PatternType::FORCE_NOT_STACK) {
            return self.generate_random_pattern(1.0 / 2.0 + p2 / 2.0, p2, (p2 + p3) / 2.0, p3);
        }

        let mut pattern = Pattern::default();

        let (note_count, add_to_centre) =
            self.get_random_note_count_mirrored(centre_probability, p2, p3);

        let column_limit = if self.total_columns % 2 == 0 {
            self.total_columns / 2
        } else {
            (self.total_columns - 1) / 2
        };

        let mut next_column = PatternGenerator::get_random_column(self, None, Some(column_limit));

        for _ in 0..note_count {
            next_column = self.find_available_column(
                next_column,
                None,
                Some(column_limit),
                None,
                None,
                &[&pattern],
            );

            // * Add normal note
            pattern.add_note(self, next_column);

            // * Add mirrored note
            let column = (self.random_start() + self.total_columns) as u8 - next_column - 1;
            pattern.add_note(self, column);
        }

        if add_to_centre {
            pattern.add_note(self, self.total_columns as u8 / 2);
        }

        if self.random_start() > 0 && self.has_special_column() {
            pattern.add_note(self, 0);
        }

        pattern
    }

    fn get_random_note_count_mirrored(
        &mut self,
        mut centre_probability: f64,
        mut p2: f64,
        mut p3: f64,
    ) -> (i32, bool) {
        match self.total_columns {
            2 => {
                centre_probability = 0.0;
                p2 = 0.0;
                p3 = 0.0;
            }
            3 => {
                centre_probability = centre_probability.min(0.03);
                p2 = 0.0;
                p3 = 0.0;
            }
            4 => {
                centre_probability = 0.0;

                // * Stable requires rngValue > x, which is an inverse-probability. Lazer uses true probability (1 - x).
                // * But multiplying this value by 2 (stable) is not the same operation as dividing it by 2 (lazer),
                // * so it needs to be converted to from a probability and then back after the multiplication.
                p2 = 1.0 - ((1.0 - p2) * 2.0).max(0.8);
                p3 = 0.0;
            }
            5 => {
                centre_probability = centre_probability.min(0.03);
                p3 = 0.0;
            }
            6 => {
                centre_probability = 0.0;

                // * Stable requires rngValue > x, which is an inverse-probability. Lazer uses true probability (1 - x).
                // * But multiplying this value by 2 (stable) is not the same operation as dividing it by 2 (lazer),
                // * so it needs to be converted to from a probability and then back after the multiplication.
                p2 = 1.0 - ((1.0 - p2) * 2.0).max(0.05);
                p3 = 1.0 - ((1.0 - p3) * 2.0).max(0.85);
            }
            _ => {}
        }

        // * The stable values were allowed to exceed 1, which indicate <0% probability.
        // * These values needs to be clamped otherwise GetRandomNoteCount() will throw an exception.
        p2 = p2.clamp(0.0, 1.0);
        p3 = p3.clamp(0.0, 1.0);

        let centre_val = self.random.gen_double();
        let note_count = PatternGenerator::get_random_note_count(self, p2, p3, None, None, None);
        let add_to_centre =
            self.total_columns % 2 != 0 && note_count != 3 && centre_val > 1.0 - centre_probability;

        (note_count, add_to_centre)
    }
}

impl PatternGenerator for HitObjectPatternGenerator<'_> {
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
