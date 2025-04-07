use crate::{
    mania::object::ManiaObject,
    model::{beatmap::Beatmap, hit_object::HitObject},
    util::random::osu::Random,
};

pub(super) mod end_time_object;
pub(super) mod hit_object;
pub(super) mod path_object;

pub struct PatternGenerator<'a> {
    pub hit_object: &'a HitObject,
    pub total_columns: i32,
    random: &'a mut Random,
    original_map: &'a Beatmap,
}

impl<'a> PatternGenerator<'a> {
    const fn new(
        hit_object: &'a HitObject,
        total_columns: i32,
        random: &'a mut Random,
        original_map: &'a Beatmap,
    ) -> Self {
        Self {
            hit_object,
            total_columns,
            random,
            original_map,
        }
    }

    fn random_start(&self) -> i32 {
        i32::from(self.total_columns == 8)
    }

    fn get_column(&self, allow_special: Option<bool>) -> u8 {
        let allow_special = allow_special.unwrap_or(false);

        if allow_special && self.total_columns == 8 {
            const LOCAL_X_DIVISOR: f32 = 512.0 / 7.0;

            ((self.hit_object.pos.x / LOCAL_X_DIVISOR).floor() as u8).clamp(0, 6) + 1
        } else {
            ManiaObject::column(self.hit_object.pos.x, self.total_columns as f32) as u8
        }
    }

    fn get_random_note_count(
        &mut self,
        p2: f64,
        p3: f64,
        p4: Option<f64>,
        p5: Option<f64>,
        p6: Option<f64>,
    ) -> i32 {
        let p4 = p4.unwrap_or(0.0);
        let p5 = p5.unwrap_or(0.0);
        let p6 = p6.unwrap_or(0.0);

        let val = self.random.next_double();

        if val >= 1.0 - p6 {
            6
        } else if val >= 1.0 - p5 {
            5
        } else if val >= 1.0 - p4 {
            4
        } else if val >= 1.0 - p3 {
            3
        } else {
            1 + i32::from(val >= 1.0 - p2)
        }
    }

    fn conversion_difficulty(&self) -> f64 {
        let orig = self.original_map;
        let last_obj_time = orig.hit_objects.last().map_or(0.0, |h| h.start_time);
        let first_obj_time = orig.hit_objects.first().map_or(0.0, |h| h.start_time);

        // * Drain time in seconds
        let total_break_time = orig.total_break_time();
        let mut drain_time = ((last_obj_time - first_obj_time - total_break_time) / 1000.0) as i32;

        if drain_time == 0 {
            drain_time = 10_000;
        }

        let mut conversion_difficulty = 0.0;
        conversion_difficulty += f64::from(orig.hp + orig.ar.clamp(4.0, 7.0)) / 1.5;
        conversion_difficulty += orig.hit_objects.len() as f64 / f64::from(drain_time) * 9.0;
        conversion_difficulty /= 38.0;
        conversion_difficulty *= 5.0;
        conversion_difficulty /= 1.15;
        conversion_difficulty = conversion_difficulty.min(12.0);

        conversion_difficulty
    }

    fn get_random_column(&mut self, lower: Option<i32>, upper: Option<i32>) -> u8 {
        let lower = lower.unwrap_or_else(|| self.random_start());
        let upper = upper.unwrap_or(self.total_columns);

        self.random.next_int_range(lower, upper) as u8
    }
}
