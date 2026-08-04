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
        reading: $reading:literal,
        slider_factor: $slider_factor:literal,
        aim_top_weighted_slider_factor: $aim_top_weighted_slider_factor:literal,
        speed_top_weighted_slider_factor: $speed_top_weighted_slider_factor:literal,
        speed_note_count: $speed_note_count:literal,
        reading_difficult_note_count: $reading_difficult_note_count:literal,
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
            reading: $reading,
            slider_factor: $slider_factor,
            aim_top_weighted_slider_factor: $aim_top_weighted_slider_factor,
            speed_top_weighted_slider_factor: $speed_top_weighted_slider_factor,
            speed_note_count: $speed_note_count,
            reading_difficult_note_count: $reading_difficult_note_count,
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
        mechanical_difficulty: $mechanical_difficulty:literal,
        consistency_factor: $consistency_factor:literal,
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
            mechanical_difficulty: $mechanical_difficulty,
            consistency_factor: $consistency_factor,
            stars: $stars,
            max_combo: $max_combo,
            is_convert: $is_convert,
        }
    };
    ( @Catch {
        stars: $stars:literal,
        preempt: $preempt:literal,
        n_fruits: $n_fruits:literal,
        n_droplets: $n_droplets:literal,
        n_tiny_droplets: $n_tiny_droplets:literal,
        is_convert: $is_convert:literal,
    }) => {
        CatchDifficultyAttributes {
            stars: $stars,
            preempt: $preempt,
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
    test_cases! {
        Osu: OSU {
            NM => {
                aim: 3.27863857424994,
                aim_difficult_slider_count: 192.5269999738169,
                speed: 2.4917265153109014,
                flashlight: 0.0,
                reading: 0.8229208521405954,
                slider_factor: 0.9630386892765709,
                aim_top_weighted_slider_factor: 1.524202370856421,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                reading_difficult_note_count: 34.92251595856365,
                aim_difficult_strain_count: 124.69544446818438,
                speed_difficult_strain_count: 81.74921671931915,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 5.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 9.30000019,
                great_hit_window: 26.5,
                ok_hit_window: 68.5,
                meh_hit_window: 110.5,
                hp: 5.0,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.004027372552197,
                max_combo: 909,
            };
            HD => {
                aim: 3.27863857424994,
                aim_difficult_slider_count: 192.5269999738169,
                speed: 2.4917265153109014,
                flashlight: 0.0,
                reading: 2.1827264342501156,
                slider_factor: 0.963038689276571,
                aim_top_weighted_slider_factor: 1.524202370856421,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                reading_difficult_note_count: 135.7746987103988,
                aim_difficult_strain_count: 124.69544446818438,
                speed_difficult_strain_count: 81.74921671931915,
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
                stars: 6.308336834596954,
                max_combo: 909,
            };
            HR => {
                aim: 3.799232322249643,
                aim_difficult_slider_count: 191.8309507640488,
                speed: 2.4917265153109014,
                flashlight: 0.0,
                reading: 0.9144658746351508,
                slider_factor: 0.9475983634088616,
                aim_top_weighted_slider_factor: 1.510078933241033,
                speed_top_weighted_slider_factor: 0.5361916412312123,
                speed_note_count: 183.0639785973236,
                reading_difficult_note_count: 38.427842331296304,
                aim_difficult_strain_count: 119.62860170592188,
                speed_difficult_strain_count: 81.74921671931915,
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
                stars: 6.713673504226164,
                max_combo: 909,
            };
            DT => {
                aim: 4.693556954514378,
                aim_difficult_slider_count: 207.97415619378023,
                speed:  3.674242476685813,
                flashlight: 0.0,
                reading: 2.0228172499241897,
                slider_factor: 0.9674908735064722,
                aim_top_weighted_slider_factor: 1.476888448482664,
                speed_top_weighted_slider_factor: 0.6387657108874909,
                speed_note_count: 211.45478779956713,
                reading_difficult_note_count: 190.90786995621218,
                aim_difficult_strain_count: 144.31095827870723,
                speed_difficult_strain_count: 86.60969041721593,
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
                stars: 8.762958976840826,
                max_combo: 909,
            };
            FL => {
                aim: 3.27863857424994,
                aim_difficult_slider_count: 192.5269999738169,
                speed: 2.4917265153109014,
                flashlight: 2.345608737345168,
                reading: 0.8229208521405954,
                slider_factor: 0.963038689276571,
                aim_top_weighted_slider_factor: 1.524202370856421,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                reading_difficult_note_count: 34.92251595856365,
                aim_difficult_strain_count: 124.69544446818438,
                speed_difficult_strain_count: 81.74921671931915,
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
                stars: 7.036258413665859,
                max_combo: 909,
            };
            HD EZ => {
                aim: 2.7625488821040367,
                aim_difficult_slider_count: 196.89037873007746,
                speed: 2.3995360680924946,
                flashlight: 0.0,
                reading: 3.5538685094268883,
                slider_factor: 0.9796909646357417,
                aim_top_weighted_slider_factor: 1.562739917685155,
                speed_top_weighted_slider_factor: 0.5369556982593612,
                speed_note_count: 192.2649456376246,
                reading_difficult_note_count: 124.55280093376417,
                aim_difficult_strain_count: 129.38126835796155,
                speed_difficult_strain_count: 84.40439036292067,
                nested_score_per_object: 34.991680532445926,
                legacy_score_base_multiplier: 3.0,
                maximum_legacy_combo_score: 15729840.0,
                ar: 4.650000095367432,
                great_hit_window: 52.5,
                ok_hit_window: 103.5,
                meh_hit_window: 154.5,
                hp: 2.5,
                n_circles: 307,
                n_sliders: 293,
                n_large_ticks: 15,
                n_spinners: 1,
                stars: 6.891768477382506,
                max_combo: 909,
            };
            HD FL => {
                aim: 3.27863857424994,
                aim_difficult_slider_count: 192.5269999738169,
                speed: 2.4917265153109014,
                flashlight: 2.6270894946667935,
                reading: 2.1827264342501156,
                slider_factor: 0.963038689276571,
                aim_top_weighted_slider_factor: 1.524202370856421,
                speed_top_weighted_slider_factor: 0.536191641231213,
                speed_note_count: 183.0639785973236,
                reading_difficult_note_count: 135.7746987103988,
                aim_difficult_strain_count: 124.6954444681844,
                speed_difficult_strain_count: 81.74921671931915,
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
                stars: 7.47402836368438,
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
                stamina: 2.0538739969959194,
                rhythm: 0.20910773140367978,
                color: 0.6533063635604147,
                reading: 1.6783022238696096E-05,
                great_hit_window: 34.5,
                ok_hit_window: 79.5,
                mono_stamina_factor: 2.585220903145618E-07,
                mechanical_difficulty: 2.7071803605563343,
                consistency_factor: 0.6315890845261888,
                stars: 2.9163048749822527,
                max_combo: 289,
                is_convert: false,
            };
            HR => {
                stamina: 1.7575509868953703,
                rhythm: 0.1803685258404691,
                color: 0.5590504800684293,
                reading: 0.5006234843367281,
                great_hit_window: 28.5,
                ok_hit_window: 67.5,
                mono_stamina_factor: 2.585220903145618E-07,
                mechanical_difficulty: 2.3166014669637995,
                consistency_factor: 0.6323578625670226,
                stars: 2.9975934771409967,
                max_combo: 289,
                is_convert: false,
            };
            DT => {
                stamina: 2.5520061698054346,
                rhythm: 0.5994985831319557,
                color: 0.7173945446143202,
                reading: 0.18956996617034277,
                great_hit_window: 23.0,
                ok_hit_window: 53.0,
                mono_stamina_factor: 2.465693827167051E-07,
                mechanical_difficulty: 3.2694007144197546,
                consistency_factor: 0.621885489483516,
                stars: 4.058469263722054,
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
                stamina: 2.207066240409116,
                rhythm: 0.8365139147559882,
                color: 0.8396618490452487,
                reading: 1.0735173315689464,
                great_hit_window: 22.5,
                ok_hit_window: 56.5,
                mono_stamina_factor: 0.0014311041774359666,
                mechanical_difficulty: 3.0467280894543647,
                consistency_factor: 0.6655548318078143,
                stars: 4.9567593357792985,
                max_combo: 908,
                is_convert: true,
            };
            HR => {
                stamina: 2.2682938123290377,
                rhythm: 0.8719256263669988,
                color: 0.8629554209868424,
                reading: 1.4197363916621928,
                great_hit_window: 19.5,
                ok_hit_window: 49.5,
                mono_stamina_factor: 0.0014311041774359666,
                mechanical_difficulty: 3.13124923331588,
                consistency_factor: 0.6679579465054277,
                stars: 5.422911251345071,
                max_combo: 908,
                is_convert: true,
            };
            DT => {
                stamina: 3.210799549552373,
                rhythm: 1.2585331678081546,
                color: 1.0837796585509605,
                reading: 1.7833409886818568,
                great_hit_window: 15.0,
                ok_hit_window: 37.666666666666664,
                mono_stamina_factor: 0.0014418086037955797,
                mechanical_difficulty: 4.294579208103333,
                consistency_factor: 0.6621583643602745,
                stars: 7.336453364593345,
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
                preempt: 750.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            HR => {
                stars: 4.313360856186517,
                preempt: 450.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            EZ => {
                stars: 4.06522224010957,
                preempt: 1320.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            DT => {
                stars: 4.635262826575386,
                preempt: 500.0,
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
                preempt: 554.9999713897705,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            HR => {
                stars: 5.076698043567007,
                preempt: 450.0,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            EZ => {
                stars: 3.593264064535228,
                preempt: 1241.9999885559082,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            DT => {
                stars: 6.15540143757313,
                preempt: 369.9999809265137,
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
            reading,
            slider_factor,
            aim_top_weighted_slider_factor,
            speed_top_weighted_slider_factor,
            speed_note_count,
            reading_difficult_note_count,
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
        assert_eq_float(*reading, expected.reading);
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
            *reading_difficult_note_count,
            expected.reading_difficult_note_count,
        );
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
        assert_eq_float(*ar as f32, expected.ar as f32);
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
            mechanical_difficulty,
            consistency_factor,
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
        assert_eq_float(*mechanical_difficulty, expected.mechanical_difficulty);
        assert_eq_float(*consistency_factor, expected.consistency_factor);
        assert_eq_float(*stars, expected.stars);
        assert_eq!(*max_combo, expected.max_combo);
        assert_eq!(*is_convert, expected.is_convert);
    }
}

impl AssertEq for CatchDifficultyAttributes {
    fn assert_eq(&self, expected: &Self) {
        let Self {
            stars,
            preempt,
            n_fruits,
            n_droplets,
            n_tiny_droplets,
            is_convert,
        } = self;

        assert_eq_float(*stars, expected.stars);
        assert_eq_float(*preempt, expected.preempt);
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
