pub trait FloatExt: Sized {
    /// `self == other`
    fn eq(self, other: Self) -> bool;

    /// `self != other`
    fn not_eq(self, other: Self) -> bool;
}

macro_rules! impl_float_ext {
    ( $ty:ty ) => {
        impl FloatExt for $ty {
            fn eq(self, other: Self) -> bool {
                (self - other).abs() < <$ty>::EPSILON
            }

            fn not_eq(self, other: Self) -> bool {
                (self - other).abs() >= <$ty>::EPSILON
            }
        }
    };
}

impl_float_ext!(f32);
impl_float_ext!(f64);
