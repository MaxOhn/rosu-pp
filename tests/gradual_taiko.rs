#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::{
    taiko::{
        TaikoGradualDifficulty, TaikoGradualPerformance, TaikoOwnedGradualPerformance,
        TaikoScoreState,
    },
    Beatmap, TaikoPP,
};

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attrs = TaikoGradualDifficulty::new(&map, 0);

    assert!(attrs.next().is_none());
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Taiko);
    let regular = TaikoPP::new(&map).calculate();
    let mut gradual = TaikoGradualPerformance::new(&map, 0);

    let state = TaikoScoreState {
        max_combo: 289,
        n300: 289,
        n100: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.last(state).unwrap();

    assert_eq!(regular, gradual_end);
}

#[test]
fn gradual_complete_next() {
    let map = test_map!(Taiko);
    let mods = 88; // HDHRDT

    let mut gradual = TaikoGradualPerformance::new(map, mods);
    let mut gradual_2nd = TaikoGradualPerformance::new(map, mods);
    let mut gradual_3rd = TaikoGradualPerformance::new(map, mods);
    let mut gradual_owned = TaikoOwnedGradualPerformance::new(map.to_owned(), mods);

    let mut state = TaikoScoreState::default();

    for i in 1.. {
        state.n_misses += 1;

        let Some(next_gradual) = gradual.next(state.clone()) else {
            assert_eq!(i, map.n_circles as usize + 1);
            assert!(gradual_2nd.last(state.clone()).is_some() || map.hit_objects.len() % 2 == 0);
            assert!(gradual_3rd.last(state.clone()).is_some() || map.hit_objects.len() % 3 == 0);
            assert!(gradual_owned.next(state.clone()).is_none());
            break;
        };

        if i % 2 == 0 {
            let next_gradual_2nd = gradual_2nd.nth(state.clone(), 1).unwrap();
            assert_eq!(next_gradual, next_gradual_2nd, "i={i}");
        }

        if i % 3 == 0 {
            let next_gradual_3rd = gradual_3rd.nth(state.clone(), 2).unwrap();
            assert_eq!(next_gradual, next_gradual_3rd, "i={i}");
        }

        let next_gradual_owned = gradual_owned.next(state.clone()).unwrap();

        let mut regular_calc = TaikoPP::new(&map)
            .mods(mods)
            .passed_objects(i)
            .state(state.clone());

        let regular_state = regular_calc.generate_state();
        assert_eq!(state, regular_state);

        let regular = regular_calc.calculate();

        assert_eq!(next_gradual, next_gradual_owned, "i={i}");
        assert_eq!(next_gradual, regular, "i={i}");
    }
}
