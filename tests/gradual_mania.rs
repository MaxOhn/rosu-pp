#![cfg(all(not(any(feature = "async_tokio", feature = "async_std")), feature = "gradual"))]

use rosu_pp::{
    mania::{ManiaGradualDifficultyAttributes, ManiaGradualPerformanceAttributes, ManiaScoreState},
    Beatmap, ManiaPP, ManiaStars,
};

use crate::common::Mania;

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = ManiaGradualDifficultyAttributes::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn iter_end_eq_regular() {
    let map = test_map!(Mania);
    let regular = ManiaStars::new(&map).calculate();

    let iter_end = ManiaGradualDifficultyAttributes::new(&map, 0)
        .last()
        .expect("empty iter");

    assert_eq!(regular, iter_end);
}

#[test]
fn correct_empty() {
    let map = test_map!(Mania);
    let mut gradual = ManiaGradualPerformanceAttributes::new(&map, 0);

    let state = ManiaScoreState {
        n320: 0,
        n300: 0,
        n200: 0,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let first_attrs = gradual.process_next_n_objects(state.clone(), usize::MAX);

    assert!(first_attrs.is_some());
    assert!(gradual.process_next_object(state).is_none());
}

#[test]
fn next_and_next_n() {
    let map = test_map!(Mania);

    let mut state = ManiaScoreState {
        n320: 0,
        n300: 0,
        n200: 0,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let mut gradual1 = ManiaGradualPerformanceAttributes::new(&map, 0);
    let mut gradual2 = ManiaGradualPerformanceAttributes::new(&map, 0);

    for _ in 0..20 {
        let _ = gradual1.process_next_object(state.clone());
        let _ = gradual2.process_next_object(state.clone());
        state.n320 += 1;
    }

    let n = 80;

    for _ in 1..n {
        let _ = gradual1.process_next_object(state.clone());
        state.n320 += 1;
    }

    let next = gradual1.process_next_object(state.clone());
    let next_n = gradual2.process_next_n_objects(state, n);

    assert_eq!(next_n, next);
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Mania);
    let regular = ManiaPP::new(&map).calculate();

    let mut gradual = ManiaGradualPerformanceAttributes::new(&map, 0);

    let state = ManiaScoreState {
        n320: 3238,
        n300: 0,
        n200: 0,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.process_next_n_objects(state, usize::MAX).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_eq_regular_passed() {
    let map = test_map!(Mania);
    let n = 100;

    let state = ManiaScoreState {
        n320: 100,
        n300: 0,
        n200: 0,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let regular = ManiaPP::new(&map)
        .passed_objects(n)
        .state(state.clone())
        .calculate();

    let mut gradual = ManiaGradualPerformanceAttributes::new(&map, 0);
    let gradual = gradual.process_next_n_objects(state, n).unwrap();

    assert_eq!(regular, gradual);
}
