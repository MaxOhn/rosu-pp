pub trait Mods: Copy {
    const NF: u32 = 1 << 0;
    const EZ: u32 = 1 << 1;
    const TD: u32 = 1 << 2;
    const HD: u32 = 1 << 3;
    const HR: u32 = 1 << 4;
    const DT: u32 = 1 << 6;
    const HT: u32 = 1 << 8;
    const NC: u32 = Self::DT | (1 << 9);
    const FL: u32 = 1 << 10;
    const SO: u32 = 1 << 12;

    fn change_speed(self) -> bool;
    fn change_map(self) -> bool;
    fn speed(self) -> f32;
    fn od_ar_hp_multiplier(self) -> f32;
    fn nf(self) -> bool;
    fn ez(self) -> bool;
    fn td(self) -> bool;
    fn hd(self) -> bool;
    fn hr(self) -> bool;
    fn dt(self) -> bool;
    fn ht(self) -> bool;
    fn nc(self) -> bool;
    fn fl(self) -> bool;
    fn so(self) -> bool;
}

impl Mods for u32 {
    #[inline]
    fn change_speed(self) -> bool {
        self & (Self::HT | Self::DT) > 0
    }

    #[inline]
    fn change_map(self) -> bool {
        self & (Self::HT | Self::DT | Self::HR | Self::EZ) > 0
    }

    #[inline]
    fn speed(self) -> f32 {
        if self & Self::DT > 0 {
            1.5
        } else if self & Self::HT > 0 {
            0.75
        } else {
            1.0
        }
    }

    #[inline]
    fn od_ar_hp_multiplier(self) -> f32 {
        if self & Self::HR > 0 {
            1.4
        } else if self & Self::EZ > 0 {
            0.5
        } else {
            1.0
        }
    }

    #[inline]
    fn nf(self) -> bool {
        self & Self::NF > 0
    }

    #[inline]
    fn ez(self) -> bool {
        self & Self::EZ > 0
    }

    #[inline]
    fn td(self) -> bool {
        self & Self::TD > 0
    }

    #[inline]
    fn hd(self) -> bool {
        self & Self::HD > 0
    }

    #[inline]
    fn hr(self) -> bool {
        self & Self::HR > 0
    }

    #[inline]
    fn dt(self) -> bool {
        self & Self::DT > 0
    }

    #[inline]
    fn ht(self) -> bool {
        self & Self::HT > 0
    }

    #[inline]
    fn nc(self) -> bool {
        self & Self::NC > 0
    }

    #[inline]
    fn fl(self) -> bool {
        self & Self::FL > 0
    }

    #[inline]
    fn so(self) -> bool {
        self & Self::SO > 0
    }
}
