use std::{iter::Copied, slice::Iter};

use self::entry::CompactZerosEntry;

#[derive(Clone)]
pub struct CompactZerosVec {
    inner: Vec<CompactZerosEntry>,
    len: usize,
    #[cfg(debug_assertions)]
    // Ensures that methods are used correctly
    has_zero: bool,
}

impl CompactZerosVec {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            len: 0,
            #[cfg(debug_assertions)]
            has_zero: false,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: f64) {
        if value.to_bits() > 0 {
            self.inner.push(CompactZerosEntry::new_value(value));
        } else if let Some(last) = self.inner.last_mut().filter(|e| e.is_zero()) {
            last.incr_zero_count();
        } else {
            self.inner.push(CompactZerosEntry::new_zero());

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
        self.inner.retain(CompactZerosEntry::is_value);

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
        debug_assert!(!self.has_zero);

        self.inner.iter().copied().map(CompactZerosEntry::value)
    }

    /// Same as [`CompactZerosVec::retain_non_zero_and_sort`] followed by
    /// [`CompactZerosVec::iter`] but the resulting iterator is faster
    /// because it doesn't need to check whether entries are zero.
    pub fn sorted_non_zero_iter(&mut self) -> impl ExactSizeIterator<Item = f64> + '_ {
        self.retain_non_zero_and_sort();

        self.non_zero_iter()
    }

    pub fn sorted_non_zero_iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut f64> {
        self.retain_non_zero_and_sort();

        self.inner.iter_mut().map(CompactZerosEntry::as_value_mut)
    }

    pub fn sum(&self) -> f64 {
        self.inner
            .iter()
            .copied()
            .filter(CompactZerosEntry::is_value)
            .fold(0.0, |sum, e| sum + e.value())
    }

    pub fn iter(&self) -> CompactZerosIter<'_> {
        CompactZerosIter::new(self)
    }

    pub fn to_vec(&self) -> Vec<f64> {
        let mut vec = Vec::with_capacity(self.len);
        vec.extend(self.iter());

        vec
    }
}

pub struct CompactZerosIter<'a> {
    inner: Copied<Iter<'a, CompactZerosEntry>>,
    curr: Option<CompactZerosEntry>,
    len: usize,
}

impl<'a> CompactZerosIter<'a> {
    pub fn new(vec: &'a CompactZerosVec) -> Self {
        let mut inner = vec.inner.iter().copied();
        let curr = inner.next();

        Self {
            inner,
            curr,
            len: vec.len,
        }
    }
}

impl<'a> Iterator for CompactZerosIter<'a> {
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
            } else {
                self.curr = self.inner.next();
            }
        }
    }
}

impl ExactSizeIterator for CompactZerosIter<'_> {
    fn len(&self) -> usize {
        self.len
    }
}

mod entry {

    #[derive(Copy, Clone)]
    pub union CompactZerosEntry {
        value: f64,
        zero_count: u64,
    }

    impl CompactZerosEntry {
        const ZERO_COUNT_MASK: u64 = u64::MAX >> 1;

        pub const fn new_value(value: f64) -> Self {
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

            // SAFETY: Union has the same layout as a `f64`.
            unsafe { &mut *(self as *mut CompactZerosEntry).cast::<f64>() }
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
