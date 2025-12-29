// TODO: remove; suspicion checks should now be used to prevent calculation on
// malicious maps

pub use inner::*;

#[cfg(not(feature = "raw_strains"))]
mod inner {
    use std::{
        iter::{self, Copied},
        mem,
        slice::{self, Iter},
    };

    use crate::util::hint::{likely, unlikely};

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
        #[inline]
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                inner: Vec::with_capacity(capacity),
                len: 0,
                #[cfg(debug_assertions)]
                has_zero: false,
            }
        }

        /// Returns the number of elements.
        #[inline]
        pub const fn len(&self) -> usize {
            self.len
        }

        /// Appends an element to the back.
        #[inline]
        pub fn push(&mut self, value: f64) {
            if likely(value.to_bits() > 0 && value.is_sign_positive()) {
                // SAFETY: we just checked whether it's positive
                self.inner.push(unsafe { StrainsEntry::new_value(value) });
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
        #[inline]
        pub fn sort_desc(&mut self) {
            #[cfg(debug_assertions)]
            debug_assert!(!self.has_zero);

            self.inner.sort_by(|a, b| b.value().total_cmp(&a.value()));
        }

        /// Removes all zero entries
        #[inline]
        pub fn retain_non_zero(&mut self) {
            self.inner.retain(|e| likely(e.is_value()));

            #[cfg(debug_assertions)]
            {
                self.has_zero = false;
            }
        }

        /// Removes all zeros and sorts the remaining entries in descending order.
        #[inline]
        pub fn retain_non_zero_and_sort(&mut self) {
            self.retain_non_zero();
            self.sort_desc();
        }

        /// Removes all zeros, sorts the remaining entries in descending order, and
        /// returns an iterator over mutable references to the values.
        #[inline]
        pub fn sorted_non_zero_iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut f64> {
            self.retain_non_zero_and_sort();

            self.inner.iter_mut().map(StrainsEntry::as_value_mut)
        }

        /// Sum up all values.
        #[inline]
        pub fn sum(&self) -> f64 {
            self.inner
                .iter()
                .copied()
                .filter_map(StrainsEntry::try_as_value)
                .sum()
        }

        /// Returns an iterator over the [`StrainsVec`].
        #[inline]
        pub fn iter(&self) -> StrainsIter<'_> {
            StrainsIter::new(self)
        }

        /// Converts this [`StrainsVec`] into `Vec<f64>`.
        ///
        /// # Safety
        ///
        /// `self` may not include *any* zeros.
        pub unsafe fn transmute_into_vec(self) -> Vec<f64> {
            // SAFETY: `StrainsEntry` has the same properties as `f64`
            unsafe { mem::transmute::<Vec<StrainsEntry>, Vec<f64>>(self.inner) }
        }

        /// Allocates a new `Vec<f64>` to store all values, including zeros.
        pub fn into_vec(self) -> Vec<f64> {
            /// Copies the first `count` items of `slice` into `dst`.
            fn copy_slice(slice: &[StrainsEntry], count: usize, dst: &mut Vec<f64>) {
                if unlikely(count == 0) {
                    return;
                }

                let ptr = slice.as_ptr().cast();

                // SAFETY: `StrainsEntry` has the same properties as `f64`
                let slice = unsafe { slice::from_raw_parts(ptr, count) };
                dst.extend_from_slice(slice);
            }

            /// Drives the iterator until it finds a zero count. It then copies
            /// entries up to that and returns the zero count.
            #[inline]
            fn copy_non_zero(
                iter: &mut Iter<'_, StrainsEntry>,
                dst: &mut Vec<f64>,
            ) -> Option<usize> {
                let mut count = 0;
                let slice = iter.as_slice();

                for entry in iter {
                    if unlikely(entry.is_zero()) {
                        copy_slice(slice, count, dst);

                        return Some(entry.zero_count() as usize);
                    }

                    count += 1;
                }

                copy_slice(slice, count, dst);

                None
            }

            let mut vec = Vec::with_capacity(self.len);
            let mut iter = self.inner.iter();

            while let Some(zero_count) = copy_non_zero(&mut iter, &mut vec) {
                vec.extend(iter::repeat_n(0.0, zero_count));
            }

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

    impl Iterator for StrainsIter<'_> {
        type Item = f64;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let curr = self.curr.as_mut()?;

                if likely(curr.is_value()) {
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
        use super::likely;

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

            /// # Safety
            ///
            /// `value` must be positive, i.e. neither negative nor zero.
            #[inline]
            pub const unsafe fn new_value(value: f64) -> Self {
                Self { value }
            }

            #[inline]
            pub const fn new_zero() -> Self {
                Self {
                    zero_count: !Self::ZERO_COUNT_MASK + 1,
                }
            }

            #[inline]
            pub const fn is_zero(self) -> bool {
                unsafe { self.value.is_sign_negative() }
            }

            #[inline]
            pub const fn is_value(self) -> bool {
                !self.is_zero()
            }

            #[inline]
            pub const fn value(self) -> f64 {
                unsafe { self.value }
            }

            #[inline]
            pub const fn try_as_value(self) -> Option<f64> {
                if likely(self.is_value()) {
                    Some(self.value())
                } else {
                    None
                }
            }

            #[inline]
            pub const fn as_value_mut(&mut self) -> &mut f64 {
                unsafe { &mut self.value }
            }

            #[inline]
            pub const fn zero_count(self) -> u64 {
                unsafe { self.zero_count & Self::ZERO_COUNT_MASK }
            }

            #[inline]
            pub const fn incr_zero_count(&mut self) {
                unsafe {
                    self.zero_count += 1;
                }
            }

            #[inline]
            pub const fn decr_zero_count(&mut self) {
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
            fn expected(values in prop::collection::vec(prop::option::of(0.0..1_000.0), 0..1_000)) {
                let mut vec = StrainsVec::with_capacity(values.len());
                let mut raw = Vec::with_capacity(values.len());

                let mut additional_zeros = 0;
                let mut prev_zero = false;
                let mut sum = 0.0;

                for opt in values.iter().copied() {
                    if let Some(value) = opt.filter(|&value| value != 0.0) {
                        let value = f64::abs(value);

                        vec.push(value);
                        raw.push(value);
                        prev_zero = false;
                        sum += value;
                    } else {
                        vec.push(0.0);
                        raw.push(0.0);

                        if prev_zero {
                            additional_zeros += 1;
                        }

                        prev_zero = true;
                    }
                }

                assert_eq!(vec.len(), raw.len());
                assert_eq!(vec.inner.len(), raw.len() - additional_zeros);
                assert!(vec.sum().eq(sum));
                assert!(vec.iter().eq(raw.iter().copied()));
                assert_eq!(vec.clone().into_vec(), raw);

                vec.retain_non_zero_and_sort();
                raw.retain(|&n| n > 0.0);
                raw.sort_by(|a, b| b.total_cmp(a));

                assert_eq!(unsafe { vec.transmute_into_vec() }, raw);
            }
        }
    }
}

#[cfg(feature = "raw_strains")]
mod inner {
    use std::{
        iter::Copied,
        slice::{Iter, IterMut},
    };

    /// Plain wrapper around `Vec<f64>` because the `raw_strains` feature
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

        pub unsafe fn transmute_into_vec(self) -> Vec<f64> {
            self.inner
        }

        pub fn into_vec(self) -> Vec<f64> {
            self.inner
        }
    }
}
