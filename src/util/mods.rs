pub trait Mods: Copy {
    fn nf(self) -> bool;
    fn ez(self) -> bool;
    fn td(self) -> bool;
    fn hd(self) -> bool;
    fn hr(self) -> bool;
    fn dt(self) -> bool;
    fn rx(self) -> bool;
    fn ht(self) -> bool;
    fn fl(self) -> bool;
    fn so(self) -> bool;

    fn clock_rate(self) -> f64 {
        if self.dt() {
            1.5
        } else if self.ht() {
            0.75
        } else {
            1.0
        }
    }

    fn od_ar_hp_multiplier(self) -> f64 {
        if self.hr() {
            1.4
        } else if self.ez() {
            0.5
        } else {
            1.0
        }
    }
}

macro_rules! impl_mods_fn {
    ( $fn_name:ident, $bits:expr ) => {
        fn $fn_name(self) -> bool {
            self & ($bits) != 0
        }
    };
}

impl Mods for u32 {
    impl_mods_fn!(nf, 1 << 0);
    impl_mods_fn!(ez, 1 << 1);
    impl_mods_fn!(td, 1 << 2);
    impl_mods_fn!(hd, 1 << 3);
    impl_mods_fn!(hr, 1 << 4);
    impl_mods_fn!(dt, 1 << 6);
    impl_mods_fn!(rx, 1 << 7);
    impl_mods_fn!(ht, 1 << 8);
    impl_mods_fn!(fl, 1 << 10);
    impl_mods_fn!(so, 1 << 12);
}
