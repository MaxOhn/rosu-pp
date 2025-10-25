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
        effective_miss_count: $effective_miss_count:expr,
        speed_deviation: $speed_deviation:expr,
    }) => {
        (
            OsuPerformance::from(&$map).lazer(true),
            OsuPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_aim: $pp_aim,
                pp_flashlight: $pp_flashlight,
                pp_speed: $pp_speed,
                effective_miss_count: $effective_miss_count,
                speed_deviation: $speed_deviation,
                ..Default::default()
            },
        )
    };
    ( @Taiko {
        $map: ident,
        pp: $pp:expr,
        pp_acc: $pp_acc:expr,
        pp_difficulty: $pp_difficulty:expr,
        effective_miss_count: $effective_miss_count:expr,
        estimated_unstable_rate: $estimated_unstable_rate:expr,
    }) => {
        (
            TaikoPerformance::from(&$map),
            TaikoPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_difficulty: $pp_difficulty,
                effective_miss_count: $effective_miss_count,
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
fn basic_osu() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Osu: OSU {
            NM => {
                pp: 273.55482143018787,
                pp_acc: 97.62287463107766,
                pp_aim: 98.91907543994563,
                pp_flashlight: 0.0,
                pp_speed: 65.89857918351103,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.855079578025586),
            };
            HD => {
                pp: 300.22543969392876,
                pp_acc: 105.43270460156388,
                pp_aim: 109.6023348327671,
                pp_flashlight: 0.0,
                pp_speed: 73.01562523256396,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.855079578025586),
            };
            EZ HD => {
                pp: 186.89279040924507,
                pp_acc: 16.6270597231239,
                pp_aim: 96.66495037071935,
                pp_flashlight: 0.0,
                pp_speed: 63.20254195234833,
                effective_miss_count: 0.0,
                speed_deviation: Some(23.1539101317497),
            };
            HR => {
                pp: 405.7923033508039,
                pp_acc: 161.55575439788055,
                pp_aim: 145.16956208688748,
                pp_flashlight: 0.0,
                pp_speed: 81.7375074976892,
                effective_miss_count: 0.0,
                speed_deviation: Some(8.857953596258472),
            };
            DT => {
                pp: 741.3788121712137,
                pp_acc: 184.09450675506795,
                pp_aim: 301.88645138363296,
                pp_flashlight: 0.0,
                pp_speed: 224.91817628556774,
                effective_miss_count: 0.0,
                speed_deviation: Some(7.873979522967204),
            };
            FL => {
                pp: 403.31324405364177,
                pp_acc: 99.57533212369923,
                pp_aim: 98.91907543994563,
                pp_flashlight: 132.28811994208644,
                pp_speed: 65.89857918351103,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.855079578025586),
            };
            HD FL => {
                pp: 470.3193633451629,
                pp_acc: 107.54135869359516,
                pp_aim: 109.6023348327671,
                pp_flashlight: 171.61406165164135,
                pp_speed: 73.01562523256396,
                effective_miss_count: 0.0,
                speed_deviation: Some(11.855079578025586),
            };
        }
    };
}

#[test]
fn basic_taiko() {
    test_cases! {
        Taiko: TAIKO {
            NM => {
                pp: 104.65974235594882,
                pp_acc: 67.01508452097738,
                pp_difficulty: 30.951117266143964,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(148.44150180469418),
            };
            HD => {
                pp: 113.35231886537841,
                pp_acc: 67.01508452097738,
                pp_difficulty: 31.72489519779756,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(148.44150180469418),
            };
            HR => {
                pp: 125.39316057548226,
                pp_acc: 83.3355298805701,
                pp_difficulty: 33.77220597125385,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(122.99438720960376),
            };
            DT => {
                pp: 217.2255599983772,
                pp_acc: 119.35453575917016,
                pp_difficulty: 85.09547264616562,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(98.96100120312946),
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
                pp: 321.96508788209525,
                pp_acc: 150.50068595207387,
                pp_difficulty: 152.95500113793892,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(85.75868894575865),
            };
            HD => {
                pp: 326.0279405978374,
                pp_acc: 150.50068595207387,
                pp_difficulty: 156.7788761663874,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(85.75868894575865),
            };
            HR => {
                pp: 400.1259115798042,
                pp_acc: 187.46770845243455,
                pp_difficulty: 189.65602547641478,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(72.67685680089848),
            };
            DT => {
                pp: 688.6809319343615,
                pp_acc: 274.8702821415836,
                pp_difficulty: 373.46911205993484,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(57.17245929717244),
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
            pp_speed,
            effective_miss_count,
            speed_deviation,
        } = self;

        assert_eq_float(*pp, expected.pp);
        assert_eq_float(*pp_acc, expected.pp_acc);
        assert_eq_float(*pp_aim, expected.pp_aim);
        assert_eq_float(*pp_flashlight, expected.pp_flashlight);
        assert_eq_float(*pp_speed, expected.pp_speed);
        assert_eq_float(*effective_miss_count, expected.effective_miss_count);
        assert_eq_option(*speed_deviation, expected.speed_deviation);
    }
}

impl AssertEq for TaikoPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            difficulty: _,
            pp,
            pp_acc,
            pp_difficulty,
            effective_miss_count,
            estimated_unstable_rate,
        } = self;

        assert_eq_float(*pp, expected.pp);
        assert_eq_float(*pp_acc, expected.pp_acc);
        assert_eq_float(*pp_difficulty, expected.pp_difficulty);
        assert_eq_float(*effective_miss_count, expected.effective_miss_count);
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
