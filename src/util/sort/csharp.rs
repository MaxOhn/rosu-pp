use std::cmp::Ordering;

/// C#'s unstable sorting algorithm.
///
/// <https://referencesource.microsoft.com/#mscorlib/system/collections/generic/list.cs,fc1bc8c930f6c93c,references>
pub fn sort<T, F>(keys: &mut [T], cmp: F)
where
    F: Fn(&T, &T) -> Ordering,
{
    introspective_sort(keys, 0, keys.len(), &cmp);
}

fn introspective_sort<T, F>(keys: &mut [T], left: usize, len: usize, cmp: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    if len >= 2 {
        intro_sort(keys, left, len + left - 1, 2 * keys.len().ilog2(), cmp);
    }
}

fn intro_sort<T, F>(keys: &mut [T], lo: usize, mut hi: usize, mut depth_limit: u32, cmp: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    const INTRO_SORT_SIZE_THRESHOLD: usize = 16;

    while hi > lo {
        let partition_size = hi - lo + 1;

        if partition_size <= INTRO_SORT_SIZE_THRESHOLD {
            match partition_size {
                1 => {}
                2 => super::swap_if_greater(keys, cmp, lo, hi),
                3 => {
                    super::swap_if_greater(keys, cmp, lo, hi - 1);
                    super::swap_if_greater(keys, cmp, lo, hi);
                    super::swap_if_greater(keys, cmp, hi - 1, hi);
                }
                _ => insertion_sort(keys, lo, hi, cmp),
            }

            break;
        }

        if depth_limit == 0 {
            super::heap_sort(keys, lo, hi, cmp);

            break;
        }

        depth_limit -= 1;
        let p = pick_pivot_and_partition(keys, lo, hi, cmp);
        intro_sort(keys, p + 1, hi, depth_limit, cmp);
        hi = p - 1;
    }
}

fn pick_pivot_and_partition<T, F>(keys: &mut [T], lo: usize, hi: usize, cmp: &F) -> usize
where
    F: Fn(&T, &T) -> Ordering,
{
    let mid = lo + (hi - lo) / 2;
    super::swap_if_greater(keys, cmp, lo, mid);
    super::swap_if_greater(keys, cmp, lo, hi);
    super::swap_if_greater(keys, cmp, mid, hi);
    super::swap(keys, mid, hi - 1);
    let mut left = lo;
    let mut right = hi - 1;
    let pivot_idx = right;

    while left < right {
        while {
            left += 1;

            cmp(&keys[left], &keys[pivot_idx]).is_lt()
        } {}

        while {
            right -= 1;

            cmp(&keys[pivot_idx], &keys[right]).is_lt()
        } {}

        if left >= right {
            break;
        }

        super::swap(keys, left, right);
    }

    super::swap(keys, left, hi - 1);

    left
}

fn insertion_sort<T, F>(keys: &mut [T], lo: usize, hi: usize, cmp: F)
where
    F: Fn(&T, &T) -> Ordering,
{
    for i in lo..hi {
        let t = &keys[i + 1];

        let shift = keys[lo..=i]
            .iter()
            .rev()
            .take_while(|curr| cmp(t, curr).is_lt())
            .count();

        keys[i + 1 - shift..=i + 1].rotate_right(1);
    }
}
