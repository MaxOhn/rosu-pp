use std::panic::{self, UnwindSafe};

use rosu_pp::{
    catch::{CatchPerformance, CatchPerformanceAttributes},
    mania::{ManiaPerformance, ManiaPerformanceAttributes},
    osu::{OsuPerformance, OsuPerformanceAttributes},
    taiko::{TaikoPerformance, TaikoPerformanceAttributes},
    Beatmap,
};

use self::common::*;

mod common;

macro_rules! test_cases {
    ( $mode:ident: $path:ident {
        $( $( $mods:ident )+ => {
            $( $key:ident: $value:expr $( , )? )*
        } ;)*
    } ) => {
        let map = Beatmap::from_path(common::$path)
            .unwrap()
            .unchecked_into_converted();

        $(
            let mods = 0 $( + $mods )*;
            let (calc, expected) = test_cases!(@$mode { map, $( $key: $value, )* });
            let actual = calc.mods(mods).calculate();
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
    }) => {
        (
            OsuPerformance::from($map.as_owned()).lazer(true),
            OsuPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_aim: $pp_aim,
                pp_flashlight: $pp_flashlight,
                pp_speed: $pp_speed,
                effective_miss_count: $effective_miss_count,
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
            TaikoPerformance::from($map.as_owned()),
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
            CatchPerformance::from($map.as_owned()),
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
            ManiaPerformance::from($map.as_owned()),
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
    test_cases! {
        Osu: OSU {
            NM => {
                pp: 272.6047426867276,
                pp_acc: 97.62287463107766,
                pp_aim: 99.3726518686143,
                pp_flashlight: 0.0,
                pp_speed: 64.48542022217285,
                effective_miss_count: 0.0,
            };
            HD => {
                pp: 299.17174736245374,
                pp_acc: 105.43270460156388,
                pp_aim: 110.10489751227146,
                pp_flashlight: 0.0,
                pp_speed: 71.4498451141828,
                effective_miss_count: 0.0,
            };
            EZ HD => {
                pp: 186.7137498214991,
                pp_acc: 16.6270597231239,
                pp_aim: 98.11121656070222,
                pp_flashlight: 0.0,
                pp_speed: 61.51901495973101,
                effective_miss_count: 0.0,
            };
            HR => {
                pp: 404.7030358947424,
                pp_acc: 161.55575439788055,
                pp_aim: 145.04665418031985,
                pp_flashlight: 0.0,
                pp_speed: 80.77088499277514,
                effective_miss_count: 0.0,
            };
            DT => {
                pp: 738.7899608061098,
                pp_acc: 184.09450675506795,
                pp_aim: 304.16666833057235,
                pp_flashlight: 0.0,
                pp_speed: 220.06297202966698,
                effective_miss_count: 0.0,
            };
            FL => {
                pp: 402.408877784248,
                pp_acc: 99.57533212369923,
                pp_aim: 99.3726518686143,
                pp_flashlight: 132.29720631068272,
                pp_speed: 64.48542022217285,
                effective_miss_count: 0.0,
            };
            HD FL => {
                pp: 469.3245236137446,
                pp_acc: 107.54135869359516,
                pp_aim: 110.10489751227146,
                pp_flashlight: 171.62594459401154,
                pp_speed: 71.4498451141828,
                effective_miss_count: 0.0,
            };
        }
    };
}

#[test]
fn basic_taiko() {
    test_cases! {
        Taiko: TAIKO {
            NM => {
                pp: 117.93083232512124,
                pp_acc: 67.10083752258917,
                pp_difficulty: 43.804435430934774,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(148.44150180469418),
            };
            HD => {
                pp: 127.99624094636974,
                pp_acc: 67.10083752258917,
                pp_difficulty: 44.89954631670814,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(148.44150180469418),
            };
            HR => {
                pp: 139.75239372681187,
                pp_acc: 82.52109686788792,
                pp_difficulty: 48.75926757049594,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(122.99438720960376),
            };
            DT => {
                pp: 220.51543873147975,
                pp_acc: 118.28107309573312,
                pp_difficulty: 89.35584221033577,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(98.96100120312946),
            };
        }
    };
}

#[test]
fn convert_taiko() {
    test_cases! {
        Taiko: OSU {
            NM => {
                pp: 396.36982258196866,
                pp_acc: 160.00481201044695,
                pp_difficulty: 213.19920144243838,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(85.75868894575865),
            };
            HD => {
                pp: 426.0975592756163,
                pp_acc: 160.00481201044695,
                pp_difficulty: 213.19920144243838,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(85.75868894575865),
            };
            HR => {
                pp: 452.71458235192836,
                pp_acc: 191.95668459371925,
                pp_difficulty: 234.5205569790155,
                effective_miss_count: 0.0,
                estimated_unstable_rate: Some(72.67685680089848),
            };
            DT => {
                pp: 739.7393581199891,
                pp_acc: 280.8904545747157,
                pp_difficulty: 415.0249135067657,
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
            NM => { pp: 108.08430593303873, pp_difficulty: 108.08430593303873 };
            EZ => { pp: 54.04215296651937, pp_difficulty: 108.08430593303873 };
            DT => { pp: 222.79838979800365, pp_difficulty: 222.79838979800365 };
        }
    };
}

#[test]
fn convert_mania() {
    test_cases! {
        Mania: OSU {
            NM => { pp: 99.73849552661329, pp_difficulty: 99.73849552661329 };
            EZ => { pp: 49.869247763306646, pp_difficulty: 99.73849552661329 };
            DT => { pp: 195.23247718805612, pp_difficulty: 195.23247718805612 };
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
        assert_eq_float(self.pp, expected.pp);
        assert_eq_float(self.pp_acc, expected.pp_acc);
        assert_eq_float(self.pp_aim, expected.pp_aim);
        assert_eq_float(self.pp_flashlight, expected.pp_flashlight);
        assert_eq_float(self.pp_speed, expected.pp_speed);
        assert_eq_float(self.effective_miss_count, expected.effective_miss_count);
    }
}

impl AssertEq for TaikoPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.pp, expected.pp);
        assert_eq_float(self.pp_acc, expected.pp_acc);
        assert_eq_float(self.pp_difficulty, expected.pp_difficulty);
        assert_eq_float(self.effective_miss_count, expected.effective_miss_count);
    }
}

impl AssertEq for CatchPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.pp, expected.pp);
    }
}

impl AssertEq for ManiaPerformanceAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.pp_difficulty, expected.pp_difficulty);
        assert_eq_float(self.pp, expected.pp);
    }
}
