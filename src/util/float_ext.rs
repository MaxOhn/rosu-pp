pub trait FloatExt: Sized {
    const EPS: Self;

    /// `self == other`
    fn eq(self, other: Self) -> bool {
        self.almost_eq(other, Self::EPS)
    }

    /// `self ~= other` (within `acceptable_difference`)
    fn almost_eq(self, other: Self, acceptable_difference: Self) -> bool;

    /// `self != other`
    fn not_eq(self, other: Self) -> bool;

    /// Performs a linear interpolation between two values based on the given weight.
    fn lerp(value1: Self, value2: Self, amount: Self) -> Self;
}

macro_rules! impl_float_ext {
    ( $ty:ty ) => {
        impl FloatExt for $ty {
            const EPS: Self = <$ty>::EPSILON;

            fn almost_eq(self, other: Self, acceptable_difference: Self) -> bool {
                (self - other).abs() <= acceptable_difference
            }

            fn not_eq(self, other: Self) -> bool {
                (self - other).abs() >= Self::EPS
            }

            // <https://github.com/dotnet/runtime/blob/1d1bf92fcf43aa6981804dc53c5174445069c9e4/src/libraries/System.Private.CoreLib/src/System/Double.cs#L841>
            fn lerp(value1: Self, value2: Self, amount: Self) -> Self {
                (value1 * (1.0 - amount)) + (value2 * amount)
            }
        }
    };
}

impl_float_ext!(f32);
impl_float_ext!(f64);
