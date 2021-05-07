#![cfg(feature = "osu")]

extern crate rosu_pp;

use rosu_pp::Beatmap;

struct MapResult {
    map_id: u32,
    mods: u32,
    stars: f32,
    pp: f32,
}

macro_rules! assert_result {
    ($kind:expr => $result:expr, $margin:expr, $expected:ident, $map_id:ident, $mods:ident) => {
        assert!(
            ($result - $expected).abs() < $margin * $expected,
            "\n{kind}:\n\
                Calculated: {calculated} | Expected: {expected}\n \
                => {margin} margin ({allowed} allowed)\n\
                [map {map} | mods {mods}]\n",
            kind = $kind,
            calculated = $result,
            expected = $expected,
            margin = ($result - $expected).abs(),
            allowed = $margin * $expected,
            map = $map_id,
            mods = $mods
        );
    };
}

fn margin() -> f32 {
    if cfg!(feature = "no_sliders_no_leniency") {
        0.0075
    } else if cfg!(feature = "no_leniency") {
        0.0025
    } else if cfg!(feature = "all_included") {
        0.001
    } else {
        unreachable!()
    }
}

fn osu_test(map: Beatmap, result: &MapResult) {
    let margin = margin();

    let star_margin = margin;
    let pp_margin = margin;

    let MapResult {
        map_id,
        mods,
        stars,
        pp,
    } = result;

    let result = rosu_pp::OsuPP::new(&map).mods(*mods).calculate();

    assert_result!("Stars" => result.stars(), star_margin, stars, map_id, mods);
    assert_result!("PP" => result.pp(), pp_margin, pp, map_id, mods);
}

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
#[test]
fn osu_sync() {
    for result in RESULTS {
        let file = match std::fs::File::open(format!("./maps/{}.osu", result.map_id)) {
            Ok(file) => file,
            Err(why) => panic!("Could not open file {}.osu: {}", result.map_id, why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map {}: {}", result.map_id, why),
        };

        osu_test(map, result);
    }
}

#[cfg(feature = "async_tokio")]
#[test]
fn osu_async_tokio() {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("could not start runtime")
        .block_on(async {
            for result in RESULTS {
                let file =
                    match tokio::fs::File::open(format!("./maps/{}.osu", result.map_id)).await {
                        Ok(file) => file,
                        Err(why) => panic!("Could not open file {}.osu: {}", result.map_id, why),
                    };

                let map = match Beatmap::parse(file).await {
                    Ok(map) => map,
                    Err(why) => panic!("Error while parsing map {}: {}", result.map_id, why),
                };

                osu_test(map, result);
            }
        });
}

#[cfg(feature = "async_std")]
#[test]
fn osu_async_std() {
    async_std::task::block_on(async {
        for result in RESULTS {
            let file =
                match async_std::fs::File::open(format!("./maps/{}.osu", result.map_id)).await {
                    Ok(file) => file,
                    Err(why) => panic!("Could not open file {}.osu: {}", result.map_id, why),
                };

            let map = match Beatmap::parse(file).await {
                Ok(map) => map,
                Err(why) => panic!("Error while parsing map {}: {}", result.map_id, why),
            };

            osu_test(map, result);
        }
    })
}

const RESULTS: &[MapResult] = &[
    MapResult {
        map_id: 1851299,
        mods: 256,
        stars: 4.23514130038547,
        pp: 96.59532540603362,
    },
    MapResult {
        map_id: 1851299,
        mods: 0,
        stars: 5.356786475158158,
        pp: 191.72186087468594,
    },
    MapResult {
        map_id: 1851299,
        mods: 8,
        stars: 5.356786475158158,
        pp: 211.17333597663404,
    },
    MapResult {
        map_id: 1851299,
        mods: 64,
        stars: 7.450616908751305,
        pp: 476.39199443787675,
    },
    MapResult {
        map_id: 1851299,
        mods: 16,
        stars: 5.6834681957637665,
        pp: 243.32730989490153,
    },
    MapResult {
        map_id: 1851299,
        mods: 2,
        stars: 4.937817303399699,
        pp: 110.44350158714633,
    },
    // -----
    MapResult {
        map_id: 70090,
        mods: 256,
        stars: 2.2929922580201803,
        pp: 17.530571082228978,
    },
    MapResult {
        map_id: 70090,
        mods: 0,
        stars: 2.8322940761833983,
        pp: 40.611760049886534,
    },
    MapResult {
        map_id: 70090,
        mods: 8,
        stars: 2.8322940761833983,
        pp: 46.252172598153074,
    },
    MapResult {
        map_id: 70090,
        mods: 64,
        stars: 3.8338563325375485,
        pp: 110.32389105793393,
    },
    MapResult {
        map_id: 70090,
        mods: 16,
        stars: 3.0617492228478174,
        pp: 84.67846960014381,
    },
    MapResult {
        map_id: 70090,
        mods: 2,
        stars: 2.698823231324141,
        pp: 21.861918597227252,
    },
    // -----
    MapResult {
        map_id: 1241370,
        mods: 256,
        stars: 5.662809600985943,
        pp: 346.1069865511771,
    },
    MapResult {
        map_id: 1241370,
        mods: 0,
        stars: 7.0367002127481975,
        pp: 658.4944314112954,
    },
    MapResult {
        map_id: 1241370,
        mods: 8,
        stars: 7.0367002127481975,
        pp: 720.8614284211293,
    },
    MapResult {
        map_id: 1241370,
        mods: 64,
        stars: 11.144720506574934,
        pp: 2414.665180655108,
    },
    MapResult {
        map_id: 1241370,
        mods: 16,
        stars: 7.641688110458715,
        pp: 853.7411405318841,
    },
    MapResult {
        map_id: 1241370,
        mods: 2,
        stars: 6.316288616688052,
        pp: 357.3089261481221,
    },
];
