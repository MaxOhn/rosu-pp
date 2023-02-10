use std::{
    iter::{self, Copied},
    slice,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct CompactZerosVec {
    inner: Vec<Number>,
    non_zero_count: usize,
}

impl CompactZerosVec {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, num: f64) {
        if num.abs() > f64::EPSILON {
            self.inner.push(Number::NonZero(num));
            self.non_zero_count += 1;
        } else {
            self.push_n_zeros(1);
        }
    }

    pub(crate) fn push_n_zeros(&mut self, n: usize) {
        match self.inner.last_mut() {
            Some(Number::NonZero(_)) | None => self.inner.push(Number::Zeros(n)),
            Some(Number::Zeros(zeros)) => *zeros += n,
        }
    }

    pub(crate) fn to_non_zeros(&self) -> Vec<f64> {
        let mut non_zeros = Vec::with_capacity(self.non_zero_count);

        let iter = self.inner.iter().filter_map(|num| match num {
            Number::NonZero(n) => Some(*n),
            Number::Zeros(_) => None,
        });

        non_zeros.extend(iter);

        non_zeros
    }

    pub(crate) fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    pub(crate) fn to_vec(&self) -> Vec<f64> {
        let mut nums = Vec::with_capacity(self.inner.len() * 2);

        for &num in self.inner.iter() {
            match num {
                Number::NonZero(num) => nums.push(num),
                Number::Zeros(zeros) => nums.extend(iter::repeat(0.0).take(zeros)),
            }
        }

        nums
    }
}

#[derive(Copy, Clone, Debug)]
enum Number {
    NonZero(f64),
    Zeros(usize),
}

pub(crate) struct Iter<'a> {
    iter: Copied<slice::Iter<'a, Number>>,
    curr: Option<Number>,
}

impl<'a> Iter<'a> {
    fn new(compact_zeros: &'a CompactZerosVec) -> Self {
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
        match self.curr.as_mut()? {
            Number::NonZero(n) => {
                let n = *n;
                self.curr = self.iter.next();

                Some(n)
            }
            Number::Zeros(zeros @ 1..) => {
                *zeros -= 1;

                Some(0.0)
            }
            Number::Zeros(_) => {
                self.curr = self.iter.next();

                self.next()
            }
        }
    }
}
