#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::{
    mania::{ManiaGradualDifficulty, ManiaGradualPerformance, ManiaScoreState},
    Beatmap, ManiaPP, ManiaStars,
};

use crate::common::Mania;

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = ManiaGradualDifficulty::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn iter_end_eq_regular() {
    let map = test_map!(Mania);
    let regular = ManiaStars::new(&map).calculate();

    let iter_end = ManiaGradualDifficulty::new(&map, 0)
        .last()
        .expect("empty iter");

    assert_eq!(regular, iter_end);
}

#[test]
fn correct_empty() {
    let map = test_map!(Mania);
    let mut gradual = ManiaGradualPerformance::new(&map, 0);

    let state = ManiaScoreState::default();

    let first_attrs = gradual.nth(state.clone(), usize::MAX);

    assert!(first_attrs.is_some());
    assert!(gradual.next(state).is_none());
}

#[test]
fn next_and_next_n() {
    let map = test_map!(Mania);

    let mut state = ManiaScoreState::default();

    let mut gradual1 = ManiaGradualPerformance::new(&map, 0);
    let mut gradual2 = ManiaGradualPerformance::new(&map, 0);

    for _ in 0..20 {
        let _ = gradual1.next(state.clone());
        let _ = gradual2.next(state.clone());
        state.n320 += 1;
    }

    let n = 80;

    for _ in 1..n {
        let _ = gradual1.next(state.clone());
        state.n320 += 1;
    }

    let next = gradual1.next(state.clone());
    let next_n = gradual2.nth(state, n - 1);

    assert_eq!(next_n, next);
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Mania);
    let regular = ManiaPP::new(&map).calculate();

    let mut gradual = ManiaGradualPerformance::new(&map, 0);

    let state = ManiaScoreState {
        n320: map.hit_objects.len(),
        ..Default::default()
    };

    let gradual_end = gradual.nth(state, usize::MAX).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_eq_regular_passed() {
    let map = test_map!(Mania);
    let n = 100;

    let state = ManiaScoreState {
        n320: 100,
        ..Default::default()
    };

    let regular = ManiaPP::new(&map)
        .passed_objects(n)
        .state(state.clone())
        .calculate();

    let gradual = ManiaGradualPerformance::new(&map, 0)
        .nth(state, n - 1)
        .unwrap();

    assert_eq!(regular, gradual);
}
