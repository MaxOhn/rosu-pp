#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::osu::OsuOwnedGradualPerformance;
use rosu_pp::{
    osu::{OsuGradualDifficulty, OsuGradualPerformance, OsuScoreState},
    Beatmap, OsuPP,
};

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = OsuGradualDifficulty::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Osu);
    let regular = OsuPP::new(&map).calculate();
    let mut gradual = OsuGradualPerformance::new(&map, 0);

    let state = OsuScoreState {
        max_combo: 909,
        n300: 601,
        n100: 0,
        n50: 0,
        n_misses: 0,
    };

    let gradual_end = gradual.last(state.clone()).unwrap();

    assert_eq!(regular, gradual_end);
    assert!(gradual.next(state).is_none());
}

#[test]
fn gradual_complete_next() {
    let map = test_map!(Osu);
    let mods = 88; // HDHRDT

    let mut gradual = OsuGradualPerformance::new(map, mods);
    let mut gradual_2nd = OsuGradualPerformance::new(map, mods);
    let mut gradual_3rd = OsuGradualPerformance::new(map, mods);
    let mut gradual_owned = OsuOwnedGradualPerformance::new(map.to_owned(), mods);

    let mut state = OsuScoreState::default();

    for i in 1.. {
        state.n_misses += 1;

        let Some(next_gradual) = gradual.next(state.clone()) else {
            assert_eq!(i, map.hit_objects.len() + 1);
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

        let mut regular_calc = OsuPP::new(&map)
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
