use std::{
    iter::{Cycle, Skip, Take},
    ops::Index,
    slice::Iter,
};

/// Efficient counterpart to osu!'s [`LimitedCapacityQueue`]
/// i.e. an indexed queue with limited capacity.
///
/// [`LimitedQueue`] will use an internal array as queue which
/// is stored on the stack. Hence, if the size is very large,
/// e.g. `size_of<T>() * N`, consider using a different type
/// since heap allocation might be favorable.
///
/// [`LimitedCapacityQueue`]: https://github.com/ppy/osu/blob/b49a1aab8ac6e16e48dffd03f55635cdc1771adf/osu.Game/Rulesets/Difficulty/Utils/LimitedCapacityQueue.cs
#[derive(Clone, Debug)]
pub(crate) struct LimitedQueue<T, const N: usize> {
    queue: [T; N],
    /// If the queue is not empty, `end` is the index of the last element.
    /// Otherwise, it has no meaning.
    end: usize,
    /// Amount of elements in the queue. This is equal to `end + 1`
    /// if the queue is not full, or `N` otherwise.
    len: usize,
}

impl<T, const N: usize> Default for LimitedQueue<T, N>
where
    T: Copy + Clone + Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            end: N - 1,
            queue: [T::default(); N],
            len: 0,
        }
    }
}

impl<T, const N: usize> LimitedQueue<T, N>
where
    T: Copy + Clone + Default,
{
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl<T, const N: usize> LimitedQueue<T, N> {
    pub(crate) fn push(&mut self, elem: T) {
        self.end = (self.end + 1) % N;
        self.queue[self.end] = elem;
        self.len += (self.len < N) as usize;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.queue[self.end])
        }
    }

    pub(crate) fn iter(&self) -> LimitedQueueIter<'_, T> {
        self.queue
            .iter()
            .cycle()
            .skip((self.len == N) as usize * (self.end + 1))
            .take(self.len)
    }
}

pub(crate) type LimitedQueueIter<'a, T> = Take<Skip<Cycle<Iter<'a, T>>>>;

impl<T, const N: usize> Index<usize> for LimitedQueue<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        assert!(
            idx < self.len,
            "index out of bounds: the len is {} but the index is {idx}",
            self.len
        );

        let idx = (idx + (self.len == N) as usize * (self.end + 1)) % N;

        &self.queue[idx]
    }
}

#[cfg(test)]
mod test {
    use super::LimitedQueue;

    #[test]
    fn empty() {
        let queue = LimitedQueue::<u8, 4>::default();
        assert!(queue.is_empty());
        assert_eq!(queue.last(), None);
        assert_eq!(queue.iter().count(), 0);
    }

    #[test]
    fn single_push() {
        let mut queue = LimitedQueue::<u8, 4>::default();
        let elem = 42;
        queue.push(elem);
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.last(), Some(&elem));
        assert!(queue.iter().eq(vec![elem].iter()));
        assert_eq!(queue[0], elem);
    }

    #[test]
    fn overfull() {
        let mut queue = LimitedQueue::<u8, 4>::default();

        for i in 1..=5 {
            queue.push(i as u8);
            assert_eq!(i.min(4), queue.len());
        }

        assert_eq!(queue.last(), Some(&5));
        assert!(queue.iter().eq(&[2, 3, 4, 5]));
        assert_eq!(queue[0], 2);
        assert_eq!(queue[3], 5);
    }
}
