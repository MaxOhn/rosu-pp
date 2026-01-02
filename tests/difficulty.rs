use std::panic::{self, UnwindSafe};

use rosu_pp::{
    Beatmap, Difficulty,
    catch::{Catch, CatchDifficultyAttributes},
    mania::{Mania, ManiaDifficultyAttributes},
    osu::{Osu, OsuDifficultyAttributes},
    taiko::{Taiko, TaikoDifficultyAttributes},
};

use self::common::*;

mod common;

macro_rules! test_cases {
    ( $mode:ident: $path:ident {
        $( $( $mods:ident )+ => {
            $( $key:ident: $value:literal $( , )? )*
        } $( ; )? )*
    } ) => {
        let map = Beatmap::from_path(common::$path).unwrap();

        $(
            let mods = 0 $( + $mods )*;
            let expected = test_cases!(@$mode { $( $key: $value, )* });

            let actual = Difficulty::new()
                .mods(mods)
                .calculate_for_mode::<$mode>(&map)
                .unwrap();

            run(&actual, &expected, mods);
        )*
    };
    ( @Osu {
        aim: $aim:literal,
        aim_difficult_slider_count: $aim_difficult_slider_count:literal,
        speed: $speed:literal,
        flashlight: $flashlight:literal,
        slider_factor: $slider_factor:literal,
        aim_top_weighted_slider_factor: $aim_top_weighted_slider_factor:literal,
        speed_top_weighted_slider_factor: $speed_top_weighted_slider_factor:literal,
        speed_note_count: $speed_note_count:literal,
        aim_difficult_strain_count: $aim_difficult_strain_count:literal,
        speed_difficult_strain_count: $speed_difficult_strain_count:literal,
        nested_score_per_object: $nested_score_per_object:literal,
        legacy_score_base_multiplier: $legacy_score_base_multiplier:literal,
        maximum_legacy_combo_score: $maximum_legacy_combo_score:literal,
        ar: $ar:literal,
        great_hit_window: $great_hit_window:literal,
        ok_hit_window: $ok_hit_window:literal,
        meh_hit_window: $meh_hit_window:literal,
        hp: $hp:literal,
        n_circles: $n_circles:literal,
        n_sliders: $n_sliders:literal,
        n_large_ticks: $n_large_ticks:literal,
        n_spinners: $n_spinners:literal,
        stars: $stars:literal,
        max_combo: $max_combo:literal,
    }) => {
        OsuDifficultyAttributes {
            aim: $aim,
            aim_difficult_slider_count: $aim_difficult_slider_count,
            speed: $speed,
            flashlight: $flashlight,
            slider_factor: $slider_factor,
            aim_top_weighted_slider_factor: $aim_top_weighted_slider_factor,
            speed_top_weighted_slider_factor: $speed_top_weighted_slider_factor,
            speed_note_count: $speed_note_count,
            aim_difficult_strain_count: $aim_difficult_strain_count,
            speed_difficult_strain_count: $speed_difficult_strain_count,
            nested_score_per_object: $nested_score_per_object,
            legacy_score_base_multiplier: $legacy_score_base_multiplier,
            maximum_legacy_combo_score: $maximum_legacy_combo_score,
            ar: $ar,
            great_hit_window: $great_hit_window,
            ok_hit_window: $ok_hit_window,
            meh_hit_window: $meh_hit_window,
            hp: $hp,
            n_circles: $n_circles,
            n_sliders: $n_sliders,
            n_large_ticks: $n_large_ticks,
            n_spinners: $n_spinners,
            stars: $stars,
            max_combo: $max_combo,
        }
    };
    ( @Taiko {
        stamina: $stamina:literal,
        rhythm: $rhythm:literal,
        color: $color:literal,
        reading: $reading:literal,
        great_hit_window: $great_hit_window:literal,
        ok_hit_window: $ok_hit_window:literal,
        mono_stamina_factor: $mono_stamina_factor:literal,
        stars: $stars:literal,
        max_combo: $max_combo:literal,
        is_convert: $is_convert:literal,
    }) => {
        TaikoDifficultyAttributes {
            stamina: $stamina,
            rhythm: $rhythm,
            color: $color,
            reading: $reading,
            great_hit_window: $great_hit_window,
            ok_hit_window: $ok_hit_window,
            mono_stamina_factor: $mono_stamina_factor,
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
        n_objects: $n_objects:literal,
        n_hold_notes: $n_hold_notes:literal,
        max_combo: $max_combo:literal,
        is_convert: $is_convert:literal,
    }) => {
        ManiaDifficultyAttributes {
            stars: $stars,
            n_objects: $n_objects,
            n_hold_notes: $n_hold_notes,
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
                aim: 3.021506412510076,
                aim_difficult_slider_count: 180.33980678704012,
                speed: 2.5263145770639976,
                flashlight: 0.0,
                slider_factor: 0.9847225384137204,
                aim_top_weighted_slider_factor: 1.3996332540321264,
                speed_top_weighted_slider_factor: 0.6014562852677632,
                speed_note_count: 202.24319351543616,
                aim_difficult_strain_count: 108.47555309841259,
                speed_difficult_strain_count: 78.39830024782772,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 5.740766046562339,
                max_combo: 909,
            };
            HD => {
                aim: 3.121489829231887,
                aim_difficult_slider_count: 180.33980678704012,
                speed: 2.614171127905441,
                flashlight: 0.0,
                slider_factor: 0.9847225384137204,
                aim_top_weighted_slider_factor: 1.3996332540321264,
                speed_top_weighted_slider_factor: 0.6014562852677632,
                speed_note_count: 202.24319351543616,
                aim_difficult_strain_count: 108.47555309841259,
                speed_difficult_strain_count: 78.39830024782772,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 5.934133851244851,
                max_combo: 909,
            };
            HR => {
                aim: 3.4309052630747257,
                aim_difficult_slider_count: 187.20300643263465,
                speed: 2.6813963801152716,
                flashlight: 0.0,
                slider_factor: 0.9748562752795166,
                aim_top_weighted_slider_factor: 1.3634873114118244,
                speed_top_weighted_slider_factor: 0.6668815233244475,
                speed_note_count: 185.01178339020348,
                aim_difficult_strain_count: 112.28112750203013,
                speed_difficult_strain_count: 74.53251006179151,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 10.0,
                great_hit_window: 19.5,
                ok_hit_window: 59.5,
                meh_hit_window: 99.5,
                hp: 7.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.375104448752039,
                max_combo: 909,
            };
            DT => {
                aim: 4.3662195513104525,
                aim_difficult_slider_count: 195.41476682131653,
                speed:  3.7793477426295814,
                flashlight: 0.0,
                slider_factor: 0.9787310737204966,
                aim_top_weighted_slider_factor: 1.3819099517666353,
                speed_top_weighted_slider_factor: 0.6923456235877925,
                speed_note_count: 208.98215163620375,
                aim_difficult_strain_count: 130.48279566301667,
                speed_difficult_strain_count: 93.64469563382437,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 10.533333460489908,
                great_hit_window: 17.666666666666668,
                ok_hit_window: 45.666666666666664,
                meh_hit_window: 73.66666666666667,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 8.40182116136074,
                max_combo: 909,
            };
            FL => {
                aim: 3.021506412510076,
                aim_difficult_slider_count: 180.33980678704012,
                speed: 2.5263145770639976,
                flashlight: 2.3005989208967885,
                slider_factor: 0.9847225384137204,
                aim_top_weighted_slider_factor: 1.3996332540321264,
                speed_top_weighted_slider_factor: 0.6014562852677632,
                speed_note_count: 202.24319351543616,
                aim_difficult_strain_count: 108.47555309841259,
                speed_difficult_strain_count: 78.39830024782772,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.864308231959398,
                max_combo: 909,
            };
            HD FL => {
                aim: 3.121489829231887,
                aim_difficult_slider_count: 180.33980678704012,
                speed: 2.614171127905441,
                flashlight: 2.620335643475851,
                slider_factor: 0.9847225384137204,
                aim_top_weighted_slider_factor: 1.3996332540321264,
                speed_top_weighted_slider_factor: 0.6014562852677632,
                speed_note_count: 202.24319351543616,
                aim_difficult_strain_count: 108.47555309841259,
                speed_difficult_strain_count: 78.39830024782772,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.300000190734863,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 7.2736222258399374,
                max_combo: 909,
            };
        }
    };
}

