use std::panic::{self, UnwindSafe};

use rosu_pp::{
    catch::{Catch, CatchDifficultyAttributes},
    mania::{Mania, ManiaDifficultyAttributes},
    osu::{Osu, OsuDifficultyAttributes},
    taiko::{Taiko, TaikoDifficultyAttributes},
    Beatmap, Difficulty,
};

use self::common::*;

mod common;

macro_rules! test_cases {
    ( $mode:ident: $path:ident {
        $( $( $mods:ident )+ => {
            $( $key:ident: $value:literal $( , )? )*
        } $( ; )? )*
    } ) => {
        let map = Beatmap::from_path(common::$path)
            .unwrap()
            .unchecked_into_converted::<$mode>();

        $(
            let mods = 0 $( + $mods )*;
            let expected = test_cases!(@$mode { $( $key: $value, )* });
            let actual = Difficulty::new().mods(mods).with_mode().calculate(&map);
            run(&actual, &expected, mods);
        )*
    };
    ( @Osu {
        aim: $aim:literal,
        speed: $speed:literal,
        flashlight: $flashlight:literal,
        slider_factor: $slider_factor:literal,
        speed_note_count: $speed_note_count:literal,
        aim_difficult_strain_count: $aim_difficult_strain_count:literal,
        speed_difficult_strain_count: $speed_difficult_strain_count:literal,
        ar: $ar:literal,
        od: $od:literal,
        hp: $hp:literal,
        n_circles: $n_circles:literal,
        n_sliders: $n_sliders:literal,
        n_slider_ticks: $n_slider_ticks:literal,
        n_spinners: $n_spinners:literal,
        stars: $stars:literal,
        max_combo: $max_combo:literal,
    }) => {
        OsuDifficultyAttributes {
            aim: $aim,
            speed: $speed,
            flashlight: $flashlight,
            slider_factor: $slider_factor,
            speed_note_count: $speed_note_count,
            aim_difficult_strain_count: $aim_difficult_strain_count,
            speed_difficult_strain_count: $speed_difficult_strain_count,
            ar: $ar,
            od: $od,
            hp: $hp,
            n_circles: $n_circles,
            n_sliders: $n_sliders,
            n_slider_ticks: $n_slider_ticks,
            n_spinners: $n_spinners,
            stars: $stars,
            max_combo: $max_combo,
        }
    };
    ( @Taiko {
        stamina: $stamina:literal,
        rhythm: $rhythm:literal,
        color: $color:literal,
        peak: $peak:literal,
        great_hit_window: $great_hit_window:literal,
        ok_hit_window: $ok_hit_window:literal,
        stars: $stars:literal,
        max_combo: $max_combo:literal,
        is_convert: $is_convert:literal,
    }) => {
        TaikoDifficultyAttributes {
            stamina: $stamina,
            rhythm: $rhythm,
            color: $color,
            peak: $peak,
            great_hit_window: $great_hit_window,
            ok_hit_window: $ok_hit_window,
            stars: $stars,
            max_combo: $max_combo,
            is_convert: $is_convert,
        }
    };
    ( @Catch {
        stars: $stars:literal,
        ar: $ar:literal,
        n_fruits: $n_fruits:literal,
        n_droplets: $n_droplets:literal,
        n_tiny_droplets: $n_tiny_droplets:literal,
        is_convert: $is_convert:literal,
    }) => {
        CatchDifficultyAttributes {
            stars: $stars,
            ar: $ar,
            n_fruits: $n_fruits,
            n_droplets: $n_droplets,
            n_tiny_droplets: $n_tiny_droplets,
            is_convert: $is_convert,
        }
    };
    ( @Mania {
        stars: $stars:literal,
        hit_window: $hit_window:literal,
        n_objects: $n_objects:literal,
        max_combo: $max_combo:literal,
        is_convert: $is_convert:literal,
    }) => {
        ManiaDifficultyAttributes {
            stars: $stars,
            hit_window: $hit_window,
            n_objects: $n_objects,
            max_combo: $max_combo,
            is_convert: $is_convert,
        }
    }
}

