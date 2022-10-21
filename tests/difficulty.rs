#![cfg(not(any(feature = "async_tokio", feature = "async_std")))]

use common::{test_map, Osu};
use rosu_pp::{CatchStars, ManiaStars, OsuStars, TaikoStars};

use crate::common::{Catch, Mania, Mode, Taiko};

mod common;

#[test]
fn difficulty_osu() {
    let map = test_map::<Osu>();
    let attrs = OsuStars::new(&map).calculate();

    assert_eq!(Osu::TEST_DIFF_ATTRS, attrs);
}

#[test]
fn difficulty_taiko() {
    let map = test_map::<Taiko>();
    let attrs = TaikoStars::new(&map).calculate();

    assert_eq!(Taiko::TEST_DIFF_ATTRS, attrs);
}

#[test]
fn difficulty_catch() {
    let map = test_map::<Catch>();
    let attrs = CatchStars::new(&map).calculate();

    assert_eq!(Catch::TEST_DIFF_ATTRS, attrs);
}

#[test]
fn difficulty_mania() {
    let map = test_map::<Mania>();
    let attrs = ManiaStars::new(&map).calculate();

    assert_eq!(Mania::TEST_DIFF_ATTRS, attrs);
}
