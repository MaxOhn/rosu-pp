#![cfg(all(
    not(any(feature = "async_tokio", feature = "async_std")),
    feature = "gradual"
))]

use rosu_pp::{
    catch::{
        CatchGradualDifficulty, CatchGradualPerformance, CatchOwnedGradualPerformance,
        CatchScoreState,
    },
    Beatmap, BeatmapExt, CatchPP,
};

mod common;

#[test]
fn empty_map() {
    let map = Beatmap::default();
    let mut attributes = CatchGradualDifficulty::new(&map, 0);

    assert!(attributes.next().is_none());
}

#[test]
fn gradual_end_eq_regular() {
    let map = test_map!(Catch);
    let regular = CatchPP::new(map).calculate();

    let mut gradual = CatchGradualPerformance::new(map, 0);

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
fn gradual_complete_next() {
    let map = test_map!(Catch);
    let mods = 88; // HDHRDT
    let n_objects = dbg!(map.catch_hitobjects(mods).len());

    let mut gradual = CatchGradualPerformance::new(map, mods);
    let mut gradual_2nd = CatchGradualPerformance::new(map, mods);
    let mut gradual_3rd = CatchGradualPerformance::new(map, mods);
    let mut gradual_owned = CatchOwnedGradualPerformance::new(map.to_owned(), mods);

    let mut state = CatchScoreState::default();

    for i in 1.. {
        state.n_misses += 1;

        let Some(next_gradual) = gradual.next(state.clone()) else {
            assert_eq!(i, n_objects + 1);
            assert!(gradual_2nd.last(state.clone()).is_some() || n_objects % 2 == 0);
            assert!(gradual_3rd.last(state.clone()).is_some() || n_objects % 3 == 0);
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

        let mut regular_calc = CatchPP::new(map)
            .mods(mods)
            .passed_objects(i)
            .state(state.clone());

        let _regular_state = regular_calc.generate_state();
        // FIXME: juicestreams are currently added to the attributes in their entirety
        //        so the state won't change while objects of the same juicestream are processed
        // assert_eq!(state, _regular_state);

        let regular = regular_calc.calculate();

        assert_eq!(next_gradual, next_gradual_owned, "i={i}");
        assert_eq!(next_gradual, regular, "i={i}");
    }
}
