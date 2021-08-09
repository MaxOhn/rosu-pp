#![cfg(feature = "taiko")]

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

fn taiko_test(map: Beatmap, result: &MapResult) {
    let star_margin = 0.0001;
    let pp_margin = 0.0001;

    let MapResult {
        map_id,
        mods,
        stars,
        pp,
    } = result;

    let result = rosu_pp::TaikoPP::new(&map).mods(*mods).calculate();

    assert_result!("Stars" => result.stars(), star_margin, stars, map_id, mods);
    assert_result!("PP" => result.pp(), pp_margin, pp, map_id, mods);
}

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
#[test]
fn taiko_sync() {
    for result in RESULTS {
        let file = match std::fs::File::open(format!("./maps/{}.osu", result.map_id)) {
            Ok(file) => file,
            Err(why) => panic!("Could not open file {}.osu: {}", result.map_id, why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map {}: {}", result.map_id, why),
        };

        taiko_test(map, result);
    }
}

#[cfg(feature = "async_tokio")]
#[test]
fn taiko_async_tokio() {
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

                taiko_test(map, result);
            }
        });
}

#[cfg(feature = "async_std")]
#[test]
fn taiko_async_std() {
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

            taiko_test(map, result);
        }
    })
}

const RESULTS: &[MapResult] = &[
    MapResult {
        map_id: 110219,
        mods: 256,
        stars: 4.08968885024159,
        pp: 172.2618767481684,
    },
    MapResult {
        map_id: 110219,
        mods: 0,
        stars: 5.137432251440863,
        pp: 253.6918375585501,
    },
    MapResult {
        map_id: 110219,
        mods: 64,
        stars: 6.7868926972961,
        pp: 420.7721479461209,
    },
    // -----
    MapResult {
        map_id: 168450,
        mods: 256,
        stars: 3.906304814419791,
        pp: 159.50978823553345,
    },
    MapResult {
        map_id: 168450,
        mods: 0,
        stars: 4.740171803038067,
        pp: 226.46677950133315,
    },
    MapResult {
        map_id: 168450,
        mods: 64,
        stars: 5.900723352868479,
        pp: 352.4632039368069,
    },
    // -----
    MapResult {
        map_id: 1097541,
        mods: 256,
        stars: 4.001086710846427,
        pp: 181.1195995270953,
    },
    MapResult {
        map_id: 1097541,
        mods: 0,
        stars: 4.891409786886079,
        pp: 258.3724413997574,
    },
    MapResult {
        map_id: 1097541,
        mods: 64,
        stars: 6.588219669147997,
        pp: 434.0217833573784,
    },
    // -----
    MapResult {
        map_id: 1432878,
        mods: 256,
        stars: 3.5826958150176584,
        pp: 127.191831998499,
    },
    MapResult {
        map_id: 1432878,
        mods: 0,
        stars: 4.416206873466799,
        pp: 183.53015221780785,
    },
    MapResult {
        map_id: 1432878,
        mods: 64,
        stars: 5.912036178782906,
        pp: 308.1608793395345,
    },
];
