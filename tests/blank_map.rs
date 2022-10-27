use rosu_pp::{Beatmap, CatchPP, GameMode, ManiaPP, OsuPP, TaikoPP};

#[test]
fn osu() {
    let map = Beatmap::default();
    let _ = OsuPP::new(&map).calculate();
}

#[test]
fn taiko() {
    let mut map = Beatmap::default();

    // convert
    let _ = TaikoPP::new(&map).calculate();

    // regular
    map.mode = GameMode::Taiko;
    let _ = TaikoPP::new(&map).calculate();
}

#[test]
fn catch() {
    let mut map = Beatmap::default();

    // convert
    let _ = CatchPP::new(&map).calculate();

    // regular
    map.mode = GameMode::Catch;
    let _ = CatchPP::new(&map).calculate();
}

#[test]
fn mania() {
    let mut map = Beatmap::default();

    // convert
    let _ = ManiaPP::new(&map).calculate();

    // regular
    map.mode = GameMode::Mania;
    let _ = ManiaPP::new(&map).calculate();
}
