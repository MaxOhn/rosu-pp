pub trait IDifficultyObject: Sized {
    fn idx(&self) -> usize;

    fn previous<'a, D>(&self, backwards_idx: usize, diff_objects: &'a [D]) -> Option<&'a D> {
        self.idx()
            .checked_sub(backwards_idx + 1)
            .and_then(|idx| diff_objects.get(idx))
    }

    fn next<'a, D>(&self, forwards_idx: usize, diff_objects: &'a [D]) -> Option<&'a D> {
        diff_objects.get(self.idx() + (forwards_idx + 1))
    }
}
