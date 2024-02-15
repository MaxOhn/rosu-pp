use std::cmp::Ordering;

use crate::model::hit_object::HitObject;

const QUICK_SORT_DEPTH_THRESHOLD: usize = 32;

/// osu!'s legacy sorting algorithm.
///
/// <https://github.com/ppy/osu/blob/e669e28dc9b6d79d82a36053e6a279de8dafddd1/osu.Game.Rulesets.Mania/MathUtils/LegacySortHelper.cs#L19>
pub fn sort(keys: &mut [HitObject]) {
    if keys.len() < 2 {
        return;
    }

    depth_limited_quick_sort(keys, 0, keys.len() - 1, QUICK_SORT_DEPTH_THRESHOLD);
}

fn depth_limited_quick_sort(
    keys: &mut [HitObject],
    mut left: usize,
    mut right: usize,
    mut depth_limit: usize,
) {
    loop {
        if depth_limit == 0 {
            super::heap_sort(keys, left, right, &cmp);

            return;
        }

        let mut i = left;
        let mut j = right;

        let mid = i + ((j - i) >> 1);

        super::swap_if_greater(keys, &cmp, i, mid);
        super::swap_if_greater(keys, &cmp, i, j);
        super::swap_if_greater(keys, &cmp, mid, j);

        loop {
            while keys[i] < keys[mid] {
                i += 1;
            }

            while keys[mid] < keys[j] {
                j -= 1;
            }

            match i.cmp(&j) {
                Ordering::Less => keys.swap(i, j),
                Ordering::Equal => {}
                Ordering::Greater => break,
            }

            i += 1;
            j = j.saturating_sub(1);

            if i > j {
                break;
            }
        }

        depth_limit -= 1;

        if j.saturating_sub(left) <= right - i {
            if left < j {
                depth_limited_quick_sort(keys, left, j, depth_limit);
            }

            left = i;
        } else {
            if i < right {
                depth_limited_quick_sort(keys, i, right, depth_limit);
            }

            right = j;
        }

        if left >= right {
            break;
        }
    }
}

fn cmp(a: &HitObject, b: &HitObject) -> Ordering {
    a.start_time.total_cmp(&b.start_time)
}
