pub use inner::*;

#[cfg(feature = "compact_strains")]
mod inner {
    use std::{iter::Copied, slice::Iter};

    use self::entry::StrainsEntry;

    /// A specialized `Vec<f64>` where all entries must be non-negative.
    ///
    /// It is compact in the sense that zeros are not stored directly but instead
    /// as amount of times they appear consecutively.
    ///
    /// For cases with few consecutive zeros, this type generally reduces
    /// performance slightly. However, for edge cases like `/b/3739922` the length
    /// of the list is massively reduced, preventing out-of-memory issues.
    #[derive(Clone)]
    pub struct StrainsVec {
        inner: Vec<StrainsEntry>,
        len: usize,
        #[cfg(debug_assertions)]
        // Ensures that methods are used correctly
        has_zero: bool,
    }

    impl StrainsVec {
        /// Constructs a new, empty [`StrainsVec`] with at least the specified
        /// capacity.
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                inner: Vec::with_capacity(capacity),
                len: 0,
                #[cfg(debug_assertions)]
                has_zero: false,
            }
        }

        /// Returns the number of elements.
        pub const fn len(&self) -> usize {
            self.len
        }

        /// Appends an element to the back.
        pub fn push(&mut self, value: f64) {
            if value.to_bits() > 0 {
                self.inner.push(StrainsEntry::new_value(value));
            } else if let Some(last) = self.inner.last_mut().filter(|e| e.is_zero()) {
                last.incr_zero_count();
            } else {
                self.inner.push(StrainsEntry::new_zero());

                #[cfg(debug_assertions)]
                {
                    self.has_zero = true;
                }
            }

            self.len += 1;
        }

        /// Sorts the entries in descending order.
        pub fn sort_desc(&mut self) {
            #[cfg(debug_assertions)]
            debug_assert!(!self.has_zero);

            self.inner.sort_by(|a, b| b.value().total_cmp(&a.value()));
        }

        /// Removes all zero entries
        pub fn retain_non_zero(&mut self) {
            self.inner.retain(StrainsEntry::is_value);

            #[cfg(debug_assertions)]
            {
                self.has_zero = false;
            }
        }

        /// Removes all zeros and sorts the remaining entries in descending order.
        pub fn retain_non_zero_and_sort(&mut self) {
            self.retain_non_zero();
            self.sort_desc();
        }

        /// Iterator over the raw entries, assuming that there are no zeros.
        ///
        /// Panics if there are zeros.
        pub fn non_zero_iter(&self) -> impl ExactSizeIterator<Item = f64> + '_ {
            #[cfg(debug_assertions)]
            debug_assert!(!self.has_zero);

            self.inner.iter().copied().map(StrainsEntry::value)
        }

        /// Same as [`StrainsVec::retain_non_zero_and_sort`] followed by
        /// [`StrainsVec::iter`] but the resulting iterator is faster
        /// because it doesn't need to check whether entries are zero.
        pub fn sorted_non_zero_iter(&mut self) -> impl ExactSizeIterator<Item = f64> + '_ {
            self.retain_non_zero_and_sort();

            self.non_zero_iter()
        }

        /// Removes all zeros, sorts the remaining entries in descending order, and
        /// returns an iterator over mutable references to the values.
        pub fn sorted_non_zero_iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut f64> {
            self.retain_non_zero_and_sort();

            self.inner.iter_mut().map(StrainsEntry::as_value_mut)
        }

        /// Sum up all values.
        pub fn sum(&self) -> f64 {
            self.inner
                .iter()
                .copied()
                .filter(StrainsEntry::is_value)
                .fold(0.0, |sum, e| sum + e.value())
        }

        /// Returns an iterator over the [`StrainsVec`].
        pub fn iter(&self) -> StrainsIter<'_> {
            StrainsIter::new(self)
        }

        /// Allocates a new `Vec<f64>` to store all values, including zeros.
        pub fn into_vec(self) -> Vec<f64> {
            let mut vec = Vec::with_capacity(self.len);
            vec.extend(self.iter());

            vec
        }
    }

    pub struct StrainsIter<'a> {
        inner: Copied<Iter<'a, StrainsEntry>>,
        curr: Option<StrainsEntry>,
        len: usize,
    }

    impl<'a> StrainsIter<'a> {
        pub fn new(vec: &'a StrainsVec) -> Self {
            let mut inner = vec.inner.iter().copied();
            let curr = inner.next();

            Self {
                inner,
                curr,
                len: vec.len,
            }
        }
    }

    impl<'a> Iterator for StrainsIter<'a> {
        type Item = f64;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let curr = self.curr.as_mut()?;

                if curr.is_value() {
                    let value = curr.value();
                    self.curr = self.inner.next();
                    self.len -= 1;

                    return Some(value);
                } else if curr.zero_count() > 0 {
                    curr.decr_zero_count();
                    self.len -= 1;

                    return Some(0.0);
                }

                self.curr = self.inner.next();
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = self.len();

            (len, Some(len))
        }
    }

    impl ExactSizeIterator for StrainsIter<'_> {
        fn len(&self) -> usize {
            self.len
        }
    }

    /// Private module to hide internal fields.
    mod entry {
        /// Either a positive `f64` or an amount of consecutive `0.0`.
        ///
        /// If the first bit is not set, i.e. the sign bit of a `f64` indicates
        /// that it's positive, the union represents that `f64`. Otherwise, the
        /// first bit is ignored and the union represents a `u64`.
        #[derive(Copy, Clone)]
        pub union StrainsEntry {
            value: f64,
            zero_count: u64,
        }

        impl StrainsEntry {
            const ZERO_COUNT_MASK: u64 = u64::MAX >> 1;

            pub fn new_value(value: f64) -> Self {
                debug_assert!(
                    value.is_sign_positive(),
                    "attempted to create negative strain entry, please report as a bug"
                );

                Self { value }
            }

            pub const fn new_zero() -> Self {
                Self {
                    zero_count: !Self::ZERO_COUNT_MASK + 1,
                }
            }

            pub fn is_zero(self) -> bool {
                unsafe { self.value.is_sign_negative() }
            }

            // Requiring `self` as a reference improves ergonomics for passing this
            // method as argument to higher-order functions.
            #[allow(clippy::trivially_copy_pass_by_ref)]
            pub fn is_value(&self) -> bool {
                !self.is_zero()
            }

            pub fn value(self) -> f64 {
                debug_assert!(self.is_value());

                unsafe { self.value }
            }

            pub fn as_value_mut(&mut self) -> &mut f64 {
                debug_assert!(self.is_value());

                unsafe { &mut self.value }
            }

            pub fn zero_count(self) -> u64 {
                debug_assert!(self.is_zero());

                unsafe { self.zero_count & Self::ZERO_COUNT_MASK }
            }

            pub fn incr_zero_count(&mut self) {
                debug_assert!(self.is_zero());

                unsafe {
                    self.zero_count += 1;
                }
            }

            pub fn decr_zero_count(&mut self) {
                debug_assert!(self.is_zero());

                unsafe {
                    self.zero_count -= 1;
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use proptest::prelude::*;

        use crate::util::float_ext::FloatExt;

        use super::*;

        proptest! {
            #[test]
            fn expected(mut values in prop::collection::vec(prop::option::of(0.0..1_000.0), 0..1_000)) {
                let mut vec = StrainsVec::with_capacity(values.len());

                let mut additional_zeros = 0;
                let mut prev_zero = false;
                let mut sum = 0.0;

                for opt in values.iter().copied() {
                    if let Some(value) = opt {
                        vec.push(value);
                        prev_zero = false;
                        sum += value;
                    } else {
                        vec.push(0.0);

                        if prev_zero {
                            additional_zeros += 1;
                        }

                        prev_zero = true;
                    }
                }

                assert_eq!(vec.len(), values.len());
                assert_eq!(vec.inner.len(), values.len() - additional_zeros);
                assert!(vec.sum().eq(sum));
                assert!(vec.iter().eq(values.iter().copied().map(|opt| opt.unwrap_or(0.0))));

                values.retain(Option::is_some);

                values.sort_by(|a, b| {
                    let (Some(a), Some(b)) = (a, b) else { unreachable!() };

                    b.total_cmp(a)
                });

                assert!(vec.sorted_non_zero_iter().eq(values.into_iter().flatten()));
            }
        }
    }
}

#[cfg(not(feature = "compact_strains"))]
mod inner {
    use std::{
        iter::Copied,
        slice::{Iter, IterMut},
    };

    /// Plain wrapper around `Vec<f64>` because the `compact_strains` feature
    /// is disabled.
    #[derive(Clone)]
    pub struct StrainsVec {
        inner: Vec<f64>,
    }

    impl StrainsVec {
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                inner: Vec::with_capacity(capacity),
            }
        }

        pub fn len(&self) -> usize {
            self.inner.len()
        }

        pub fn push(&mut self, value: f64) {
            self.inner.push(value);
        }

        pub fn sort_desc(&mut self) {
            self.inner.sort_by(|a, b| b.total_cmp(a));
        }

        pub fn retain_non_zero(&mut self) {
            self.inner.retain(|&a| a > 0.0);
        }

        pub fn retain_non_zero_and_sort(&mut self) {
            self.retain_non_zero();
            self.sort_desc();
        }

        pub fn non_zero_iter(&self) -> Copied<Iter<'_, f64>> {
            self.inner.iter().copied()
        }

        pub fn sorted_non_zero_iter(&mut self) -> Copied<Iter<'_, f64>> {
            self.retain_non_zero_and_sort();

            self.non_zero_iter()
        }

        pub fn sorted_non_zero_iter_mut(&mut self) -> IterMut<'_, f64> {
            self.retain_non_zero_and_sort();

            self.inner.iter_mut()
        }

        pub fn sum(&self) -> f64 {
            self.inner.iter().copied().sum()
        }

        pub fn iter(&self) -> Copied<Iter<'_, f64>> {
            self.inner.iter().copied()
        }

        pub fn into_vec(self) -> Vec<f64> {
            self.inner
        }
    }
}
