/// Abstract type to define hitsounds.
#[allow(missing_docs)]
pub trait HitSound {
    const HITSOUND_WHISTLE: u8 = 1 << 1;
    const HITSOUND_FINISH: u8 = 1 << 2;
    const HITSOUND_CLAP: u8 = 1 << 3;

    fn normal(self) -> bool;
    fn whistle(self) -> bool;
    fn finish(self) -> bool;
    fn clap(self) -> bool;
}

impl HitSound for u8 {
    fn normal(self) -> bool {
        self == 0
    }

    fn whistle(self) -> bool {
        self & Self::HITSOUND_WHISTLE > 0
    }

    fn finish(self) -> bool {
        self & Self::HITSOUND_FINISH > 0
    }

    fn clap(self) -> bool {
        self & Self::HITSOUND_CLAP > 0
    }
}
