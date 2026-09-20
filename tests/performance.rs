use std::panic::{self, UnwindSafe};

use rosu_pp::{
    Beatmap,
    catch::{CatchPerformance, CatchPerformanceAttributes},
    mania::{ManiaPerformance, ManiaPerformanceAttributes},
    osu::{OsuPerformance, OsuPerformanceAttributes},
    taiko::{TaikoPerformance, TaikoPerformanceAttributes},
};

use self::common::*;

mod common;

include!("data/ext_refs.rs");
include!("data/ext_acc_refs.rs");

macro_rules! test_cases {
    ( $mode:ident: $path:ident {
        $( $( $mods:ident )+ => {
            $( $key:ident: $value:expr $( , )? )*
        } ;)*
    } ) => {
        let map = Beatmap::from_path(common::$path).unwrap();

        $(
            let mods = 0 $( + $mods )*;
            let (calc, expected) = test_cases!(@$mode { map, $( $key: $value, )* });
            let actual = calc.mods(mods).calculate().unwrap();
            run(&actual, &expected, mods);
        )*
    };
    ( @Osu {
        $map:ident,
        pp: $pp:expr,
        pp_acc: $pp_acc:expr,
        pp_aim: $pp_aim:expr,
        pp_flashlight: $pp_flashlight:expr,
        pp_speed: $pp_speed:expr,
        pp_reading: $pp_reading:expr,
        effective_miss_count: $effective_miss_count:expr,
        speed_deviation: $speed_deviation:expr,
        combo_based_estimated_miss_count: $combo_based_estimated_miss_count:expr,
        score_based_estimated_miss_count: $score_based_estimated_miss_count:expr,
        aim_estimated_slider_breaks: $aim_estimated_slider_breaks:expr,
        speed_estimated_slider_breaks: $speed_estimated_slider_breaks:expr,
    }) => {
        (
            OsuPerformance::from(&$map).lazer(true),
            OsuPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_aim: $pp_aim,
                pp_flashlight: $pp_flashlight,
                pp_speed: $pp_speed,
                pp_reading: $pp_reading,
                effective_miss_count: $effective_miss_count,
                speed_deviation: $speed_deviation,
                combo_based_estimated_miss_count: $combo_based_estimated_miss_count,
                score_based_estimated_miss_count: $score_based_estimated_miss_count,
                aim_estimated_slider_breaks: $aim_estimated_slider_breaks,
                speed_estimated_slider_breaks: $speed_estimated_slider_breaks,
                ..Default::default()
            },
        )
    };
    ( @Taiko {
        $map: ident,
        pp: $pp:expr,
        pp_acc: $pp_acc:expr,
        pp_difficulty: $pp_difficulty:expr,
        estimated_unstable_rate: $estimated_unstable_rate:expr,
    }) => {
        (
            TaikoPerformance::from(&$map),
            TaikoPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_difficulty: $pp_difficulty,
                estimated_unstable_rate: $estimated_unstable_rate,
                ..Default::default()
            },
        )
    };
    ( @Catch {
        $map:ident,
        pp: $pp:expr,
    }) => {
        (
            CatchPerformance::from(&$map),
            CatchPerformanceAttributes {
                pp: $pp,
                ..Default::default()
            },
        )
    };
    ( @Mania {
        $map:ident,
        pp: $pp:expr,
        pp_difficulty: $pp_difficulty:expr,
    }) => {
        (
            ManiaPerformance::from(&$map),
            ManiaPerformanceAttributes {
                pp: $pp,
                pp_difficulty: $pp_difficulty,
                ..Default::default()
            },
        )
    };
}

#[test]
fn basic_taiko() {
    test_cases! {
        Taiko: TAIKO {
            NM => {
                pp: 130.3342753050141,
                pp_acc: 96.78235028730231,
                pp_difficulty: 33.551925017711795,
                estimated_unstable_rate: Some(146.32383579722838),
            };
            HD => {
                pp: 138.26399007691603,
                pp_acc: 104.04102655884999,
                pp_difficulty: 34.22296351806603,
                estimated_unstable_rate: Some(146.32383579722838),
            };
            HR => {
                pp: 166.77434181278937,
                pp_acc: 130.20134262470424,
                pp_difficulty: 36.57299918808513,
                estimated_unstable_rate: Some(120.87621218031911),
            };
            DT => {
                pp: 266.1232222806763,
                pp_acc: 173.19552687459048,
                pp_difficulty: 92.92769540608585,
                estimated_unstable_rate: Some(97.54922386481893),
            };
        }
    };
}

