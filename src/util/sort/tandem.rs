use std::cmp::Ordering;

/// Stores the sorted order for an initial list so that multiple
/// lists can be sorted based on that order.
pub struct TandemSorter {
    indices: Box<[usize]>,
    should_reset: bool,
}

macro_rules! new_fn {
    ( $fn:ident: $sort:expr ) => {
        /// Sort indices based on the given slice.
        ///
        /// Note that this does **not** sort the given slice.
        pub fn $fn<T>(slice: &[T], cmp: fn(&T, &T) -> Ordering) -> Self {
            let mut indices: Box<[usize]> = (0..slice.len()).collect();
            $sort(&mut indices, |&i, &j| cmp(&slice[i], &slice[j]));

            Self {
                indices,
                should_reset: false,
            }
        }
    };
}

impl TandemSorter {
    new_fn!(new_stable: <[_]>::sort_by);
    new_fn!(new_unstable: super::csharp);

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

    /// Unsort the given slice based on the internal ordering.
    pub fn unsort<T>(mut self, slice: &mut [T]) {
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

            while j != j_idx {
                self.indices[j] = Self::toggle_mark_idx(j_idx);
                self.indices.swap(j, j_idx);
                slice.swap(j, j_idx);
                j = self.indices[j];
                j_idx = self.indices[j];
            }

            self.indices[j] = Self::toggle_mark_idx(j_idx);
        }
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
        let mut sorter = TandemSorter::new_stable(&base, u8::cmp);

        sorter.sort(&mut base);
        assert_eq!(base, vec![1, 2, 3, 4, 5, 7, 8, 9]);

        sorter.sort(&mut other);
        assert_eq!(other.iter().collect::<String>(), "lo oWelhrld");
    }
}
