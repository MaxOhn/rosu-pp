use crate::catch::attributes::CatchDifficultyAttributesBuilder;

pub struct Fruit {
    pub x_offset: f32,
}

impl Fruit {
    pub fn new(attrs: &mut CatchDifficultyAttributesBuilder) -> Self {
        attrs.inc_fruits();

        Self { x_offset: 0.0 }
    }
}
