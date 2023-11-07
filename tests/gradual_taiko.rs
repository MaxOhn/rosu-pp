#![cfg(all(not(any(feature = "async_tokio", feature = "async_std")), feature = "gradual"))]

use rosu_pp::{
    taiko::{TaikoGradualDifficultyAttributes, TaikoGradualPerformanceAttributes, TaikoScoreState},
    Beatmap, TaikoPP, TaikoStars,
};

use crate::common::Taiko;

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attrs = TaikoGradualDifficultyAttributes::new(&map, 0);

    assert!(attrs.next().is_none());
}

#[test]
fn iter_end_eq_regular() {
    let map = test_map!(Taiko);
    let regular = TaikoStars::new(&map).calculate();

    let iter_end = TaikoGradualDifficultyAttributes::new(&map, 0)
        .last()
        .expect("empty iter");

    assert_eq!(regular, iter_end);
}

#[test]
fn correct_empty() {
    let map = test_map!(Taiko);
    let mut gradual = TaikoGradualPerformanceAttributes::new(&map, 0);
    let state = TaikoScoreState::default();

    let first_attrs = gradual.process_next_n_objects(state.clone(), usize::MAX);

    assert!(first_attrs.is_some());
    assert!(gradual.process_next_object(state).is_none());
}

#[test]
fn next_and_next_n() {
    let map = test_map!(Taiko);
    let state = TaikoScoreState::default();

    let mut gradual1 = TaikoGradualPerformanceAttributes::new(&map, 0);
    let mut gradual2 = TaikoGradualPerformanceAttributes::new(&map, 0);

    for _ in 0..50 {
        let _ = gradual1.process_next_object(state.clone());
        let _ = gradual2.process_next_object(state.clone());
    }

    let n = 200;

    for _ in 1..n {
        let _ = gradual1.process_next_object(state.clone());
    }

    let state = TaikoScoreState {
        max_combo: 246,
        n300: 200,
        n100: 40,
        n_misses: 6,
    };

    let next = gradual1.process_next_object(state.clone());
    let next_n = gradual2.process_next_n_objects(state, n);

    assert_eq!(next_n, next);
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Taiko);
    let regular = TaikoPP::new(&map).calculate();
    let mut gradual = TaikoGradualPerformanceAttributes::new(&map, 0);

    let state = TaikoScoreState {
        max_combo: 289,
        n300: 289,
        n100: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.process_next_n_objects(state, usize::MAX).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_eq_regular_passed() {
    let map = test_map!(Taiko);
    let n = 250;

    let regular = TaikoPP::new(&map).passed_objects(n).calculate();
    let mut gradual = TaikoGradualPerformanceAttributes::new(&map, 0);

    let state = TaikoScoreState {
        max_combo: 250,
        n300: 250,
        n100: 0,
        n_misses: 0,
    };

    let gradual = gradual.process_next_n_objects(state, n).unwrap();

    assert_eq!(regular, gradual);
}
