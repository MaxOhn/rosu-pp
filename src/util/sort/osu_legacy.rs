use std::cmp::Ordering;

const QUICK_SORT_DEPTH_THRESHOLD: usize = 32;

/// osu!'s legacy sorting algorithm.
///
/// <https://github.com/ppy/osu/blob/e669e28dc9b6d79d82a36053e6a279de8dafddd1/osu.Game.Rulesets.Mania/MathUtils/LegacySortHelper.cs#L19>
pub fn sort_unstable_by<T, F>(keys: &mut [T], comparer: F)
where
    F: Fn(&T, &T) -> Ordering,
{
    if keys.len() < 2 {
        return;
    }

    depth_limited_quick_sort(
        keys,
        0,
        keys.len() - 1,
        &comparer,
        QUICK_SORT_DEPTH_THRESHOLD,
    );
}

fn depth_limited_quick_sort<T, F>(
    keys: &mut [T],
    mut left: usize,
    mut right: usize,
    comparer: &F,
    mut depth_limit: usize,
) where
    F: Fn(&T, &T) -> Ordering,
{
    loop {
        if depth_limit == 0 {
            super::heap_sort(keys, left, right, &comparer);

            return;
        }

        let mut i = left;
        let mut j = right;

        let middle = i + ((j - i) >> 1);

        super::swap_if_greater(keys, &comparer, i, middle);
        super::swap_if_greater(keys, &comparer, i, j);
        super::swap_if_greater(keys, &comparer, middle, j);

        loop {
            while comparer(&keys[i], &keys[middle]).is_lt() {
                i += 1;
            }

            while comparer(&keys[middle], &keys[j]).is_lt() {
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
                depth_limited_quick_sort(keys, left, j, comparer, depth_limit);
            }

            left = i;
        } else {
            if i < right {
                depth_limited_quick_sort(keys, i, right, comparer, depth_limit);
            }

            right = j;
        }

        if left >= right {
            break;
        }
    }
}