#[test]
fn basic_osu() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Osu: OSU {
            NM => {
                aim: 2.881184366758021,
                speed: 2.468469273849314,
                flashlight: 2.287888783550428,
                slider_factor: 0.9803293523973865,
                speed_note_count: 204.88794724609374,
                aim_difficult_strain_count: 106.63833474488378,
                speed_difficult_strain_count: 79.9883004295862,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_slider_ticks: 15,
                n_spinners: 1,
                stars: 5.643619989739299,
                max_combo: 909,
            };
            HD => {
                aim: 2.881184366758021,
                speed: 2.468469273849314,
                flashlight: 2.605859779358901,
                slider_factor: 0.9803293523973865,
                speed_note_count: 204.88794724609374,
                aim_difficult_strain_count: 106.63833474488378,
                speed_difficult_strain_count: 79.9883004295862,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_slider_ticks: 15,
                n_spinners: 1,
                stars: 5.643619989739299,
                max_combo: 909,
            };
            HR => {
                aim: 3.2515300463985666,
                speed: 2.6323568908654615,
                flashlight: 2.853761577136605,
                slider_factor: 0.969089944826546,
                speed_note_count: 178.52041495886283,
                aim_difficult_strain_count: 108.03970474535397,
                speed_difficult_strain_count: 73.27713411796513,
                ar: 10.0,
                od: 10.0,
                hp: 7.0,
                n_circles: 307,
                n_sliders: 293,
                n_slider_ticks: 15,
                n_spinners: 1,
                stars: 6.243301253337941,
                max_combo: 909,
            };
            DT => {
                aim: 4.058080039906945,
                speed: 3.570932204630734,
                flashlight: 3.318209122186825,
                slider_factor: 0.9777224379583133,
                speed_note_count: 211.29204189490912,
                aim_difficult_strain_count: 126.9561362975524,
                speed_difficult_strain_count: 95.63810649133869,
                ar: 10.53333346048991,
                od: 10.311111238267687,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_slider_ticks: 15,
                n_spinners: 1,
                stars: 8.030649319285482,
                max_combo: 909,
            };
            FL => {
                aim: 2.881184366758021,
                speed: 2.468469273849314,
                flashlight: 2.287888783550428,
                slider_factor: 0.9803293523973865,
                speed_note_count: 204.88794724609374,
                aim_difficult_strain_count: 106.63833474488378,
                speed_difficult_strain_count: 79.9883004295862,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_slider_ticks: 15,
                n_spinners: 1,
                stars: 6.858771801534423,
                max_combo: 909,
            };
            HD FL => {
                aim: 2.881184366758021,
                speed: 2.468469273849314,
                flashlight: 2.605859779358901,
                slider_factor: 0.9803293523973865,
                speed_note_count: 204.88794724609374,
                aim_difficult_strain_count: 106.63833474488378,
                speed_difficult_strain_count: 79.9883004295862,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_slider_ticks: 15,
                n_spinners: 1,
                stars: 7.167932950561898,
                max_combo: 909,
            };
        }
    };
    #[cfg(target_os = "linux")] // TODO
    test_cases! {
        Osu: OSU {
            NM => {
                aim: 2.8693628443424104,
                speed: 2.533869745015772,
                flashlight: 2.288770487900865,
                slider_factor: 0.9803052946037858,
                speed_note_count: 210.36373973116545,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 5.669858729379631,
                max_combo: 909,
            };
            HD => {
                aim: 2.8693628443424104,
                speed: 2.533869745015772,
                flashlight: 2.606877929965889,
                slider_factor: 0.9803052946037858,
                speed_note_count: 210.36373973116545,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 5.669858729379631,
                max_combo: 909,
            };
            HR => {
                aim: 3.2385394176190507,
                speed: 2.7009854505234308,
                flashlight: 2.8549217213059936,
                slider_factor: 0.9690667605258665,
                speed_note_count: 184.01205359079387,
                ar: 10.0,
                od: 10.0,
                hp: 7.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 6.263576582906263,
                max_combo: 909,
            };
            DT => {
                aim: 4.041442573946681,
                speed: 3.6784866216272474,
                flashlight: 3.319522943625448,
                slider_factor: 0.9776943279272041,
                speed_note_count: 214.80421464205617,
                ar: 10.53333346048991,
                od: 10.311111238267687,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 8.085307648397626,
                max_combo: 909,
            };
            FL => {
                aim: 2.8693628443424104,
                speed: 2.533869745015772,
                flashlight: 2.288770487900865,
                slider_factor: 0.9803052946037858,
                speed_note_count: 210.36373973116545,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 6.866778075388425,
                max_combo: 909,
            };
            HD FL => {
                aim: 2.8693628443424104,
                speed: 2.533869745015772,
                flashlight: 2.606877929965889,
                slider_factor: 0.9803052946037858,
                speed_note_count: 210.36373973116545,
                ar: 9.300000190734863,
                od: 8.800000190734863,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_spinners: 1,
                stars: 7.172580382476152,
                max_combo: 909,
            };
        }
    };
}

#[test]
fn basic_taiko() {
    test_cases! {
        Taiko: TAIKO {
            NM => {
                stamina: 1.3991746883284406,
                rhythm: 0.20130047251681948,
                color: 1.0487315549761433,
                peak: 1.8422453377400778,
                great_hit_window: 35.0,
                ok_hit_window: 80.0,
                stars: 2.914589700180437,
                max_combo: 289,
                is_convert: false,
            };
            HR => {
                stamina: 1.3991746883284406,
                rhythm: 0.20130047251681948,
                color: 1.0487315549761433,
                peak: 1.8422453377400778,
                great_hit_window: 29.0,
                ok_hit_window: 68.0,
                stars: 2.914589700180437,
                max_combo: 289,
                is_convert: false,
            };
            DT => {
                stamina: 2.0358868555131586,
                rhythm: 0.4448175371191029,
                color: 1.363762496098889,
                peak: 2.625066421324458,
                great_hit_window: 23.333333333333332,
                ok_hit_window: 53.333333333333336,
                stars: 3.942709244618132,
                max_combo: 289,
                is_convert: false,
            };
        }
    };
}

