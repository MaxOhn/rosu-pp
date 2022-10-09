use crate::parse::HitObject;

pub(crate) struct ManiaObject<'h> {
    hit_object: &'h HitObject,
}

impl<'h> ManiaObject<'h> {
    pub(crate) fn new(hit_object: &'h HitObject) -> Self {
        Self { hit_object }
    }

    pub(crate) fn start_time(&self) -> f64 {
        self.hit_object.start_time
    }

    pub(crate) fn end_time(&self) -> f64 {
        self.hit_object.end_time()
    }

    pub(crate) fn column(&self, total_columns: f32) -> usize {
        let x_divisor = 512.0 / total_columns;

        (self.hit_object.pos.x / x_divisor)
            .floor()
            .min(total_columns - 1.0) as usize
    }
}
