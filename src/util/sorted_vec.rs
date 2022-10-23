use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Result as FmtResult},
    ops::{Deref, Index},
    slice::SliceIndex,
};

use crate::beatmap::{DifficultyPoint, EffectPoint, TimingPoint};

/// A [`Vec`] whose elements are guaranteed to be in order based on the given comparator.
#[derive(Clone)]
pub struct SortedVec<T> {
    inner: Vec<T>,
    cmp: fn(&T, &T) -> Ordering,
}

impl<T> SortedVec<T> {
    /// Same as [`slice::binary_search_by`] with the internal compare function
    #[inline]
    pub fn find(&self, value: &T) -> Result<usize, usize> {
        self.inner
            .binary_search_by(|probe| (self.cmp)(probe, value))
    }

    /// Extracts the inner [`Vec`].
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    /// Push a new value into the sorted list.
    /// If there is already an element that matches the new value,
    /// the old element will be replaced.
    pub(crate) fn push(&mut self, value: T) {
        match self.find(&value) {
            Ok(i) => self.inner[i] = value,
            Err(i) if i == self.inner.len() => self.inner.push(value),
            Err(i) => self.inner.insert(i, value),
        }
    }
}

impl<T> Deref for SortedVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        <Vec<T> as Deref>::deref(&self.inner)
    }
}

impl<T, I> Index<I> for SortedVec<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        <Vec<T> as Index<I>>::index(&self.inner, index)
    }
}

impl<T: Debug> Debug for SortedVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        <Vec<T> as Debug>::fmt(&self.inner, f)
    }
}

impl<T> Default for SortedVec<T>
where
    T: Ord,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: Vec::default(),
            cmp: <T as Ord>::cmp,
        }
    }
}

macro_rules! impl_default_control_point {
    ( $( $ty:ident ),* ) => {
        $(
            impl Default for SortedVec<$ty> {
                #[inline]
                fn default() -> Self {
                    Self {
                        inner: Vec::new(),
                        cmp: |a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal),
                    }
                }
            }

            impl SortedVec<$ty> {
                #[allow(unused)]
                pub(crate) fn with_capacity(capacity: usize) -> Self {
                    Self {
                        inner: Vec::with_capacity(capacity),
                        cmp: |a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal),
                    }
                }
            }
        )*
    }
}

impl_default_control_point!(TimingPoint, DifficultyPoint, EffectPoint);

impl SortedVec<DifficultyPoint> {
    pub(crate) fn push_if_not_redundant(&mut self, value: DifficultyPoint) {
        let is_redundant = match self.find(&value).map_err(|idx| idx.checked_sub(1)) {
            Ok(idx) | Err(Some(idx)) => value.is_redundant(&self[idx]),
            Err(None) => value.is_redundant(&DifficultyPoint::default()),
        };

        if !is_redundant {
            self.push(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::beatmap::DifficultyPoint;

    use super::SortedVec;

    #[test]
    fn sorts_on_push() {
        let mut v = SortedVec {
            inner: Vec::new(),
            cmp: <i32 as Ord>::cmp,
        };

        v.push(42);
        v.push(13);
        v.push(20);
        v.push(0);

        assert_eq!(&v[..], &[0_i32, 13, 20, 42]);
    }

    #[test]
    fn no_push_if_redundant() {
        let mut v = SortedVec::default();

        v.push(DifficultyPoint::default());
        assert_eq!(v.len(), 1);

        v.push_if_not_redundant(DifficultyPoint::default());
        assert_eq!(v.len(), 1);
    }
}
