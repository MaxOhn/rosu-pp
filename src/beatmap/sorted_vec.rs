use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Result as FmtResult},
    ops::{Deref, Index},
    slice::SliceIndex,
};

use crate::beatmap::{DifficultyPoint, EffectPoint, TimingPoint};

/// A [`Vec`] whose elements are guaranteed to be unique and in order.
#[derive(Clone)]
pub struct SortedVec<T> {
    inner: Vec<T>,
}

impl<T> SortedVec<T> {
    /// Constructs a new, empty `SortedVec<T>`.
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Constructs a new, empty `SortedVec<T>` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Extracts the inner [`Vec`].
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    /// Extracts a slice containing the entire sorted vector.
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    /// Returns a mutable reference to the underlying `Vec`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the items stay in order.
    pub unsafe fn as_inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.inner
    }

    /// Removes the last element and returns it, or `None` if the vec is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all items `i` for which `f(&i)` returns `false`.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }
}

impl<T: Sortable> SortedVec<T> {
    /// Same as [`slice::binary_search_by`] with the function
    /// [`<T as Sortable>::cmp`](Sortable::cmp).
    #[inline]
    pub fn find(&self, value: &T) -> Result<usize, usize> {
        self.inner
            .binary_search_by(|probe| <T as Sortable>::cmp(probe, value))
    }

    /// Push a new value into the sorted list based on [`<T as Sortable>::push`](Sortable::push).
    pub fn push(&mut self, value: T) {
        <T as Sortable>::push(value, self)
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

impl<T> Default for SortedVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sortable> From<Vec<T>> for SortedVec<T> {
    fn from(mut v: Vec<T>) -> Self {
        v.sort_by(<T as Sortable>::cmp);
        v.dedup_by(|a, b| {
            <T as Sortable>::cmp(a, b) == Ordering::Equal || <T as Sortable>::is_redundant(b, a)
        });

        Self { inner: v }
    }
}

impl<T: Sortable> FromIterator<T> for SortedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(Vec::from_iter(iter))
    }
}

impl<T: Sortable> Extend<T> for SortedVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

/// Trait for types that can be sorted in a [`SortedVec`].
pub trait Sortable: Sized {
    /// An [`Ordering`] between `self` and `other`.
    fn cmp(&self, other: &Self) -> Ordering;

    /// Indicates whether `self` and `_existing` are identical.
    #[inline]
    fn is_redundant(&self, _existing: &Self) -> bool {
        false
    }

    /// Pushes a value into the [`SortedVec`].
    #[inline]
    fn push(self, sorted_vec: &mut SortedVec<Self>) {
        match sorted_vec.find(&self) {
            Ok(i) => sorted_vec.inner[i] = self,
            Err(i) if i == sorted_vec.len() => sorted_vec.inner.push(self),
            Err(i) => sorted_vec.inner.insert(i, self),
        }
    }
}

impl Sortable for TimingPoint {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Sortable for DifficultyPoint {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }

    #[inline]
    fn is_redundant(&self, existing: &Self) -> bool {
        self.is_redundant(existing)
    }

    fn push(self, sorted_vec: &mut SortedVec<Self>) {
        enum Action {
            Insert(usize),
            Replace(usize),
            Push,
            Skip,
        }

        let action = match sorted_vec.find(&self).map_err(|idx| idx.checked_sub(1)) {
            Ok(i) | Err(Some(i)) if self.is_redundant(&sorted_vec[i]) => Action::Skip,
            Ok(i) => Action::Replace(i),
            Err(Some(i)) if i == sorted_vec.len() - 1 => Action::Push,
            Err(Some(i)) => Action::Insert(i),
            Err(None) if self.is_redundant(&Self::default()) => Action::Skip,
            Err(None) => Action::Insert(0),
        };

        match action {
            Action::Insert(i) => sorted_vec.inner.insert(i, self),
            Action::Replace(i) => sorted_vec.inner[i] = self,
            Action::Push => sorted_vec.inner.push(self),
            Action::Skip => {}
        }
    }
}

impl Sortable for EffectPoint {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl<T: Ord> Sortable for T {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        <Self as Ord>::cmp(self, other)
    }
}

#[cfg(test)]
mod tests {
    use crate::beatmap::DifficultyPoint;

    use super::SortedVec;

    #[test]
    fn sorts_on_push() {
        let mut v = SortedVec::with_capacity(4);

        v.push(42);
        v.push(13);
        v.push(20);
        v.push(0);

        assert_eq!(v.as_slice(), &[0_i32, 13, 20, 42]);
    }

    #[test]
    fn no_push_if_redundant() {
        let mut v = SortedVec::default();

        v.push(DifficultyPoint::default());
        assert_eq!(v.len(), 0);

        v.push(DifficultyPoint::new(1.0, 2.0, 3.0));
        assert_eq!(v.len(), 1);

        v.push(DifficultyPoint::new(2.0, 2.0, 3.0));
        v.push(DifficultyPoint::default());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn from_iter() {
        let base = vec![
            DifficultyPoint {
                time: 5.0,
                slider_vel: 10.0,
                bpm_mult: 1.0,
                generate_ticks: true,
            },
            DifficultyPoint {
                time: 3.0,
                slider_vel: 20.0,
                bpm_mult: 2.0,
                generate_ticks: false,
            },
            DifficultyPoint {
                time: 6.0,
                slider_vel: 10.0,
                bpm_mult: 3.0,
                generate_ticks: true,
            },
            DifficultyPoint {
                time: 10.0,
                slider_vel: 15.0,
                bpm_mult: 4.0,
                generate_ticks: true,
            },
        ];

        let sorted = SortedVec::from_iter(base);

        let v: Vec<_> = sorted
            .into_inner()
            .into_iter()
            .map(|tp| tp.bpm_mult)
            .collect();

        assert_eq!(v, vec![2.0, 1.0, 4.0]);
    }
}
