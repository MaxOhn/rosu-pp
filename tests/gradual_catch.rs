#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::{
    catch::{CatchGradualDifficulty, CatchGradualPerformance, CatchScoreState},
    Beatmap, CatchPP, CatchStars,
};

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = CatchGradualDifficulty::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn iter_end_eq_regular() {
    let map = test_map!(Catch);
    let regular = CatchStars::new(&map).calculate();

    let iter_end = CatchGradualDifficulty::new(&map, 0)
        .last()
        .expect("empty iter");

    assert_eq!(regular, iter_end);
}

#[test]
fn correct_empty() {
    let map = test_map!(Catch);
    let mut gradual = CatchGradualPerformance::new(&map, 0);
    let state = CatchScoreState::default();

    let first_attrs = gradual.nth(state.clone(), usize::MAX);

    assert!(first_attrs.is_some());
    assert!(gradual.next(state).is_none());
}

#[test]
fn next_and_next_n() {
    let map = test_map!(Catch);
    let state = CatchScoreState::default();

    let mut gradual1 = CatchGradualPerformance::new(&map, 0);
    let mut gradual2 = CatchGradualPerformance::new(&map, 0);

    for _ in 0..20 {
        let _ = gradual1.next(state.clone());
        let _ = gradual2.next(state.clone());
    }

    let n = 80;

    for _ in 1..n {
        let _ = gradual1.next(state.clone());
    }

    let state = CatchScoreState {
        max_combo: 101,
        n_fruits: 99,
        n_droplets: 2,
        n_tiny_droplets: 68,
        n_tiny_droplet_misses: 0,
        n_misses: 0,
    };

    let next = gradual1.next(state.clone());
    let next_n = gradual2.nth(state, n - 1);

    assert_eq!(next_n, next);
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Catch);
    let regular = CatchPP::new(&map).calculate();

    let mut gradual = CatchGradualPerformance::new(&map, 0);

    let state = CatchScoreState {
        max_combo: 730,
        n_fruits: 728,
        n_droplets: 2,
        n_tiny_droplets: 291,
        n_tiny_droplet_misses: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.nth(state, usize::MAX).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_eq_regular_passed() {
    let map = test_map!(Catch);
    let n = 100;

    let regular = CatchPP::new(&map).passed_objects(n).calculate();
    let mut gradual = CatchGradualPerformance::new(&map, 0);

    let state = CatchScoreState {
        max_combo: 101,
        n_fruits: 99,
        n_droplets: 2,
        n_tiny_droplets: 68,
        n_tiny_droplet_misses: 0,
        n_misses: 0,
    };

    let gradual = gradual.nth(state, n - 1).unwrap();

    assert_eq!(regular, gradual);
}
