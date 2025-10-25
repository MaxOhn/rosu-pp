use rosu_pp::{Beatmap, GameMods, model::mode::GameMode};

use crate::common::assert_eq_float;

mod common;

#[test]
fn osu() {
    let map = Beatmap::from_path(common::OSU).unwrap();

    assert_eq!(map.mode, GameMode::Osu);
    assert_eq!(map.version, 14);
    assert_eq_float(map.ar, 9.3);
    assert_eq_float(map.od, 8.8);
    assert_eq_float(map.cs, 4.5);
    assert_eq_float(map.hp, 5.0);
    assert_eq_float(map.slider_multiplier, 1.7);
    assert_eq_float(map.slider_tick_rate, 1.0);
    assert_eq!(map.hit_objects.len(), 601);
    assert_eq!(map.hit_sounds.len(), 601);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 50);
    assert_eq!(map.effect_points.len(), 0);
    assert_eq_float(map.stack_leniency, 0.5);
    assert_eq!(map.breaks.len(), 1);
}

#[test]
fn taiko() {
    let map = Beatmap::from_path(common::TAIKO).unwrap();

    assert_eq!(map.mode, GameMode::Taiko);
    assert_eq!(map.version, 14);
    assert_eq_float(map.ar, 8.0);
    assert_eq_float(map.od, 5.0);
    assert_eq_float(map.cs, 2.0);
    assert_eq_float(map.hp, 6.0);
    assert_eq_float(map.slider_multiplier, 1.4);
    assert_eq_float(map.slider_tick_rate, 1.0);
    assert_eq!(map.hit_objects.len(), 295);
    assert_eq!(map.hit_sounds.len(), 295);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 3);
    assert_eq!(map.effect_points.len(), 7);
    assert_eq_float(map.stack_leniency, 0.7);
    assert_eq!(map.breaks.len(), 0);
}

#[test]
fn catch() {
    let map = Beatmap::from_path(common::CATCH).unwrap();

    assert_eq!(map.mode, GameMode::Catch);
    assert_eq!(map.version, 14);
    assert_eq_float(map.ar, 8.0);
    assert_eq_float(map.od, 8.0);
    assert_eq_float(map.cs, 3.5);
    assert_eq_float(map.hp, 5.0);
    assert_eq_float(map.slider_multiplier, 1.45);
    assert_eq_float(map.slider_tick_rate, 1.0);
    assert_eq!(map.hit_objects.len(), 477);
    assert_eq!(map.hit_sounds.len(), 477);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 0);
    assert_eq!(map.effect_points.len(), 16);
    assert_eq_float(map.stack_leniency, 0.7);
    assert_eq!(map.breaks.len(), 0);
}

#[test]
fn mania() {
    let map = Beatmap::from_path(common::MANIA).unwrap();

    assert_eq!(map.mode, GameMode::Mania);
    assert_eq!(map.version, 14);
    assert_eq_float(map.ar, 5.0);
    assert_eq_float(map.od, 8.0);
    assert_eq_float(map.cs, 4.0);
    assert_eq_float(map.hp, 8.0);
    assert_eq_float(map.slider_multiplier, 1.4);
    assert_eq_float(map.slider_tick_rate, 1.0);
    assert_eq!(map.hit_objects.len(), 594);
    assert_eq!(map.hit_sounds.len(), 594);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 0);
    assert_eq!(map.effect_points.len(), 0);
    assert_eq_float(map.stack_leniency, 0.7);
    assert_eq!(map.breaks.len(), 0);
}

#[test]
fn empty_osu() {
    let map = Beatmap::from_bytes(&[]).unwrap();
    let _ = map.convert(GameMode::Osu, &GameMods::default());
}

#[test]
fn empty_taiko() {
    let map = Beatmap::from_bytes(&[]).unwrap();
    let _ = map.convert(GameMode::Taiko, &GameMods::default());
}

#[test]
fn empty_catch() {
    let map = Beatmap::from_bytes(&[]).unwrap();
    let _ = map.convert(GameMode::Catch, &GameMods::default());
}

#[test]
fn empty_mania() {
    let map = Beatmap::from_bytes(&[]).unwrap();
    let _ = map.convert(GameMode::Mania, &GameMods::default());
}
