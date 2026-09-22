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

include!("data/ext_refs.rs");
include!("data/ext_refs_mania.rs");
include!("data/ext_refs_catch.rs");

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
    ( @OsuBasic {
        mods: $mods:expr,
        ar: $ar:literal,
        great_hit_window: $great_hit_window:literal,
        ok_hit_window: $ok_hit_window:literal,
        meh_hit_window: $meh_hit_window:literal,
        hp: $hp:literal,
        n_circles: $n_circles:literal,
        n_sliders: $n_sliders:literal,
        n_large_ticks: $n_large_ticks:literal,
        n_spinners: $n_spinners:literal,
    }) => {
        let map = Beatmap::from_path(common::OSU).unwrap();
        let actual = Difficulty::new().mods($mods).calculate_for_mode::<Osu>(&map).unwrap();
        assert_eq_float(actual.ar as f32, $ar as f32);
        assert_eq_float(actual.great_hit_window, $great_hit_window);
        assert_eq_float(actual.ok_hit_window, $ok_hit_window);
        assert_eq_float(actual.meh_hit_window, $meh_hit_window);
        assert_eq_float(actual.hp, $hp);
        assert_eq!(actual.n_circles, $n_circles);
        assert_eq!(actual.n_sliders, $n_sliders);
        assert_eq!(actual.n_large_ticks, $n_large_ticks);
        assert_eq!(actual.n_spinners, $n_spinners);
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
    // Only the fields NOT covered by ext_osu (hit windows, object counts, ar, hp).
    // The difficulty/PP values (aim/speed/reading/flashlight/stars) are gated by ext_osu.
    // ar/hp/hit-windows ARE mod-dependent, so all 7 mods are tested.
    test_cases! { @OsuBasic {
        mods: 0, // NM
        ar: 9.300000190734863, great_hit_window: 26.5, ok_hit_window: 68.5, meh_hit_window: 110.5, hp: 5.0,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
    test_cases! { @OsuBasic {
        mods: 8, // HD
        ar: 9.300000190734863, great_hit_window: 26.5, ok_hit_window: 68.5, meh_hit_window: 110.5, hp: 5.0,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
    test_cases! { @OsuBasic {
        mods: 16, // HR
        ar: 10.0, great_hit_window: 19.5, ok_hit_window: 59.5, meh_hit_window: 99.5, hp: 7.0,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
    test_cases! { @OsuBasic {
        mods: 64, // DT
        ar: 10.533333460489908, great_hit_window: 17.666666666666668, ok_hit_window: 45.666666666666664, meh_hit_window: 73.66666666666667, hp: 5.0,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
    test_cases! { @OsuBasic {
        mods: 1024, // FL
        ar: 9.300000190734863, great_hit_window: 26.5, ok_hit_window: 68.5, meh_hit_window: 110.5, hp: 5.0,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
    test_cases! { @OsuBasic {
        mods: 10, // HD EZ
        ar: 4.650000095367432, great_hit_window: 52.5, ok_hit_window: 103.5, meh_hit_window: 154.5, hp: 2.5,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
    test_cases! { @OsuBasic {
        mods: 1032, // HD FL
        ar: 9.300000190734863, great_hit_window: 26.5, ok_hit_window: 68.5, meh_hit_window: 110.5, hp: 5.0,
        n_circles: 307, n_sliders: 293, n_large_ticks: 15, n_spinners: 1,
    } }
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
                stars: 3.2340182503279706,
                preempt: 750.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            HR => {
                stars: 4.308291009137178,
                preempt: 450.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            EZ => {
                stars: 4.059198145823293,
                preempt: 1320.0,
                n_fruits: 728,
                n_droplets: 2,
                n_tiny_droplets: 263,
                is_convert: false,
            };
            DT => {
                stars: 4.6192881825873275,
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
                stars: 4.526991300645072,
                preempt: 554.9999713897705,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            HR => {
                stars: 5.0738627744810545,
                preempt: 450.0,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            EZ => {
                stars: 3.590187752268528,
                preempt: 1241.9999885559082,
                n_fruits: 908,
                n_droplets: 0,
                n_tiny_droplets: 159,
                is_convert: true,
            };
            DT => {
                stars: 6.151552522578919,
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

/// Bit-exact difficulty-attribute parity across 18 diverse osu!standard maps
/// (low OD<5, rate-change, spinner-heavy) x 7 mod combos, against C# refs.
/// Regenerate refs: `python scripts/gen_ext_refs.py scripts/manifest.txt tests/data/ext_refs.rs`.
#[test]
fn ext_osu() {
    for ext in EXT_REFS {
        let map = Beatmap::from_path(ext.path).unwrap();
        let attrs = Difficulty::new()
            .mods(ext.mods)
            .calculate_for_mode::<Osu>(&map)
            .unwrap();
        for &(fname, fstr) in ext.difficulty {
            let rv = attr_val(&attrs, fname);
            let cv: f64 = fstr.parse().unwrap();
            assert_eq!(
                rv.to_bits(),
                cv.to_bits(),
                "{}/{}: rs={rv} cs={fstr}",
                ext.path,
                fname
            );
        }
    }
}

/// Bit-exact difficulty-attribute parity (star rating + max combo) across 14
/// diverse mania maps (2K-18K columns, hold-heavy, rate-change) x 7 mod combos,
/// against C# refs.
/// Regenerate refs: `python scripts/gen_ext_refs.py mania scripts/manifest_mania.txt tests/data/ext_refs_mania.rs`.
#[test]
fn ext_mania() {
    for ext in MANIA_EXT_REFS {
        let map = Beatmap::from_path(ext.path).unwrap();
        let attrs = Difficulty::new()
            .mods(ext.mods)
            .calculate_for_mode::<Mania>(&map)
            .unwrap();
        for &(fname, fstr) in ext.difficulty {
            let rv = match fname {
                "star_rating" => attrs.stars,
                "max_combo" => attrs.max_combo as f64,
                _ => panic!("unknown difficulty field {fname}"),
            };
            let cv: f64 = fstr.parse().unwrap();
            assert_eq!(
                rv.to_bits(),
                cv.to_bits(),
                "{}/{}: rs={rv} cs={fstr}",
                ext.path,
                fname
            );
        }
    }
}

/// Bit-exact difficulty-attribute parity (star rating + max combo) across 13
/// diverse catch maps (CS 2.0-9.9, rate-change) x 7 mod combos, against C# refs.
/// Regenerate refs: `python scripts/gen_ext_refs.py catch scripts/manifest_catch.txt tests/data/ext_refs_catch.rs`.
#[test]
fn ext_catch() {
    for ext in CATCH_EXT_REFS {
        let map = Beatmap::from_path(ext.path).unwrap();
        let attrs = Difficulty::new()
            .mods(ext.mods)
            .calculate_for_mode::<Catch>(&map)
            .unwrap();
        for &(fname, fstr) in ext.difficulty {
            let rv = match fname {
                "star_rating" => attrs.stars,
                "max_combo" => attrs.max_combo() as f64,
                _ => panic!("unknown difficulty field {fname}"),
            };
            let cv: f64 = fstr.parse().unwrap();
            assert_eq!(
                rv.to_bits(),
                cv.to_bits(),
                "{}/{}: rs={rv} cs={fstr}",
                ext.path,
                fname
            );
        }
    }
}

fn attr_val(attrs: &OsuDifficultyAttributes, fname: &str) -> f64 {
    match fname {
        "star_rating" => attrs.stars,
        "max_combo" => attrs.max_combo as f64,
        "aim_difficulty" => attrs.aim,
        "aim_difficult_slider_count" => attrs.aim_difficult_slider_count,
        "speed_difficulty" => attrs.speed,
        "speed_note_count" => attrs.speed_note_count,
        "reading_difficulty" => attrs.reading,
        "slider_factor" => attrs.slider_factor,
        "aim_top_weighted_slider_factor" => attrs.aim_top_weighted_slider_factor,
        "speed_top_weighted_slider_factor" => attrs.speed_top_weighted_slider_factor,
        "aim_difficult_strain_count" => attrs.aim_difficult_strain_count,
        "speed_difficult_strain_count" => attrs.speed_difficult_strain_count,
        "flashlight_difficulty" => attrs.flashlight,
        "reading_difficult_note_count" => attrs.reading_difficult_note_count,
        "nested_score_per_object" => attrs.nested_score_per_object,
        "legacy_score_base_multiplier" => attrs.legacy_score_base_multiplier,
        "maximum_legacy_combo_score" => attrs.maximum_legacy_combo_score,
        _ => panic!("unknown difficulty field {fname}"),
    }
}
