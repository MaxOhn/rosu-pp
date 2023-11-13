#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::{
    mania::{
        ManiaGradualDifficulty, ManiaGradualPerformance, ManiaOwnedGradualPerformance,
        ManiaScoreState,
    },
    Beatmap, ManiaPP,
};

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = ManiaGradualDifficulty::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Mania);
    let regular = ManiaPP::new(map).calculate();

    let state = ManiaScoreState {
        n320: map.hit_objects.len(),
        ..Default::default()
    };

    let gradual = ManiaGradualPerformance::new(map, 0).last(state).unwrap();

    assert_eq!(regular, gradual);
}

#[test]
fn gradual_complete_next() {
    let map = test_map!(Mania);
    let mods = 67; // NFEZDT

    let mut gradual = ManiaGradualPerformance::new(map, mods);
    let mut gradual_2nd = ManiaGradualPerformance::new(map, mods);
    let mut gradual_3rd = ManiaGradualPerformance::new(map, mods);
    let mut gradual_owned = ManiaOwnedGradualPerformance::new(map.to_owned(), mods);

    let mut state = ManiaScoreState::default();

    for i in 1.. {
        state.n320 += 1;

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

        let regular_calc = ManiaPP::new(map)
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
