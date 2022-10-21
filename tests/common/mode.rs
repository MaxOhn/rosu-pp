use rosu_pp::{
    catch::CatchDifficultyAttributes, mania::ManiaDifficultyAttributes,
    osu::OsuDifficultyAttributes, taiko::TaikoDifficultyAttributes,
};

pub struct Osu;
pub struct Taiko;
pub struct Catch;
pub struct Mania;

pub trait Mode {
    type DifficultyAttributes;

    const TEST_MAP_ID: u32;
    const TEST_DIFF_ATTRS: Self::DifficultyAttributes;
}

macro_rules! impl_mode {
    ( $( $mode:ident: $map_id:literal, $diff_attrs:ident $fields:tt; )* ) => {
        $(
            impl Mode for $mode {
                type DifficultyAttributes = $diff_attrs;

                const TEST_MAP_ID: u32 = $map_id;
                const TEST_DIFF_ATTRS: Self::DifficultyAttributes = $diff_attrs $fields;
            }
        )*
    };
}

impl_mode! {
    Osu: 2785319, OsuDifficultyAttributes {
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
        stars: 5.669858729379628,
        max_combo: 909,
    };
    Taiko: 1028484, TaikoDifficultyAttributes {
        stamina: 1.4528845068865617,
        rhythm: 0.20130047251681948,
        colour: 1.0487315549761433,
        peak: 1.8881824429738323,
        hit_window: 35.0,
        stars: 2.9778030386845606,
        max_combo: 289,
    };
    Catch: 2118524, CatchDifficultyAttributes {
        stars: 3.2502669316166624,
        ar: 8.0,
        n_fruits: 728,
        n_droplets: 2,
        n_tiny_droplets: 291,
    };
    Mania: 1974394, ManiaDifficultyAttributes {
        stars: 4.824631127426499,
        hit_window: 40.0,
        max_combo: 5064,
    };
}
