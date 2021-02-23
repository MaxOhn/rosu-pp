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

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
#[test]
fn taiko_sync() {
    let star_margin = 0.0001;
    let pp_margin = 0.0001;

    for result in RESULTS {
        let MapResult {
            map_id,
            mods,
            stars,
            pp,
        } = result;

        let file = match std::fs::File::open(format!("./maps/{}.osu", map_id)) {
            Ok(file) => file,
            Err(why) => panic!("Could not open file {}.osu: {}", map_id, why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map {}: {}", map_id, why),
        };

        let result = rosu_pp::TaikoPP::new(&map).mods(*mods).calculate();

        assert_result!("Stars" => result.stars(), star_margin, stars, map_id, mods);
        assert_result!("PP" => result.pp(), pp_margin, pp, map_id, mods);
    }
}

#[cfg(feature = "async_tokio")]
#[test]
fn taiko_async_tokio() {
    tokio::runtime::Runtime::new()
        .expect("could not start runtime")
        .block_on(async {
            let star_margin = 0.0001;
            let pp_margin = 0.0001;

            for result in RESULTS {
                let MapResult {
                    map_id,
                    mods,
                    stars,
                    pp,
                } = result;

                let file = match tokio::fs::File::open(format!("./maps/{}.osu", map_id)).await {
                    Ok(file) => file,
                    Err(why) => panic!("Could not open file {}.osu: {}", map_id, why),
                };

                let map = match Beatmap::parse(file).await {
                    Ok(map) => map,
                    Err(why) => panic!("Error while parsing map {}: {}", map_id, why),
                };

                let result = rosu_pp::TaikoPP::new(&map).mods(*mods).calculate();

                assert_result!("Stars" => result.stars(), star_margin, stars, map_id, mods);
                assert_result!("PP" => result.pp(), pp_margin, pp, map_id, mods);
            }
        });
}

#[cfg(feature = "async_std")]
#[test]
fn taiko_async_std() {
    async_std::task::block_on(async {
        let star_margin = 0.0001;
        let pp_margin = 0.0001;

        for result in RESULTS {
            let MapResult {
                map_id,
                mods,
                stars,
                pp,
            } = result;

            let file = match async_std::fs::File::open(format!("./maps/{}.osu", map_id)).await {
                Ok(file) => file,
                Err(why) => panic!("Could not open file {}.osu: {}", map_id, why),
            };

            let map = match Beatmap::parse(file).await {
                Ok(map) => map,
                Err(why) => panic!("Error while parsing map {}: {}", map_id, why),
            };

            let result = rosu_pp::TaikoPP::new(&map).mods(*mods).calculate();

            assert_result!("Stars" => result.stars(), star_margin, stars, map_id, mods);
            assert_result!("PP" => result.pp(), pp_margin, pp, map_id, mods);
        }
    })
}

const RESULTS: &[MapResult] = &[
    MapResult {
        map_id: 110219,
        mods: 256,
        stars: 4.090461690284154,
        pp: 172.2934532692781,
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
        stars: 6.785308286298745,
        pp: 420.66337091577,
    },
    // -----
    MapResult {
        map_id: 168450,
        mods: 256,
        stars: 3.9102755155437663,
        pp: 159.66231311695273,
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
        stars: 5.894260068145283,
        pp: 352.08717183038954,
    },
    // -----
    MapResult {
        map_id: 1097541,
        mods: 256,
        stars: 4.0027499635116595,
        pp: 181.18594125657705,
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
        stars: 6.587467490088248,
        pp: 433.97174733352375,
    },
    // -----
    MapResult {
        map_id: 1432878,
        mods: 256,
        stars: 3.5850143199594258,
        pp: 127.27033873288904,
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
        stars: 5.908970879987477,
        pp: 307.9875634986321,
    },
];
