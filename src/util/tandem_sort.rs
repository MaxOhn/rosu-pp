use std::cmp::Ordering;

///  Stores the sorted order for an initial list so that multiple
///  lists can be sorted based on that order.
pub(crate) struct TandemSorter {
    indices: Vec<usize>,
}

impl TandemSorter {
    /// Sort indices based on the given slice.
    /// Note that this does **not** sort the given slice.
    pub(crate) fn new<T>(slice: &[T], stable: bool) -> Self
    where
        T: PartialOrd,
    {
        let mut indices: Vec<_> = (0..).take(slice.len()).collect();

        let closure =
            |&i: &usize, &j: &usize| slice[i].partial_cmp(&slice[j]).unwrap_or(Ordering::Equal);

        if stable {
            indices.sort_by(closure);
        } else {
            indices.sort_unstable_by(closure);
        }

        Self { indices }
    }

    /// Sort the given slice based on the internal ordering.
    ///
    /// If you intend to sort another slice afterwards,
    /// don't forget to call [`Self::toggle_marks`] first.
    pub(crate) fn sort<T>(&mut self, slice: &mut [T]) {
        for i in 0..self.indices.len() {
            let i_idx = self.indices[i];

            if Self::idx_is_marked(i_idx) {
                continue;
            }

            let mut j = i;
            let mut j_idx = i_idx;

            // When we loop back to the first index, we stop
            while j_idx != i {
                self.indices[j] = Self::toggle_mark_idx(j_idx);
                slice.swap(j, j_idx);
                j = j_idx;
                j_idx = self.indices[j];
            }

            self.indices[j] = Self::toggle_mark_idx(j_idx);
        }
    }

    /// This method must be called inbetween sorting slices.
    pub(crate) fn toggle_marks(&mut self) {
        for idx in self.indices.iter_mut() {
            *idx = Self::toggle_mark_idx(*idx);
        }
    }

    #[inline(always)]
    fn idx_is_marked(idx: usize) -> bool {
        // Check if first bit is set
        idx.leading_zeros() == 0
    }

    #[inline(always)]
    fn toggle_mark_idx(idx: usize) -> usize {
        // Flip the first bit
        idx ^ !(usize::MAX >> 1)
    }
}

#[cfg(test)]
mod tests {
    use super::TandemSorter;

    #[test]
    fn sort() {
        let mut base = vec![9, 7, 8, 1, 4, 3, 5, 2];
        let mut sorter = TandemSorter::new(&base, false);

        sorter.sort(&mut base);
        assert_eq!(base, vec![1, 2, 3, 4, 5, 7, 8, 9]);

        sorter.toggle_marks();
        let mut other = vec!['h', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd'];
        sorter.sort(&mut other);
        assert_eq!(
            other,
            vec!['l', 'o', ' ', 'o', 'W', 'e', 'l', 'h', 'r', 'l', 'd']
        );
    }
}
