#![cfg(not(any(feature = "async_tokio", feature = "async_std")))]

use rosu_pp::{
    catch::{CatchGradualDifficultyAttributes, CatchGradualPerformanceAttributes, CatchScoreState},
    Beatmap, CatchPP, CatchStars,
};

use crate::common::Catch;

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = CatchGradualDifficultyAttributes::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn iter_end_eq_regular() {
    let map = test_map!(Catch);
    let regular = CatchStars::new(&map).calculate();

    let iter_end = CatchGradualDifficultyAttributes::new(&map, 0)
        .last()
        .expect("empty iter");

    assert_eq!(regular, iter_end);
}

#[test]
fn correct_empty() {
    let map = test_map!(Catch);
    let mut gradual = CatchGradualPerformanceAttributes::new(&map, 0);
    let state = CatchScoreState::default();

    let first_attrs = gradual.process_next_n_objects(state.clone(), usize::MAX);

    assert!(first_attrs.is_some());
    assert!(gradual.process_next_object(state).is_none());
}

#[test]
fn next_and_next_n() {
    let map = test_map!(Catch);
    let state = CatchScoreState::default();

    let mut gradual1 = CatchGradualPerformanceAttributes::new(&map, 0);
    let mut gradual2 = CatchGradualPerformanceAttributes::new(&map, 0);

    for _ in 0..20 {
        let _ = gradual1.process_next_object(state.clone());
        let _ = gradual2.process_next_object(state.clone());
    }

    let n = 80;

    for _ in 1..n {
        let _ = gradual1.process_next_object(state.clone());
    }

    let state = CatchScoreState {
        max_combo: 101,
        n_fruits: 99,
        n_droplets: 2,
        n_tiny_droplets: 68,
        n_tiny_droplet_misses: 0,
        n_misses: 0,
    };

    let next = gradual1.process_next_object(state.clone());
    let next_n = gradual2.process_next_n_objects(state, n);

    assert_eq!(next_n, next);
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Catch);
    let regular = CatchPP::new(&map).calculate();

    let mut gradual = CatchGradualPerformanceAttributes::new(&map, 0);

    let state = CatchScoreState {
        max_combo: 730,
        n_fruits: 728,
        n_droplets: 2,
        n_tiny_droplets: 291,
        n_tiny_droplet_misses: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.process_next_n_objects(state, usize::MAX).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_eq_regular_passed() {
    let map = test_map!(Catch);
    let n = 100;

    let regular = CatchPP::new(&map).passed_objects(n).calculate();
    let mut gradual = CatchGradualPerformanceAttributes::new(&map, 0);

    let state = CatchScoreState {
        max_combo: 101,
        n_fruits: 99,
        n_droplets: 2,
        n_tiny_droplets: 68,
        n_tiny_droplet_misses: 0,
        n_misses: 0,
    };

    let gradual = gradual.process_next_n_objects(state, n).unwrap();

    assert_eq!(regular, gradual);
}
