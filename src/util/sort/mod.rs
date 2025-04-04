use std::cmp::Ordering;

pub use self::{csharp::sort as csharp, osu_legacy::sort as osu_legacy, tandem::TandemSorter};

mod csharp;
mod osu_legacy;
mod tandem;

fn heap_sort<T, F>(keys: &mut [T], lo: usize, hi: usize, cmp: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    let n = hi - lo + 1;

    for i in (1..=n / 2).rev() {
        down_heap(keys, i, n, lo, cmp);
    }

    for i in (2..=n).rev() {
        swap(keys, lo, lo + i - 1);
        down_heap(keys, 1, i - 1, lo, cmp);
    }
}

fn down_heap<T, F>(keys: &mut [T], mut i: usize, n: usize, lo: usize, cmp: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    while i <= n / 2 {
        let mut child = 2 * i;

        if child < n && cmp(&keys[lo + child - 1], &keys[lo + child]).is_lt() {
            child += 1;
        }

        if cmp(&keys[lo + i - 1], &keys[lo + child - 1]).is_ge() {
            break;
        }

        keys.swap(lo + i - 1, lo + child - 1);
        i = child;
    }
}

fn swap_if_greater<T, F>(keys: &mut [T], cmp: &F, a: usize, b: usize)
where
    F: Fn(&T, &T) -> Ordering,
{
    if a != b && cmp(&keys[a], &keys[b]).is_gt() {
        keys.swap(a, b);
    }
}

const fn swap<T>(keys: &mut [T], i: usize, j: usize) {
    if i != j {
        keys.swap(i, j);
    }
}
