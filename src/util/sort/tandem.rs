use std::cmp::Ordering;

/// Stores the sorted order for an initial list so that multiple
/// lists can be sorted based on that order.
pub struct TandemSorter {
    indices: Box<[usize]>,
    should_reset: bool,
}

impl TandemSorter {
    /// Sort indices based on the given slice.
    ///
    /// Note that this does **not** sort the given slice.
    pub fn new<T>(slice: &[T], cmp: fn(&T, &T) -> Ordering, stable: bool) -> Self {
        let mut indices: Box<[usize]> = (0..slice.len()).collect();
        let sort_by = |&i: &usize, &j: &usize| cmp(&slice[i], &slice[j]);

        if stable {
            indices.sort_by(sort_by);
        } else {
            // When sorting integers, the order of elements with equal values
            // does not matter so we can use rust's sort instead of C#'s.
            indices.sort_unstable_by(sort_by);
        }

        Self {
            indices,
            should_reset: false,
        }
    }

    /// Sort the given slice based on the internal ordering.
    pub fn sort<T>(&mut self, slice: &mut [T]) {
        if self.should_reset {
            self.toggle_marks();
            self.should_reset = false;
        }

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

        self.should_reset = true;
    }

    fn toggle_marks(&mut self) {
        for idx in self.indices.iter_mut() {
            *idx = Self::toggle_mark_idx(*idx);
        }
    }

    const fn idx_is_marked(idx: usize) -> bool {
        // Check if first bit is set
        idx.leading_zeros() == 0
    }

    const fn toggle_mark_idx(idx: usize) -> usize {
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
        let mut other = "hello World".chars().collect::<Vec<_>>();
        let mut sorter = TandemSorter::new(&base, u8::cmp, false);

        sorter.sort(&mut base);
        assert_eq!(base, vec![1, 2, 3, 4, 5, 7, 8, 9]);

        sorter.sort(&mut other);
        assert_eq!(other, "lo oWelhrld".chars().collect::<Vec<_>>());
    }
}
