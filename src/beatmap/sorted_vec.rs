use std::{
    cmp::Ordering,
    convert::identity,
    fmt::{Debug, Formatter, Result as FmtResult},
    ops::Deref,
};

use super::{control_points::EffectPoint, DifficultyPoint, TimingPoint};

/// A [`Vec`] whose elements are guaranteed to be in order based on the given comparator.
#[derive(Clone)]
pub struct SortedVec<T> {
    inner: Vec<T>,
    cmp: fn(&T, &T) -> Ordering,
}

impl<T> SortedVec<T> {
    /// If the value is found then [`Result::Ok`] is returned, containing the
    /// index of the matching element. If there are multiple matches, then any
    /// one of the matches could be returned.
    /// If the value is not found then [`Result::Err`] is returned, containing
    /// the index where a matching element could be inserted while maintaining
    /// sorted order.
    pub fn find(&self, value: &T) -> Result<usize, usize> {
        self.inner
            .binary_search_by(|probe| (self.cmp)(probe, value))
    }

    pub(crate) fn push(&mut self, value: T) {
        let idx = self.find(&value).map_or_else(identity, identity);

        self.inner.insert(idx, value);
    }

    pub(crate) fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.inner.dedup_by(|a, b| key(a) == key(b))
    }
}

impl<T> Deref for SortedVec<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Debug> Debug for SortedVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.inner, f)
    }
}

impl Default for SortedVec<TimingPoint> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            cmp: |a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal),
        }
    }
}

impl Default for SortedVec<DifficultyPoint> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            cmp: |a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal),
        }
    }
}

impl Default for SortedVec<EffectPoint> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            cmp: |a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal),
        }
    }
}

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
