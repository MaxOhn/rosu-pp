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
        stars: 4.19951953364192,
        pp: 95.35544846090738,
    },
    MapResult {
        map_id: 1851299,
        mods: 0,
        stars: 5.305946555352317,
        pp: 188.7611225698759,
    },
    MapResult {
        map_id: 1851299,
        mods: 8,
        stars: 5.305946555352317,
        pp: 207.87782991080368,
    },
    MapResult {
        map_id: 1851299,
        mods: 64,
        stars: 7.352573837272898,
        pp: 465.60165096277717,
    },
    MapResult {
        map_id: 1851299,
        mods: 16,
        stars: 5.628029058321052,
        pp: 239.33966091681467,
    },
    MapResult {
        map_id: 1851299,
        mods: 2,
        stars: 4.892665488817249,
        pp: 108.66545494037493,
    },
    // -----
    MapResult {
        map_id: 70090,
        mods: 256,
        stars: 2.2531214736733975,
        pp: 17.064864347414005,
    },
    MapResult {
        map_id: 70090,
        mods: 0,
        stars: 2.7853401027561353,
        pp: 39.80360462535964,
    },
    MapResult {
        map_id: 70090,
        mods: 8,
        stars: 2.7853401027561353,
        pp: 45.27724541951056,
    },
    MapResult {
        map_id: 70090,
        mods: 64,
        stars: 3.7775299223395877,
        pp: 108.27132697867293,
    },
    MapResult {
        map_id: 70090,
        mods: 16,
        stars: 3.0128373294988626,
        pp: 83.70428900396428,
    },
    MapResult {
        map_id: 70090,
        mods: 2,
        stars: 2.673167484837261,
        pp: 21.254867316318986,
    },
    // -----
    MapResult {
        map_id: 1241370,
        mods: 256,
        stars: 5.558026586611704,
        pp: 334.51647189180727,
    },
    MapResult {
        map_id: 1241370,
        mods: 0,
        stars: 6.983848076867755,
        pp: 649.1653315906666,
    },
    MapResult {
        map_id: 1241370,
        mods: 8,
        stars: 6.983848076867755,
        pp: 710.594432790455,
    },
    MapResult {
        map_id: 1241370,
        mods: 64,
        stars: 11.076248857241552,
        pp: 2378.593642686663,
    },
    MapResult {
        map_id: 1241370,
        mods: 16,
        stars: 7.616427878599069,
        pp: 843.421001465897,
    },
    MapResult {
        map_id: 1241370,
        mods: 2,
        stars: 6.289772601786212,
        pp: 354.960619132034,
    },
];
