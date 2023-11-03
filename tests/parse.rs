use rosu_pp::{Beatmap, GameMode};

use crate::common::{Catch, Mania, Osu, Taiko};

mod common;

#[cfg(not(any(feature = "async_tokio", feature = "async_std")))]
mod sync {
    use super::*;

    #[test]
    fn parse_osu() {
        assert_osu(test_map!(Osu));
    }

    #[test]
    fn parse_taiko() {
        assert_taiko(test_map!(Taiko));
    }

    #[test]
    fn parse_catch() {
        assert_catch(test_map!(Catch));
    }

    #[test]
    fn parse_mania() {
        assert_mania(test_map!(Mania));
    }
}

#[cfg(feature = "async_tokio")]
mod async_tokio {
    use tokio::runtime::Builder as RuntimeBuilder;

    use super::*;

    #[test]
    fn parse_osu() {
        RuntimeBuilder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async { assert_osu(test_map!(Osu)) });
    }

    #[test]
    fn parse_taiko() {
        RuntimeBuilder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async { assert_taiko(test_map!(Taiko)) });
    }

    #[test]
    fn parse_catch() {
        RuntimeBuilder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async { assert_catch(test_map!(Catch)) });
    }

    #[test]
    fn parse_mania() {
        RuntimeBuilder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async { assert_mania(test_map!(Mania)) });
    }
}

#[cfg(feature = "async_std")]
mod async_tokio {
    use super::*;

    #[test]
    fn parse_osu() {
        async_std::task::block_on(async { assert_osu(test_map!(Osu)) });
    }

    #[test]
    fn parse_taiko() {
        async_std::task::block_on(async { assert_taiko(test_map!(Taiko)) });
    }

    #[test]
    fn parse_catch() {
        async_std::task::block_on(async { assert_catch(test_map!(Catch)) });
    }

    #[test]
    fn parse_mania() {
        async_std::task::block_on(async { assert_mania(test_map!(Mania)) });
    }
}

fn assert_osu(map: Beatmap) {
    assert_eq!(map.mode, GameMode::Osu);
    assert_eq!(map.version, 14);
    assert_eq!(map.n_circles, 307);
    assert_eq!(map.n_sliders, 293);
    assert_eq!(map.n_spinners, 1);
    assert!((map.ar - 9.3).abs() <= f32::EPSILON);
    assert!((map.od - 8.8).abs() <= f32::EPSILON);
    assert!((map.cs - 4.5).abs() <= f32::EPSILON);
    assert!((map.hp - 5.0).abs() <= f32::EPSILON);
    assert!((map.slider_mult - 1.7).abs() <= f64::EPSILON);
    assert!((map.tick_rate - 1.0).abs() <= f64::EPSILON);
    assert_eq!(map.hit_objects.len(), 601);
    assert_eq!(map.sounds.len(), 601);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 50);
    assert_eq!(map.effect_points.len(), 131);
    assert!((map.stack_leniency - 0.5).abs() <= f32::EPSILON);
    assert_eq!(map.breaks.len(), 1)
}

fn assert_taiko(map: Beatmap) {
    assert_eq!(map.mode, GameMode::Taiko);
    assert_eq!(map.version, 14);
    assert_eq!(map.n_circles, 289);
    assert_eq!(map.n_sliders, 4);
    assert_eq!(map.n_spinners, 2);
    assert!((map.ar - 8.0).abs() <= f32::EPSILON);
    assert!((map.od - 5.0).abs() <= f32::EPSILON);
    assert!((map.cs - 2.0).abs() <= f32::EPSILON);
    assert!((map.hp - 6.0).abs() <= f32::EPSILON);
    assert!((map.slider_mult - 1.4).abs() <= f64::EPSILON);
    assert!((map.tick_rate - 1.0).abs() <= f64::EPSILON);
    assert_eq!(map.hit_objects.len(), 295);
    assert_eq!(map.sounds.len(), 295);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 3);
    assert_eq!(map.effect_points.len(), 8);
    assert!((map.stack_leniency - 0.7).abs() <= f32::EPSILON);
    assert_eq!(map.breaks.len(), 0)
}

fn assert_catch(map: Beatmap) {
    assert_eq!(map.mode, GameMode::Catch);
    assert_eq!(map.version, 14);
    assert_eq!(map.n_circles, 249);
    assert_eq!(map.n_sliders, 227);
    assert_eq!(map.n_spinners, 1);
    assert!((map.ar - 8.0).abs() <= f32::EPSILON);
    assert!((map.od - 8.0).abs() <= f32::EPSILON);
    assert!((map.cs - 3.5).abs() <= f32::EPSILON);
    assert!((map.hp - 5.0).abs() <= f32::EPSILON);
    assert!((map.slider_mult - 1.45).abs() <= f64::EPSILON);
    assert!((map.tick_rate - 1.0).abs() <= f64::EPSILON);
    assert_eq!(map.hit_objects.len(), 477);
    assert_eq!(map.sounds.len(), 477);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 0);
    assert_eq!(map.effect_points.len(), 57);
    assert!((map.stack_leniency - 0.7).abs() <= f32::EPSILON);
    assert_eq!(map.breaks.len(), 0)
}

fn assert_mania(map: Beatmap) {
    assert_eq!(map.mode, GameMode::Mania);
    assert_eq!(map.version, 14);
    assert_eq!(map.n_circles, 473);
    assert_eq!(map.n_sliders, 121);
    assert_eq!(map.n_spinners, 0);
    assert!((map.ar - 5.0).abs() <= f32::EPSILON);
    assert!((map.od - 8.0).abs() <= f32::EPSILON);
    assert!((map.cs - 4.0).abs() <= f32::EPSILON);
    assert!((map.hp - 8.0).abs() <= f32::EPSILON);
    assert!((map.slider_mult - 1.4).abs() <= f64::EPSILON);
    assert!((map.tick_rate - 1.0).abs() <= f64::EPSILON);
    assert_eq!(map.hit_objects.len(), 594);
    assert_eq!(map.sounds.len(), 594);
    assert_eq!(map.timing_points.len(), 1);
    assert_eq!(map.difficulty_points.len(), 0);
    assert_eq!(map.effect_points.len(), 1);
    assert!((map.stack_leniency - 0.7).abs() <= f32::EPSILON);
    assert_eq!(map.breaks.len(), 0)
}
