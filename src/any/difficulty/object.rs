pub trait IDifficultyObject {
    type DifficultyObjects: IDifficultyObjects + ?Sized;

    fn idx(&self) -> usize;

    fn previous<'a>(
        &self,
        backwards_idx: usize,
        diff_objects: &'a Self::DifficultyObjects,
    ) -> Option<&'a <Self::DifficultyObjects as IDifficultyObjects>::DifficultyObject> {
        self.idx()
            .checked_sub(backwards_idx + 1)
            .and_then(|idx| diff_objects.get(idx))
    }

    fn next<'a, D>(&self, forwards_idx: usize, diff_objects: &'a [D]) -> Option<&'a D> {
        diff_objects.get(self.idx() + (forwards_idx + 1))
    }
}

pub trait IDifficultyObjects {
    type DifficultyObject: HasStartTime;

    fn get(&self, idx: usize) -> Option<&Self::DifficultyObject>;
}

impl<T: HasStartTime> IDifficultyObjects for [T] {
    type DifficultyObject = T;

    fn get(&self, idx: usize) -> Option<&Self::DifficultyObject> {
        self.get(idx)
    }
}

pub trait HasStartTime {
    fn start_time(&self) -> f64;
}
