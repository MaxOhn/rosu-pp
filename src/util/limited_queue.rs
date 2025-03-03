use std::ops::Index;

/// Efficient counterpart to osu!'s [`LimitedCapacityQueue`] i.e. an indexed
/// queue with limited capacity.
///
/// [`LimitedQueue`] will use an internal array as queue which is stored on
/// the stack. Hence, if `size_of<T>() * N` is very large, consider using a
/// different type since heap allocation might be favorable.
///
/// [`LimitedCapacityQueue`]: https://github.com/ppy/osu/blob/b49a1aab8ac6e16e48dffd03f55635cdc1771adf/osu.Game/Rulesets/Difficulty/Utils/LimitedCapacityQueue.cs
#[derive(Clone, Debug)]
pub struct LimitedQueue<T, const N: usize> {
    queue: [T; N],
    /// If the queue is not empty, `end` is the index of the last element.
    /// Otherwise, it has no meaning.
    end: usize,
    /// Amount of elements in the queue. This is equal to `end + 1` if the
    /// queue is not full, or `N` otherwise.
    len: usize,
}

impl<T, const N: usize> Default for LimitedQueue<T, N>
where
    T: Copy + Clone + Default,
{
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
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, const N: usize> LimitedQueue<T, N> {
    pub fn push(&mut self, elem: T) {
        self.end = (self.end + 1) % N;
        self.queue[self.end] = elem;
        self.len += usize::from(self.len < N);
    }

    #[cfg(test)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    #[cfg(test)]
    pub const fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.queue[self.end])
        }
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.is_full() {
            (&self.queue[self.end + 1..N], &self.queue[0..=self.end])
        } else {
            (&[], &self.queue[0..self.len])
        }
    }
}

impl<T, const N: usize> Index<usize> for LimitedQueue<T, N> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        let idx = (idx + usize::from(self.len == N) * (self.end + 1)) % N;

        &self.queue[idx]
    }
}

#[cfg(test)]
mod test {
    use std::cmp;

    use super::LimitedQueue;

    #[test]
    fn empty() {
        let queue = LimitedQueue::<u8, 4>::default();
        assert!(queue.is_empty());
        assert_eq!(queue.last(), None);
    }

    #[test]
    fn single_push() {
        let mut queue = LimitedQueue::<u8, 4>::default();
        let elem = 42;
        queue.push(elem);
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.last(), Some(&elem));
        assert_eq!(queue[0], elem);
    }

    #[test]
    fn overfull() {
        let mut queue = LimitedQueue::<u8, 4>::default();

        for i in 1..=5 {
            queue.push(i as u8);
            assert_eq!(cmp::min(i, 4), queue.len());
        }

        assert_eq!(queue.last(), Some(&5));
        assert_eq!(queue[0], 2);
        assert_eq!(queue[3], 5);
    }

    #[test]
    fn as_slices() {
        let mut queue = LimitedQueue::<u8, 3>::default();
        assert_eq!(queue.as_slices(), ([].as_slice(), [].as_slice()));

        // Start by filling the tail slice
        queue.push(1);
        assert_eq!(queue.as_slices(), ([].as_slice(), [1].as_slice()));
        queue.push(2);
        assert_eq!(queue.as_slices(), ([].as_slice(), [1, 2].as_slice()));
        queue.push(3);
        assert_eq!(queue.as_slices(), ([].as_slice(), [1, 2, 3].as_slice()));

        // The buffer is full and wraps around so now it uses the head slice
        queue.push(4);
        assert_eq!(queue.as_slices(), ([2, 3].as_slice(), [4].as_slice()));
        queue.push(5);
        assert_eq!(queue.as_slices(), ([3].as_slice(), [4, 5].as_slice()));
        queue.push(6);
        assert_eq!(queue.as_slices(), ([].as_slice(), [4, 5, 6].as_slice()));

        queue.push(7);
        assert_eq!(queue.as_slices(), ([5, 6].as_slice(), [7].as_slice()));
    }
}
