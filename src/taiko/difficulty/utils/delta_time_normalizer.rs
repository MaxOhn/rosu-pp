use std::collections::HashMap;

use crate::{taiko::difficulty::object::TaikoDifficultyObject, util::sync::Weak};

pub struct DeltaTimeNormalizer;

impl DeltaTimeNormalizer {
    pub fn normalize(
        hit_objects: &[Weak<TaikoDifficultyObject>],
        margin_of_error: f64,
    ) -> HashMap<Weak<TaikoDifficultyObject>, f64> {
        let delta_times = {
            let mut distinct_and_ordered: Vec<f64> = Vec::new();

            for h in hit_objects.iter() {
                let Some(h) = h.upgrade() else { continue };
                let delta_time = h.get().delta_time;

                if let Err(idx) =
                    distinct_and_ordered.binary_search_by(|probe| probe.total_cmp(&delta_time))
                {
                    distinct_and_ordered.insert(idx, delta_time);
                }
            }

            distinct_and_ordered
        };

        let mut sets = Vec::new();
        let mut delta_times_iter = delta_times.into_iter();

        if let Some(value) = delta_times_iter.next() {
            let mut curr = vec![value];

            for value in delta_times_iter {
                // * Add to the current group if within margin of error
                if f64::abs(value - curr[0]) <= margin_of_error {
                    curr.push(value);

                    continue;
                }

                sets.push(curr);

                // * Otherwise begin a new group
                curr = vec![value];
            }

            sets.push(curr);
        }

        // * Compute median for each group
        let mut median_lookup = HashMap::new();

        for mut set in sets {
            set.sort_by(f64::total_cmp);
            let mid = set.len() / 2;

            let median = if set.len() % 2 == 1 {
                set[mid]
            } else {
                (set[mid - 1] + set[mid]) / 2.0
            };

            for v in set {
                median_lookup.insert(v.to_bits(), median);
            }
        }

        let mut output = HashMap::with_capacity(hit_objects.len());

        let iter = hit_objects.iter().filter_map(|h| {
            let delta_time = h.upgrade()?.get().delta_time;

            let value = median_lookup
                .get(&delta_time.to_bits())
                .copied()
                .unwrap_or(delta_time);

            Some((Weak::clone(h), value))
        });

        output.extend(iter);

        output
    }
}