#[test]
fn convert_taiko() {
    test_cases! {
        Taiko: OSU {
            NM => {
                stamina: 2.9127139214411444,
                rhythm: 1.4696991260446617,
                color: 2.303228172964907,
                peak: 4.117779264387738,
                great_hit_window: 23.59999942779541,
                ok_hit_window: 57.19999885559082,
                stars: 5.660149021515273,
                max_combo: 908,
                is_convert: true,
            };
            HR => {
                stamina: 2.9127139214411444,
                rhythm: 1.4696991260446617,
                color: 2.303228172964907,
                peak: 4.117779264387738,
                great_hit_window: 20.0,
                ok_hit_window: 50.0,
                stars: 5.660149021515273,
                max_combo: 908,
                is_convert: true,
            };
            DT => {
                stamina: 4.379782453136822,
                rhythm: 2.002843919169095,
                color: 3.1864894777399986,
                peak: 6.103209631166694,
                great_hit_window: 15.733332951863607,
                ok_hit_window: 38.13333257039388,
                stars: 7.578560915020682,
                max_combo: 908,
                is_convert: true,
            };
        }
    };
}

#[test]
fn basic_catch() {
    test_cases! {
        Catch: CATCH {
            NM => {
                stars: 3.250266313373984,
                ar: 8.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            HR => {
                stars: 4.313360856186517,
                ar: 10.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            EZ => {
                stars: 4.06522224010957,
                ar: 4.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            DT => {
                stars: 4.635262826575386,
                ar: 9.666666666666668,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
        }
    };
}

#[test]
fn convert_catch() {
    test_cases! {
        Catch: OSU {
            NM => {
                stars: 4.528720977989276,
                ar: 9.300000190734863
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            HR => {
                stars: 5.076698043567007,
                ar: 10.0,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            EZ => {
                stars: 3.593264064535228,
                ar: 4.650000095367432,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            DT => {
                stars: 6.15540143757313,
                ar: 10.53333346048991,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
        }
    };
}

#[test]
fn basic_mania() {
    test_cases! {
        Mania: MANIA {
            NM => {
                stars: 3.358304846842773,
                hit_window: 40.0,
                n_objects: 594,
                max_combo: 956,
                is_convert: false,
            };
            DT => {
                stars: 4.6072892053157295,
                hit_window: 40.0,
                n_objects: 594,
                max_combo: 956,
                is_convert: false,
            };
        }
    };
}

#[test]
fn convert_mania() {
    test_cases! {
        Mania: OSU {
            NM => {
                stars: 3.2033142085672255,
                hit_window: 34.0,
                n_objects: 1046,
                max_combo: 1381,
                is_convert: true,
            };
            DT => {
                stars:  4.2934063021960185,
                hit_window: 34.0,
                n_objects: 1046,
                max_combo: 1381,
                is_convert: true,
            };
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

impl AssertEq for OsuDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.aim, expected.aim);
        assert_eq_float(self.speed, expected.speed);
        assert_eq_float(self.flashlight, expected.flashlight);
        assert_eq_float(self.slider_factor, expected.slider_factor);
        assert_eq_float(self.speed_note_count, expected.speed_note_count);
        assert_eq_float(self.ar, expected.ar);
        assert_eq_float(self.od, expected.od);
        assert_eq_float(self.hp, expected.hp);
        assert_eq!(self.n_circles, expected.n_circles);
        assert_eq!(self.n_sliders, expected.n_sliders);
        assert_eq!(self.n_spinners, expected.n_spinners);
        assert_eq_float(self.stars, expected.stars);
        assert_eq!(self.max_combo, expected.max_combo);
    }
}

impl AssertEq for TaikoDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.stamina, expected.stamina);
        assert_eq_float(self.rhythm, expected.rhythm);
        assert_eq_float(self.color, expected.color);
        assert_eq_float(self.peak, expected.peak);
        assert_eq_float(self.great_hit_window, expected.great_hit_window);
        assert_eq_float(self.ok_hit_window, expected.ok_hit_window);
        assert_eq_float(self.stars, expected.stars);
        assert_eq!(self.max_combo, expected.max_combo);
        assert_eq!(self.is_convert, expected.is_convert);
    }
}

impl AssertEq for CatchDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.stars, expected.stars);
        assert_eq_float(self.ar, expected.ar);
        assert_eq!(self.n_fruits, expected.n_fruits);
        assert_eq!(self.n_droplets, expected.n_droplets);
        assert_eq!(self.n_tiny_droplets, expected.n_tiny_droplets);
    }
}

impl AssertEq for ManiaDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        assert_eq_float(self.stars, expected.stars);
        assert_eq_float(self.hit_window, expected.hit_window);
        assert_eq!(self.n_objects, expected.n_objects);
        assert_eq!(self.max_combo, expected.max_combo);
        assert_eq!(self.is_convert, expected.is_convert);
    }
}
