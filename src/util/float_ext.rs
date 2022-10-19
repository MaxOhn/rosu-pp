pub(crate) trait FloatExt: Sized {
    // Workaround since rust rounds ties away from 0.0
    // while C# rounds them to the nearest even integer.
    // See github
    //   - https://github.com/rust-lang/rust/issues/96710
    //   - https://github.com/rust-lang/rust/pull/82273
    fn round_even(self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn round_even(self) -> Self {
        if (0.5 - self.fract().abs()).abs() <= f32::EPSILON {
            2.0 * (self / 2.0).round()
        } else {
            self.round()
        }
    }
}

impl FloatExt for f64 {
    #[inline]
    fn round_even(self) -> Self {
        if (0.5 - self.fract().abs()).abs() <= f64::EPSILON {
            2.0 * (self / 2.0).round()
        } else {
            self.round()
        }
    }
}

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
                "expected {} for {}; got {}",
                expected,
                value,
                rounded,
            );
        }
    }
}