#[test]
fn convert_taiko() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Taiko: OSU {
            NM => {
                pp: 388.8389762212271,
                pp_acc: 220.27109512284244,
                pp_difficulty: 168.56788109838465,
                estimated_unstable_rate: Some(81.74165086164194),
            };
            HD => {
                pp: 389.68181562671896,
                pp_acc: 220.27109512284244,
                pp_difficulty: 169.41072050387652,
                estimated_unstable_rate: Some(81.74165086164194),
            };
            HR => {
                pp: 466.289743228387,
                pp_acc: 259.1975812188925,
                pp_difficulty: 207.0921620094945,
                estimated_unstable_rate: Some(70.8427640800897),
            };
            DT => {
                pp: 805.083671145498,
                pp_acc: 392.44103449845574,
                pp_difficulty: 412.64263664704225,
                estimated_unstable_rate: Some(54.494433907761305),
            };
        }
    }
}

#[test]
fn basic_catch() {
    test_cases! {
        Catch: CATCH {
            NM => { pp: 113.85903714373046 };
            HD => { pp: 136.63084457247655 };
            HD HR => { pp: 231.7403429678108 };
            DT => { pp: 247.18402249125842 };
        }
    };
}

#[test]
fn convert_catch() {
    test_cases! {
        Catch: OSU {
            NM => { pp: 232.52175944328079 };
            HD => { pp: 256.35523645996665 };
            HD HR => { pp: 327.71861407740374 };
            DT => { pp: 503.47065792054815 };
        }
    };
}

#[test]
fn basic_mania() {
    test_cases! {
        Mania: MANIA {
            NM => { pp: 108.92297471705167, pp_difficulty: 108.92297471705167 };
            EZ => { pp: 54.46148735852584, pp_difficulty: 108.92297471705167 };
            DT => { pp: 224.52717042937203, pp_difficulty: 224.52717042937203 };
        }
    };
}

#[test]
fn convert_mania() {
    test_cases! {
        Mania: OSU {
            NM => { pp: 101.39189449271568, pp_difficulty: 101.39189449271568 };
            EZ => { pp: 50.69594724635784, pp_difficulty: 101.39189449271568 };
            DT => { pp: 198.46891237015896, pp_difficulty: 198.46891237015896 };
        }
    };
}

fn run<A>(actual: &A, expected: &A, mods: u32)
where
    A: AssertEq,
    for<'a> &'a A: UnwindSafe,
{
    if panic::catch_unwind(|| actual.assert_eq(expected)).is_err() {
        panic!("Mods: {mods}");
    }
}

impl AssertEq for OsuPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_acc,
            pp_aim,
            pp_flashlight,
            pp_reading,
            pp_speed,
            effective_miss_count,
            speed_deviation,
            combo_based_estimated_miss_count,
            score_based_estimated_miss_count,
            aim_estimated_slider_breaks,
            speed_estimated_slider_breaks,
        } = self;

        assert_eq_float(*pp, expected.pp);
        assert_eq_float(*pp_acc, expected.pp_acc);
        assert_eq_float(*pp_aim, expected.pp_aim);
        assert_eq_float(*pp_flashlight, expected.pp_flashlight);
        assert_eq_float(*pp_reading, expected.pp_reading);
        assert_eq_float(*pp_speed, expected.pp_speed);
        assert_eq_float(*effective_miss_count, expected.effective_miss_count);
        assert_eq_option(*speed_deviation, expected.speed_deviation);
        assert_eq_float(
            *combo_based_estimated_miss_count,
            expected.combo_based_estimated_miss_count,
        );
        assert_eq_option(
            *score_based_estimated_miss_count,
            expected.score_based_estimated_miss_count,
        );
        assert_eq_float(
            *aim_estimated_slider_breaks,
            expected.aim_estimated_slider_breaks,
        );
        assert_eq_float(
            *speed_estimated_slider_breaks,
            expected.speed_estimated_slider_breaks,
        );
    }
}

impl AssertEq for TaikoPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_acc,
            pp_difficulty,
            estimated_unstable_rate,
        } = self;

        assert_eq_float(*pp, expected.pp);
        assert_eq_float(*pp_acc, expected.pp_acc);
        assert_eq_float(*pp_difficulty, expected.pp_difficulty);
        assert_eq_option(*estimated_unstable_rate, expected.estimated_unstable_rate);
    }
}