#[test]
fn basic_taiko() {
    #[cfg(target_os = "windows")]
    test_cases! {
        Taiko: TAIKO {
            NM => {
                stamina: 1.6946519339726132,
                rhythm: 0.1469836127567273,
                color: 0.517224522408358,
                reading: 1.3847421923785603E-05,
                great_hit_window: 35.0,
                ok_hit_window: 80.0,
                mono_stamina_factor: 2.5849839992846003E-07,
                stars: 2.9052867123859096,
                max_combo: 289,
                is_convert: false,
            };
            HR => {
                stamina: 1.6946519339726132,
                rhythm: 0.1481975365980239,
                color: 0.517224522408358,
                reading: 0.48269837569606633,
                great_hit_window: 29.0,
                ok_hit_window: 68.0,
                mono_stamina_factor: 2.5849839992846003E-07,
                stars: 2.987035945441977,
                max_combo: 289,
                is_convert: false,
            };
            DT => {
                stamina: 2.469654573687006,
                rhythm: 0.49890717637373116,
                color: 0.6582785829498352,
                reading: 0.18343567306243425,
                great_hit_window: 23.333333333333332,
                ok_hit_window: 53.333333333333336,
                mono_stamina_factor: 2.464552111442399E-07,
                stars: 4.032774431778195,
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
                stamina: 3.6244157068104066,
                rhythm: 1.1790872487169137,
                color: 1.3884965692008842,
                reading: 1.7629163125391505,
                great_hit_window: 23.59999942779541,
                ok_hit_window: 57.19999885559082,
                mono_stamina_factor: 0.0014311041774359666,
                stars: 4.856701972823887,
                max_combo: 908,
                is_convert: true,
            };
            HR => {
                stamina: 3.6244157068104066,
                rhythm: 1.198436503252946,
                color: 1.3884965692008842,
                reading: 2.26853983796185,
                great_hit_window: 20.0,
                ok_hit_window: 50.0,
                mono_stamina_factor: 0.0014311041774359666,
                stars: 5.335018644413355,
                max_combo: 908,
                is_convert: true,
            };
            DT => {
                stamina: 5.506714422351734,
                rhythm: 1.8409407770037334,
                color: 1.8926518387821614,
                reading: 3.0585370998057053,
                great_hit_window: 15.733332951863607,
                ok_hit_window: 38.13333257039388,
                mono_stamina_factor: 0.0014418086037955797,
                stars: 7.1789968695409225,
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
                ar: 9.300000190734863,
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
                n_objects: 594,
                n_hold_notes: 121,
                max_combo: 956,
                is_convert: false,
            };
            DT => {
                stars: 4.6072892053157295,
                n_objects: 594,
                n_hold_notes: 121,
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
                n_objects: 1046,
                n_hold_notes: 266,
                max_combo: 1381,
                is_convert: true,
            };
            DT => {
                stars: 4.2934063021960185,
                n_objects: 1046,
                n_hold_notes: 266,
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
        let Self {
            aim,
            aim_difficult_slider_count,
            speed,
            flashlight,
            slider_factor,
            aim_top_weighted_slider_factor,
            speed_top_weighted_slider_factor,
            speed_note_count,
            aim_difficult_strain_count,
            speed_difficult_strain_count,
            nested_score_per_object,
            legacy_score_base_multiplier,
            maximum_legacy_combo_score,
            ar,
            great_hit_window,
            ok_hit_window,
            meh_hit_window,
            hp,
            n_circles,
            n_sliders,
            n_large_ticks,
            n_spinners,
            stars,
            max_combo,
        } = self;

        assert_eq_float(*aim, expected.aim);
        assert_eq_float(
            *aim_difficult_slider_count,
            expected.aim_difficult_slider_count,
        );
        assert_eq_float(*speed, expected.speed);
        assert_eq_float(*flashlight, expected.flashlight);
        assert_eq_float(*slider_factor, expected.slider_factor);
        assert_eq_float(
            *aim_top_weighted_slider_factor,
            expected.aim_top_weighted_slider_factor,
        );
        assert_eq_float(
            *speed_top_weighted_slider_factor,
            expected.speed_top_weighted_slider_factor,
        );
        assert_eq_float(*speed_note_count, expected.speed_note_count);
        assert_eq_float(
            *aim_difficult_strain_count,
            expected.aim_difficult_strain_count,
        );
        assert_eq_float(
            *speed_difficult_strain_count,
            expected.speed_difficult_strain_count,
        );
        assert_eq_float(*nested_score_per_object, expected.nested_score_per_object);
        assert_eq_float(
            *legacy_score_base_multiplier,
            expected.legacy_score_base_multiplier,
        );
        assert_eq_float(
            *maximum_legacy_combo_score,
            expected.maximum_legacy_combo_score,
        );
        assert_eq_float(*ar, expected.ar);
        assert_eq_float(*great_hit_window, expected.great_hit_window);
        assert_eq_float(*ok_hit_window, expected.ok_hit_window);
        assert_eq_float(*meh_hit_window, expected.meh_hit_window);
        assert_eq_float(*hp, expected.hp);
        assert_eq!(*n_circles, expected.n_circles);
        assert_eq!(*n_sliders, expected.n_sliders);
        assert_eq!(*n_large_ticks, expected.n_large_ticks);
        assert_eq!(*n_spinners, expected.n_spinners);
        assert_eq_float(*stars, expected.stars);
        assert_eq!(*max_combo, expected.max_combo);
    }
}

impl AssertEq for TaikoDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stamina,
            rhythm,
            color,
            reading,
            great_hit_window,
            ok_hit_window,
            mono_stamina_factor,
            stars,
            max_combo,
            is_convert,
        } = self;

        assert_eq_float(*stamina, expected.stamina);
        assert_eq_float(*rhythm, expected.rhythm);
        assert_eq_float(*color, expected.color);
        assert_eq_float(*reading, expected.reading);
        assert_eq_float(*great_hit_window, expected.great_hit_window);
        assert_eq_float(*ok_hit_window, expected.ok_hit_window);
        assert_eq_float(*mono_stamina_factor, expected.mono_stamina_factor);
        assert_eq_float(*stars, expected.stars);
        assert_eq!(*max_combo, expected.max_combo);
        assert_eq!(*is_convert, expected.is_convert);
    }
}

impl AssertEq for CatchDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stars,
            ar,
            n_fruits,
            n_droplets,
            n_tiny_droplets,
            is_convert,
        } = self;

        assert_eq_float(*stars, expected.stars);
        assert_eq_float(*ar, expected.ar);
        assert_eq!(*n_fruits, expected.n_fruits);
        assert_eq!(*n_droplets, expected.n_droplets);
        assert_eq!(*n_tiny_droplets, expected.n_tiny_droplets);
        assert_eq!(*is_convert, expected.is_convert);
    }
}

impl AssertEq for ManiaDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stars,
            n_objects,
            n_hold_notes,
            max_combo,
            is_convert,
        } = self;

        assert_eq_float(*stars, expected.stars);
        assert_eq!(*n_objects, expected.n_objects);
        assert_eq!(*n_hold_notes, expected.n_hold_notes);
        assert_eq!(*max_combo, expected.max_combo);
        assert_eq!(*is_convert, expected.is_convert);
    }
}
