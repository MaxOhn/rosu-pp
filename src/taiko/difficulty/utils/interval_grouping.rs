use crate::{
    taiko::difficulty::utils::has_interval::HasInterval,
    util::{
        float_ext::FloatExt,
        sync::{RefCount, Weak},
    },
};

pub struct IntervalGroupingUtils;

impl IntervalGroupingUtils {
    // * The margin of error when comparing intervals for grouping, or snapping intervals to a common value.
    pub const MARGIN_OF_ERROR: f64 = 5.0;
}

pub const fn group_by_interval<T: HasInterval>(
    objects: &[RefCount<T>],
) -> GroupedByIntervalIter<'_, T> {
    GroupedByIntervalIter::new(objects)
}

pub struct GroupedByIntervalIter<'a, T> {
    objects: &'a [RefCount<T>],
    i: usize,
}

impl<'a, T> GroupedByIntervalIter<'a, T> {
    const fn new(objects: &'a [RefCount<T>]) -> Self {
        Self { objects, i: 0 }
    }
}

impl<T: HasInterval> Iterator for GroupedByIntervalIter<'_, T> {
    type Item = Vec<Weak<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.objects.len() {
            Some(self.create_next_group())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = usize::from(!self.objects.is_empty());
        let max = self.objects.len() - self.i;

        (min, Some(max))
    }
}

impl<T: HasInterval> GroupedByIntervalIter<'_, T> {
    fn create_next_group(&mut self) -> Vec<Weak<T>> {
        const MARGIN_OF_ERROR: f64 = IntervalGroupingUtils::MARGIN_OF_ERROR;

        let &mut Self { objects, ref mut i } = self;

        // * This never compares the first two elements in the group.
        // * This sounds wrong but is apparently "as intended" (https://github.com/ppy/osu/pull/31636#discussion_r1942673329)
        let mut grouped_objects = vec![objects[*i].downgrade()];

        *i += 1;

        while *i < objects.len() - 1 {
            if !(objects[*i]
                .get()
                .interval()
                .almost_eq(objects[*i + 1].get().interval(), MARGIN_OF_ERROR))
            {
                // * When an interval change occurs, include the object with the differing interval in the case it increased
                // * See https://github.com/ppy/osu/pull/31636#discussion_r1942368372 for rationale.
                if objects[*i + 1].get().interval() > objects[*i].get().interval() + MARGIN_OF_ERROR
                {
                    grouped_objects.push(objects[*i].downgrade());
                    *i += 1;
                }

                return grouped_objects;
            }

            // * No interval change occurred
            grouped_objects.push(objects[*i].downgrade());

            *i += 1;
        }

        // * Check if the last two objects in the object form a "flat" rhythm pattern within the specified margin of error.
        // * If true, add the current object to the group and increment the index to process the next object.
        if objects.len() > 2
            && *i < objects.len()
            && objects[objects.len() - 1]
                .get()
                .interval()
                .almost_eq(objects[objects.len() - 2].get().interval(), MARGIN_OF_ERROR)
        {
            grouped_objects.push(objects[*i].downgrade());
            *i += 1;
        }

        grouped_objects
    }
}
