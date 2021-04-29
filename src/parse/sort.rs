use super::HitObject;

use std::cmp::Ordering;

const QUICK_SORT_DEPTH_THRESHOLD: usize = 32;

/// Algorithm from https://github.com/ppy/osu/blob/master/osu.Game.Rulesets.Mania/MathUtils/LegacySortHelper.cs#L21
pub(crate) fn legacy_sort(keys: &mut [HitObject]) {
    if keys.is_empty() {
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
            heap_sort(keys, left, right);

            return;
        }

        let mut i = left;
        let mut j = right;

        let mid = i + ((j - i) >> 1);

        if keys[i] > keys[mid] {
            keys.swap(i, mid);
        }

        if keys[i] > keys[j] {
            keys.swap(i, j);
        }

        if keys[mid] > keys[j] {
            keys.swap(mid, j);
        }

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

fn heap_sort(keys: &mut [HitObject], lo: usize, hi: usize) {
    let n = hi - lo + 1;

    for i in (1..=n / 2).rev() {
        down_heap(keys, i, n, lo);
    }

    for i in (2..=n).rev() {
        keys.swap(lo, lo + i - 1);
        down_heap(keys, 1, i - 1, lo);
    }
}

fn down_heap(keys: &mut [HitObject], mut i: usize, n: usize, lo: usize) {
    while i <= n / 2 {
        let mut child = 2 * i;

        if child < n && keys[lo + child - 1] < keys[lo + child] {
            child += 1;
        }

        if keys[lo + i - 1] >= keys[lo + child - 1] {
            break;
        }

        keys.swap(lo + i - 1, lo + child - 1);
        i = child;
    }
}
