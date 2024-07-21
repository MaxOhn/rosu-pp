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
            $( $key:ident: $value:literal $( , )? )*
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
        pp: $pp:literal,
        pp_acc: $pp_acc:literal,
        pp_aim: $pp_aim:literal,
        pp_flashlight: $pp_flashlight:literal,
        pp_speed: $pp_speed:literal,
        effective_miss_count: $effective_miss_count:literal,
    }) => {
        (
            OsuPerformance::from($map.as_owned()),
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
        pp: $pp:literal,
        pp_acc: $pp_acc:literal,
        pp_difficulty: $pp_difficulty:literal,
        effective_miss_count: $effective_miss_count:literal,
    }) => {
        (
            TaikoPerformance::from($map.as_owned()),
            TaikoPerformanceAttributes {
                pp: $pp,
                pp_acc: $pp_acc,
                pp_difficulty: $pp_difficulty,
                effective_miss_count: $effective_miss_count,
                ..Default::default()
            },
        )
    };
    ( @Catch {
        $map:ident,
        pp: $pp:literal,
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
        pp: $pp:literal,
        pp_difficulty: $pp_difficulty:literal,
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
                pp: 255.9419635475736,
                pp_acc: 79.84500076626814,
                pp_aim: 98.13131344235279,
                pp_flashlight: 0.0,
                pp_speed: 69.86876965478146,
                effective_miss_count: 0.0,
            };
            HD => {
                pp: 281.28736211196446,
                pp_acc: 86.2326008275696,
                pp_aim: 108.72949454544438,
                pp_flashlight: 0.0,
                pp_speed: 77.41459624444144,
                effective_miss_count: 0.0,
            };
            EZ HD => {
                pp: 185.64881287702838,
                pp_acc: 13.59914468151693,
                pp_aim: 96.88083530160195,
                pp_flashlight: 0.0,
                pp_speed: 65.96268917477774,
                effective_miss_count: 0.0,
            };
            HR => {
                pp: 375.0764291059058,
                pp_acc: 132.13521300659738,
                pp_aim: 143.28598037767793,
                pp_flashlight: 0.0,
                pp_speed: 87.39375701955078,
                effective_miss_count: 0.0,
            };
            DT => {
                pp: 716.3683237855254,
                pp_acc: 150.5694857734174,
                pp_aim: 300.39084638572484,
                pp_flashlight: 0.0,
                pp_speed: 240.8765306794618,
                effective_miss_count: 0.0,
            };
            FL => {
                pp: 384.8917879591265,
                pp_acc: 81.4419007815935,
                pp_aim: 98.13131344235279,
                pp_flashlight: 132.3991950960219,
                pp_speed: 69.86876965478146,
                effective_miss_count: 0.0,
            };
            HD FL => {
                pp: 450.3709760368082,
                pp_acc: 87.95725284412099,
                pp_aim: 108.72949454544438,
                pp_flashlight: 171.7600847331662,
                pp_speed: 77.41459624444144,
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
                pp: 98.47602106219567,
                pp_acc: 46.11642717726248,
                pp_difficulty: 46.69844233558799,
                effective_miss_count: 0.0,
            };
            HD => {
                pp: 107.19493857245885,
                pp_acc: 46.11642717726248,
                pp_difficulty: 47.86590339397769,
                effective_miss_count: 0.0,
            };
            HR => {
                pp: 112.22705475791287,
                pp_acc: 56.71431676265808,
                pp_difficulty: 49.033364452367394,
                effective_miss_count: 0.0,
            };
            DT => {
                pp: 181.5021832786881,
                pp_acc: 80.74206626516394,
                pp_difficulty: 90.29961105452931,
                effective_miss_count: 0.0,
            };
        }
    };
}

#[test]
fn convert_taiko() {
    test_cases! {
        Taiko: OSU {
            NM => {
                pp: 324.23564627433217,
                pp_acc: 125.81086361861148,
                pp_difficulty: 179.31471072573842,
                effective_miss_count: 0.0,
            };
            HD => {
                pp: 353.7513933816713,
                pp_acc: 125.81086361861148,
                pp_difficulty: 183.79757849388187,
                effective_miss_count: 0.0,
            };
            HR => {
                pp: 360.12274556551137,
                pp_acc: 150.9344373000759,
                pp_difficulty: 188.28044626202535,
                effective_miss_count: 0.0,
            };
            DT => {
                pp: 604.8167434609272,
                pp_acc: 220.7055264311451,
                pp_difficulty: 347.90986552791844,
                effective_miss_count: 0.0,
            };
        }
    }
}

#[test]
fn basic_catch() {
    test_cases! {
        Catch: CATCH {
            NM => { pp: 113.85903714373049 };
            HD => { pp: 136.63084457247658 };
            HD HR => { pp: 231.90266535529486 };
            DT => { pp: 247.18402249125862 };
        }
    };
}

#[test]
fn convert_catch() {
    test_cases! {
        Catch: OSU {
            NM => { pp: 230.99937552589745 };
            HD => { pp: 254.6768082128294 };
            HD HR => { pp: 328.41201070443725 };
            DT => { pp: 500.4365349891725 };
        }
    };
}

#[test]
fn basic_mania() {
    test_cases! {
        Mania: MANIA {
            NM => { pp: 114.37175184134917, pp_difficulty: 14.296468980168646 };
            EZ => { pp: 57.18587592067458, pp_difficulty: 14.296468980168646 };
            DT => { pp: 233.17882161546717, pp_difficulty: 29.147352701933396 };
        }
    };
}

#[test]
fn convert_mania() {
    test_cases! {
        Mania: OSU {
            NM => { pp: 99.73849552661329, pp_difficulty: 12.467311940826661 };
            EZ => { pp: 49.869247763306646, pp_difficulty: 12.467311940826661 };
            DT => { pp: 195.23247718805612, pp_difficulty: 24.404059648507015 };
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
