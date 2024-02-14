pub trait FloatExt: Sized {
    /// Workaround since rust rounds ties away from 0.0
    /// while C# rounds them to the nearest even integer.
    /// See github
    ///   - https://github.com/rust-lang/rust/issues/96710
    ///   - https://github.com/rust-lang/rust/pull/82273
    fn round_even(self) -> Self;

    /// `self == other`
    fn eq(self, other: Self) -> bool;

    /// `self != other`
    fn not_eq(self, other: Self) -> bool;
}

macro_rules! impl_float_ext {
    ( $ty:ty ) => {
        impl FloatExt for $ty {
            fn round_even(self) -> Self {
                if self.fract().abs().eq(0.5) {
                    2.0 * (self / 2.0).round()
                } else {
                    self.round()
                }
            }

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

#[cfg(test)]
mod tests {
    #[test]
    fn round_even() {
        let values = vec![
            (3.0, 3.0),
            (3.5, 4.0),
            (4.5, 4.0),
            (4.500001, 5.0),
            (3.499999, 3.0),
            (3.500001, 4.0),
            (100_000_000_002.5, 100_000_000_002.0),
            (-3.0, -3.0),
            (-3.5, -4.0),
            (-4.5, -4.0),
            (-4.500001, -5.0),
            (-3.499999, -3.0),
            (-3.500001, -4.0),
            (-100_000_000_002.5, -100_000_000_002.0),
        ];

        for (value, expected) in values {
            let rounded = <f32 as super::FloatExt>::round_even(value);

            assert!(
                (rounded - expected).abs() <= f32::EPSILON,
                "expected {expected} for {value}; got {rounded}"
            );
        }
    }
}
