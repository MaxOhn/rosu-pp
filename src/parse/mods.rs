macro_rules! impl_mods {
    ($func_name:ident, $const_name:ident) => {
        #[inline]
        fn $func_name(self) -> bool {
            self & Self::$const_name > 0
        }
    };
}

pub trait Mods: Copy {
    const NF: u32 = 1 << 0;
    const EZ: u32 = 1 << 1;
    const TD: u32 = 1 << 2;
    const HD: u32 = 1 << 3;
    const HR: u32 = 1 << 4;
    const DT: u32 = 1 << 6;
    const HT: u32 = 1 << 8;
    const FL: u32 = 1 << 10;
    const SO: u32 = 1 << 12;

    const K1: u32 = 1 << 26;
    const K2: u32 = 1 << 27;
    const K3: u32 = 1 << 28;
    const K4: u32 = 1 << 15;
    const K5: u32 = 1 << 16;
    const K6: u32 = 1 << 17;
    const K7: u32 = 1 << 18;
    const K8: u32 = 1 << 19;
    const K9: u32 = 1 << 24;

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
    fn fl(self) -> bool;
    fn so(self) -> bool;

    fn key1(self) -> bool;
    fn key2(self) -> bool;
    fn key3(self) -> bool;
    fn key4(self) -> bool;
    fn key5(self) -> bool;
    fn key6(self) -> bool;
    fn key7(self) -> bool;
    fn key8(self) -> bool;
    fn key9(self) -> bool;
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

    impl_mods!(nf, NF);
    impl_mods!(ez, EZ);
    impl_mods!(td, TD);
    impl_mods!(hd, HD);
    impl_mods!(hr, HR);
    impl_mods!(dt, DT);
    impl_mods!(ht, HT);
    impl_mods!(fl, FL);
    impl_mods!(so, SO);

    impl_mods!(key1, K1);
    impl_mods!(key2, K2);
    impl_mods!(key3, K3);
    impl_mods!(key4, K4);
    impl_mods!(key5, K5);
    impl_mods!(key6, K6);
    impl_mods!(key7, K7);
    impl_mods!(key8, K8);
    impl_mods!(key9, K9);
}
