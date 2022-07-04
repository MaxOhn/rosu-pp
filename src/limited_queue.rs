use std::cmp::Ordering;
use std::iter::{Cycle, Skip, Take};
use std::ops::Index;
use std::slice::Iter;

// TODO: make generic over const size
#[derive(Clone, Debug)]
pub(crate) struct LimitedQueue<T> {
    queue: Vec<T>,
    start: usize,
    end: usize,
}

impl<T> LimitedQueue<T> {
    #[inline]
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            end: capacity - 1,
            start: 0,
            queue: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub(crate) fn push(&mut self, elem: T) {
        let capacity = self.queue.capacity();
        self.end = (self.end + 1) % capacity;

        if self.queue.len() == capacity {
            self.start = (self.start + 1) % capacity;
            self.queue[self.end] = elem;
        } else {
            self.queue.push(elem);
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.queue.len()
    }

    #[inline]
    pub(crate) fn last(&self) -> Option<&T> {
        self.queue.get(self.end)
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.start = 0;
        self.end = self.queue.capacity() - 1;
        self.queue.clear();
    }

    #[inline]
    pub(crate) fn full(&self) -> bool {
        self.queue.len() == self.queue.capacity()
    }

    #[inline]
    pub(crate) fn iter(&self) -> LimitedQueueIter<'_, T> {
        self.queue
            .iter()
            .cycle()
            .skip(self.start)
            .take(self.queue.len())
    }
}

impl<T: PartialOrd> LimitedQueue<T> {
    pub(crate) fn min(&self) -> Option<&T> {
        self.queue
            .iter()
            .reduce(|min, next| match min.partial_cmp(next) {
                Some(Ordering::Less) => min,
                Some(Ordering::Equal) => min,
                Some(Ordering::Greater) => next,
                None => min,
            })
    }
}

impl<T> Index<usize> for LimitedQueue<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        &self.queue[(self.start + idx) % self.queue.capacity()]
    }
}

pub(crate) type LimitedQueueIter<'a, T> = Take<Skip<Cycle<Iter<'a, T>>>>;
