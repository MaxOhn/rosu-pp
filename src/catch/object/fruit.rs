use crate::catch::attributes::ObjectCountBuilder;

pub struct Fruit {
    pub x_offset: f32,
}

impl Fruit {
    pub fn new(count: &mut ObjectCountBuilder) -> Self {
        count.record_fruit();

        Self { x_offset: 0.0 }
    }
}