impl AssertEq for CatchPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self { difficulty: _, pp } = self;

        assert_eq_float(*pp, expected.pp);
    }
}

impl AssertEq for ManiaPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_difficulty,
        } = self;

        assert_eq_float(*pp_difficulty, expected.pp_difficulty);
        assert_eq_float(*pp, expected.pp);
    }
}

/// Bit-exact performance-attribute parity across 18 diverse osu!standard maps
/// (lazer, full-combo SS, 0 misses, 100% acc) x 7 mod combos, against C# refs.
/// Regenerate refs: `python scripts/gen_ext_refs.py scripts/manifest.txt tests/data/ext_refs.rs`.
#[test]
fn ext_osu() {
    for ext in EXT_REFS {
        let map = Beatmap::from_path(ext.path).unwrap();
        let attrs = OsuPerformance::from(&map)
            .lazer(true)
            .mods(ext.mods)
            .calculate()
            .unwrap();
        for (fname, val) in ext.performance {
            let cv: Option<f64> = val.map(|s| s.parse().unwrap());
            let rv = perf_opt_val(&attrs, fname);
            match (rv, cv) {
                (None, None) => {}
                (Some(a), Some(b)) => assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "{}/{}: rs={a} cs={b}",
                    ext.path,
                    fname
                ),
                _ => panic!("{}/{}: rs={rv:?} cs={cv:?}", ext.path, fname),
            }
        }
    }
}

/// Bit-exact performance-attribute parity for non-full-combo lazer plays
/// (o/m/miss > 0, combo < max combo, accuracy < 100%) across the same 18 maps x
/// 7 mod combos as `ext_osu`. Each case pins the exact hit-result counts; both
/// sides derive the accuracy from them, so the miss/combo/acc-dependent paths
/// (`effective_miss_count`, `combo_based_estimated_miss_count`, the aim/speed
/// miss penalties, `speed_deviation`, the accuracy pp-component and the
/// flashlight combo scaling) all run and are compared bit-exactly.
/// Regenerate refs: `python scripts/gen_ext_acc_refs.py scripts/manifest.txt tests/data/ext_acc_refs.rs`.
#[test]
fn ext_acc_osu() {
    for ext in EXT_ACC_REFS {
        let map = Beatmap::from_path(ext.path).unwrap();
        let attrs = OsuPerformance::from(&map)
            .lazer(true)
            .mods(ext.mods)
            .n300(ext.n300)
            .n100(ext.n100)
            .n50(ext.n50)
            .misses(ext.misses)
            .large_tick_hits(ext.large_tick_hits)
            .slider_end_hits(ext.slider_end_hits)
            .combo(ext.combo)
            .calculate()
            .unwrap();
        for (fname, val) in ext.performance {
            let cv: Option<f64> = val.map(|s| s.parse().unwrap());
            let rv = perf_opt_val(&attrs, fname);
            match (rv, cv) {
                (None, None) => {}
                (Some(a), Some(b)) => assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "{}/{}: rs={a} cs={b}",
                    ext.path,
                    fname
                ),
                _ => panic!("{}/{}: rs={rv:?} cs={cv:?}", ext.path, fname),
            }
        }
    }
}

/// Map a C# JSON performance key to the Rust `OsuPerformanceAttributes` value,
/// wrapped in `Option` so `None` refs (e.g. `score_based_estimated_miss_count`)
/// can be compared uniformly.
fn perf_opt_val(attrs: &OsuPerformanceAttributes, fname: &str) -> Option<f64> {
    match fname {
        "pp" => Some(attrs.pp),
        "accuracy" => Some(attrs.pp_acc),
        "aim" => Some(attrs.pp_aim),
        "flashlight" => Some(attrs.pp_flashlight),
        "speed" => Some(attrs.pp_speed),
        "reading" => Some(attrs.pp_reading),
        "effective_miss_count" => Some(attrs.effective_miss_count),
        "speed_deviation" => attrs.speed_deviation,
        "combo_based_estimated_miss_count" => Some(attrs.combo_based_estimated_miss_count),
        "score_based_estimated_miss_count" => attrs.score_based_estimated_miss_count,
        "aim_estimated_slider_breaks" => Some(attrs.aim_estimated_slider_breaks),
        "speed_estimated_slider_breaks" => Some(attrs.speed_estimated_slider_breaks),
        _ => panic!("unknown performance field {fname}"),
    }
}
