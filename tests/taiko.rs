extern crate rosu_pp;

use rosu_pp::Beatmap;
use std::fs::File;

struct MapResult {
    map_id: u32,
    mods: u32,
    stars: f32,
    pp: f32,
}

#[test]
fn taiko() {
    let star_margin = 0.001;
    let pp_margin = 0.0075;

    for result in RESULTS {
        let MapResult {
            map_id,
            mods,
            stars,
            pp,
        } = result;

        let file = match File::open(format!("./maps/{}.osu", map_id)) {
            Ok(file) => file,
            Err(why) => panic!("Could not open file {}.osu: {}", map_id, why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map {}: {}", map_id, why),
        };

        let result = rosu_pp::taiko::PpCalculator::new(&map)
            .mods(*mods)
            .calculate();

        assert!(
            (result.stars - stars).abs() < star_margin,
            "Stars: {} | Expected: {} => {} margin [map {} | mods {}]",
            result.stars,
            stars,
            (result.stars - stars).abs(),
            map_id,
            mods
        );

        assert!(
            (result.pp - pp).abs() < pp_margin,
            "PP: {} | Expected: {} => {} margin [map {} | mods {}]",
            result.pp,
            pp,
            (result.pp - pp).abs(),
            map_id,
            mods
        );
    }
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
