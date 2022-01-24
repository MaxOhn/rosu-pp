use crate::parse::HitSound;

pub(crate) trait Rim {
    fn is_rim(&self) -> bool;
}

impl Rim for u8 {
    #[inline]
    fn is_rim(&self) -> bool {
        self.clap() || self.whistle()
    }
}
