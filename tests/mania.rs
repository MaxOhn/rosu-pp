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
fn mania() {
    let star_margin = 0.00001;
    let pp_margin = 0.00001;

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

        let result = rosu_pp::mania::PpCalculator::new(&map)
            .mods(*mods)
            .calculate();

        assert!(
            (result.stars - stars).abs() < star_margin * stars,
            "\nStars:\n\
                Calculated: {calculated} | Expected: {expected}\n \
                => {margin} margin ({allowed} allowed)\n\
                [map {map} | mods {mods}]\n",
            calculated = result.stars,
            expected = stars,
            margin = (result.stars - stars).abs(),
            allowed = star_margin * stars,
            map = map_id,
            mods = mods
        );

        assert!(
            (result.pp - pp).abs() < pp_margin * pp,
            "\nPP:\n\
                Calculated: {calculated} | Expected: {expected}\n \
                => {margin} margin ({allowed} allowed)\n\
                [map {map} | mods {mods}]\n",
            calculated = result.pp,
            expected = pp,
            margin = (result.pp - pp).abs(),
            allowed = pp_margin * pp,
            map = map_id,
            mods = mods
        );
    }
}

const RESULTS: &[MapResult] = &[
    MapResult {
        map_id: 1355822,
        mods: 256,
        stars: 2.2710870990702627,
        pp: 43.654638516311564,
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
        stars: 3.748525363730352,
        pp: 140.25944202912322,
    },
    // -----
    MapResult {
        map_id: 1974394,
        mods: 256,
        stars: 3.8736942117487256,
        pp: 155.79817482289727,
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
        stars: 6.517894438878535,
        pp: 508.4849982082652,
    },
    // -----
    MapResult {
        map_id: 992512,
        mods: 256,
        stars: 5.29507262961579,
        pp: 317.82741466200764,
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
        stars: 8.944195050951032,
        pp: 1035.4596772151892,
    },
];
