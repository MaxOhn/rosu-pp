use self::{color::Color, rhythm::Rhythm, stamina::Stamina};

pub mod color;
pub mod rhythm;
pub mod stamina;

#[derive(Clone)]
pub struct TaikoSkills {
    pub rhythm: Rhythm,
    pub color: Color,
    pub stamina: Stamina,
}

impl TaikoSkills {
    pub fn new() -> Self {
        Self {
            rhythm: Rhythm::default(),
            color: Color::default(),
            stamina: Stamina::default(),
        }
    }
}
