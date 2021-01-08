use parse::{HitObject, HitSound};

pub(crate) trait Rim {
    fn is_rim(&self) -> bool;
}

impl Rim for HitObject {
    #[inline]
    fn is_rim(&self) -> bool {
        self.sound.clap() || self.sound.whistle()
    }
}

impl Rim for u8 {
    #[inline]
    fn is_rim(&self) -> bool {
        self.clap() || self.whistle()
    }
}
