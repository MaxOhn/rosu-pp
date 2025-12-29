#[allow(
    unused,
    reason = "false positive (also 'expect' doesn't seem to work so we use 'allow'"
)]
pub use self::{mods::*, paths::*};

/// Paths to .osu files
mod paths {
    pub const OSU: &str = "./resources/2785319.osu";
    pub const TAIKO: &str = "./resources/1028484.osu";
    pub const CATCH: &str = "./resources/2118524.osu";
    pub const MANIA: &str = "./resources/1638954.osu";
}

/// Bit values for mods
mod mods {
    #![allow(
        unused,
        reason = "false positive (also 'expect' doesn't seem to work so we use 'allow'"
    )]

    pub const NM: u32 = 0;
    pub const NF: u32 = 1 << 0;
    pub const EZ: u32 = 1 << 1;
    pub const TD: u32 = 1 << 2;
    pub const HD: u32 = 1 << 3;
    pub const HR: u32 = 1 << 4;
    pub const DT: u32 = 1 << 6;
    pub const HT: u32 = 1 << 8;
    pub const FL: u32 = 1 << 10;
    pub const SO: u32 = 1 << 12;
}

#[track_caller]
pub fn assert_eq_float<F: Float>(a: F, b: F) {
    assert!((a - b).less_than_eps(), "{a} != {b}")
}

#[track_caller]
#[allow(
    unused,
    reason = "false positive (also 'expect' doesn't seem to work so we use 'allow'"
)]
pub fn assert_eq_option<F: Float>(a: Option<F>, b: Option<F>) {
    match (a, b) {
        (Some(a), Some(b)) => assert!((a - b).less_than_eps(), "{a} != {b}"),
        (None, None) => {}
        (None, Some(b)) => panic!("None != Some({b})"),
        (Some(a), None) => panic!("Some({a}) != None"),
    }
}

/// Trait to provide flexibility in the `assert_eq_float` function.
pub trait Float:
    Copy + std::fmt::Display + std::ops::Sub<Output = Self> + PartialOrd + Sized
{
    const EPSILON: Self;

    fn abs(self) -> Self;

    fn less_than_eps(self) -> bool {
        self.abs() < Self::EPSILON
    }
}

macro_rules! impl_float {
    ( $( $ty:ty )* ) => {
        $(
            impl Float for $ty {
                const EPSILON: Self = Self::EPSILON;

                fn abs(self) -> Self {
                    self.abs()
                }
            }
        )*
    }
}

impl_float!(f32 f64);

/// Trait to compare two instances and panic if they are not equal.
#[allow(
    unused,
    reason = "false positive (also 'expect' doesn't seem to work so we use 'allow'"
)]
pub trait AssertEq {
    fn assert_eq(&self, expected: &Self);
}
