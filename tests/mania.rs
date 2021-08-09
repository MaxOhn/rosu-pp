#![cfg(feature = "mania")]

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

fn mania_test(map: Beatmap, result: &MapResult) {
    let star_margin = 0.00001;
    let pp_margin = 0.00001;

    let MapResult {
        map_id,
        mods,
        stars,
        pp,
    } = result;

    let result = rosu_pp::ManiaPP::new(&map).mods(*mods).calculate();

    assert_result!("Stars" => result.stars(), star_margin, stars, map_id, mods);
    assert_result!("PP" => result.pp(), pp_margin, pp, map_id, mods);
}

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
#[test]
fn mania_sync() {
    for result in RESULTS {
        let file = match std::fs::File::open(format!("./maps/{}.osu", result.map_id)) {
            Ok(file) => file,
            Err(why) => panic!("Could not open file {}.osu: {}", result.map_id, why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map {}: {}", result.map_id, why),
        };

        mania_test(map, result);
    }
}

#[cfg(feature = "async_tokio")]
#[test]
fn mania_async_tokio() {
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

                mania_test(map, result);
            }
        });
}

#[cfg(feature = "async_std")]
#[test]
fn mania_async_std() {
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

            mania_test(map, result);
        }
    })
}

const RESULTS: &[MapResult] = &[
    MapResult {
        map_id: 1355822,
        mods: 256,
        stars: 2.2684169993943653,
        pp: 43.53325918930298,
    },
    MapResult {
        map_id: 1355822,
        mods: 0,
        stars: 2.7966565927524574,
        pp: 71.54271564752817,
    },
    MapResult {
        map_id: 1355822,
        mods: 64,
        stars: 3.7490047677821776,
        pp: 140.30066841634704,
    },
    // -----
    MapResult {
        map_id: 1974394,
        mods: 256,
        stars: 3.873141963622833,
        pp: 155.74720954392703,
    },
    MapResult {
        map_id: 1974394,
        mods: 0,
        stars: 4.801793001581714,
        pp: 254.50572065264748,
    },
    MapResult {
        map_id: 1974394,
        mods: 64,
        stars: 6.518026726802542,
        pp: 508.5082744648102,
    },
    // -----
    MapResult {
        map_id: 992512,
        mods: 256,
        stars: 5.282601951185166,
        pp: 316.1318109880581,
    },
    MapResult {
        map_id: 992512,
        mods: 0,
        stars: 6.536292432114728,
        pp: 511.72773348069154,
    },
    MapResult {
        map_id: 992512,
        mods: 64,
        stars: 8.974411542576131,
        pp: 1043.3119173151915,
    },
];
