use std::{
    iter::{self, Copied},
    slice,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct CompactVec {
    inner: Vec<Entry>,
    len: usize,
}

impl CompactVec {
    const ACCEPTABLE_DIFFERENCE: f64 = 1e-16;

    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, num: f64) {
        self.push_n(num, 1)
    }

    pub(crate) fn push_n(&mut self, num: f64, n: usize) {
        if let Some(last) = self
            .inner
            .last_mut()
            .filter(|entry| (entry.value - num).abs() <= Self::ACCEPTABLE_DIFFERENCE)
        {
            last.count += n;
        } else if n > 0 {
            self.inner.push(Entry::new(num, n))
        }

        self.len += n;
    }

    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(f64) -> bool,
    {
        self.inner.retain(|entry| f(entry.value))
    }

    pub(crate) fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    pub(crate) fn to_vec(&self) -> Vec<f64> {
        let mut nums = Vec::with_capacity(self.len);

        for entry in self.inner.iter() {
            nums.extend(iter::repeat(entry.value).take(entry.count));
        }

        nums
    }

    pub(crate) fn sum(&self) -> f64 {
        self.inner
            .iter()
            .fold(0.0, |sum, entry| sum + entry.value * entry.count as f64)
    }
}

#[derive(Copy, Clone, Debug)]
struct Entry {
    value: f64,
    count: usize,
}

impl Entry {
    fn new(value: f64, count: usize) -> Self {
        Self { value, count }
    }
}

pub(crate) struct Iter<'a> {
    iter: Copied<slice::Iter<'a, Entry>>,
    curr: Option<Entry>,
}

impl<'a> Iter<'a> {
    fn new(compact_zeros: &'a CompactVec) -> Self {
        let mut iter = compact_zeros.inner.iter().copied();

        Self {
            curr: iter.next(),
            iter,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let curr = self.curr.as_mut()?;

            if curr.count == 0 {
                self.curr = self.iter.next();
            } else {
                curr.count -= 1;

                return Some(curr.value);
            }
        }
    }
}
