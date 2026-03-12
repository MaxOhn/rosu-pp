use std::cmp::Ordering;

pub use self::{
    csharp::sort_unstable_by as csharp, osu_legacy::sort_unstable_by as osu_legacy,
    tandem::TandemSorter,
};

mod csharp;
mod osu_legacy;
mod tandem;

fn heap_sort<T, F>(keys: &mut [T], lo: usize, hi: usize, comparer: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    let n = hi - lo + 1;

    for i in (1..=n / 2).rev() {
        down_heap(keys, i, n, lo, comparer);
    }

    for i in (2..=n).rev() {
        swap(keys, lo, lo + i - 1);
        down_heap(keys, 1, i - 1, lo, comparer);
    }
}

fn down_heap<T, F>(keys: &mut [T], mut i: usize, n: usize, lo: usize, comparer: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    while i <= n / 2 {
        let mut child = 2 * i;

        if child < n && comparer(&keys[lo + child - 1], &keys[lo + child]).is_lt() {
            child += 1;
        }

        // is_ge == !is_lt
        if comparer(&keys[lo + i - 1], &keys[lo + child - 1]).is_ge() {
            break;
        }

        keys.swap(lo + i - 1, lo + child - 1);
        i = child;
    }
}

fn swap_if_greater<T, F>(keys: &mut [T], comparer: &F, a: usize, b: usize)
where
    F: Fn(&T, &T) -> Ordering,
{
    if a != b && comparer(&keys[a], &keys[b]).is_gt() {
        keys.swap(a, b);
    }
}

const fn swap<T>(keys: &mut [T], i: usize, j: usize) {
    if i != j {
        keys.swap(i, j);
    }
}
