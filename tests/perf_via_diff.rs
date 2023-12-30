#![cfg(not(any(feature = "async_tokio", feature = "async_std")))]

use rosu_pp::{CatchPP, ManiaPP, OsuPP, TaikoPP};

mod common;

#[test]
fn osu() {
    let map = test_map!(Osu);

    let mods = 8 + 64;
    let misses = 2;

    let regular = OsuPP::new(map).mods(mods).n_misses(misses).calculate();

    let via_diff = OsuPP::from(regular.difficulty.clone())
        .mods(mods)
        .n_misses(misses)
        .calculate();

    assert_eq!(regular, via_diff);
}

#[test]
fn taiko() {
    let map = test_map!(Taiko);

    let mods = 8 + 64;
    let misses = 2;

    let regular = TaikoPP::new(map).mods(mods).n_misses(misses).calculate();

    let via_diff = TaikoPP::from(regular.difficulty.clone())
        .mods(mods)
        .n_misses(misses)
        .calculate();

    assert_eq!(regular, via_diff);
}

#[test]
fn catch() {
    let map = test_map!(Catch);

    let mods = 8 + 64;
    let misses = 2;

    let regular = CatchPP::new(map).mods(mods).misses(misses).calculate();

    let via_diff = CatchPP::from(regular.difficulty.clone())
        .mods(mods)
        .misses(misses)
        .calculate();

    assert_eq!(regular, via_diff);
}

#[test]
fn mania() {
    let map = test_map!(Mania);

    let mods = 8 + 64;
    let misses = 2;

    let regular = ManiaPP::new(map).mods(mods).n_misses(misses).calculate();

    let via_diff = ManiaPP::from(regular.difficulty.clone())
        .mods(mods)
        .n_misses(misses)
        .calculate();

    assert_eq!(regular, via_diff);
}
