#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::{
    osu::{OsuGradualDifficultyAttributes, OsuGradualPerformanceAttributes, OsuScoreState},
    Beatmap, OsuPP, OsuStars,
};

use crate::common::Osu;

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = OsuGradualDifficultyAttributes::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn iter_end_eq_regular() {
    let map = test_map!(Osu);
    let regular = OsuStars::new(&map).calculate();

    let iter_end = OsuGradualDifficultyAttributes::new(&map, 0)
        .last()
        .expect("empty iter");

    assert_eq!(regular, iter_end);
}

#[test]
fn correct_empty() {
    let map = test_map!(Osu);
    let mut gradual = OsuGradualPerformanceAttributes::new(&map, 0);
    let state = OsuScoreState::default();

    let first_attrs = gradual.nth(state.clone(), usize::MAX);

    assert!(first_attrs.is_some());
    assert!(gradual.next(state).is_none());
}

#[test]
fn next_and_next_n() {
    let map = test_map!(Osu);
    let state = OsuScoreState::default();

    let mut gradual1 = OsuGradualPerformanceAttributes::new(&map, 0);
    let mut gradual2 = OsuGradualPerformanceAttributes::new(&map, 0);

    for _ in 0..20 {
        let _ = gradual1.next(state.clone());
        let _ = gradual2.next(state.clone());
    }

    let n = 80;

    for _ in 1..n {
        let _ = gradual1.next(state.clone());
    }

    let state = OsuScoreState {
        max_combo: 122,
        n300: 88,
        n100: 8,
        n50: 2,
        n_misses: 2,
    };

    let next = gradual1.next(state.clone());
    let next_n = gradual2.nth(state, n - 1);

    assert_eq!(next_n, next);
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Osu);
    let regular = OsuPP::new(&map).calculate();
    let mut gradual = OsuGradualPerformanceAttributes::new(&map, 0);

    let state = OsuScoreState {
        max_combo: 909,
        n300: 601,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.nth(state, usize::MAX).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_eq_regular_passed() {
    let map = test_map!(Osu);
    let n = 100;

    let regular = OsuPP::new(&map).passed_objects(n).calculate();
    let mut gradual = OsuGradualPerformanceAttributes::new(&map, 0);

    let state = OsuScoreState {
        max_combo: 122,
        n300: 100,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let gradual = gradual.nth(state, n - 1).unwrap();

    assert_eq!(regular, gradual);
}
