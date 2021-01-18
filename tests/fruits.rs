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
fn fruits() {
    let star_margin = 0.0001;
    let pp_margin = 0.0001;

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

        let result = rosu_pp::FruitsPP::new(&map).mods(*mods).calculate();

        assert!(
            (result.stars() - stars).abs() < star_margin * stars,
            "\nStars:\n\
                Calculated: {calculated} | Expected: {expected}\n \
                => {margin} margin ({allowed} allowed)\n\
                [map {map} | mods {mods}]\n",
            calculated = result.stars(),
            expected = stars,
            margin = (result.stars() - stars).abs(),
            allowed = star_margin * stars,
            map = map_id,
            mods = mods
        );

        assert!(
            (result.pp() - pp).abs() < pp_margin * pp,
            "\nPP:\n\
                Calculated: {calculated} | Expected: {expected}\n \
                => {margin} margin ({allowed} allowed)\n\
                [map {map} | mods {mods}]\n",
            calculated = result.pp(),
            expected = pp,
            margin = (result.pp() - pp).abs(),
            allowed = pp_margin * pp,
            map = map_id,
            mods = mods
        );
    }
}

const RESULTS: &[MapResult] = &[
    MapResult {
        map_id: 1977380,
        mods: 256,
        stars: 2.0564713386286573,
        pp: 45.1590377326849,
    },
    MapResult {
        map_id: 1977380,
        mods: 0,
        stars: 2.5695489769068742,
        pp: 67.73537806280385,
    },
    MapResult {
        map_id: 1977380,
        mods: 8,
        stars: 2.5695489769068742,
        pp: 81.28245367536461,
    },
    MapResult {
        map_id: 1977380,
        mods: 64,
        stars: 3.589887228221038,
        pp: 141.14620680718699,
    },
    MapResult {
        map_id: 1977380,
        mods: 16,
        stars: 3.1515873669521928,
        pp: 112.14972254944568,
    },
    MapResult {
        map_id: 1977380,
        mods: 2,
        stars: 3.0035260129778396,
        pp: 101.84717128368449,
    },
    // -----
    MapResult {
        map_id: 1974968,
        mods: 256,
        stars: 1.9544305373156605,
        pp: 42.91338693937752,
    },
    MapResult {
        map_id: 1974968,
        mods: 0,
        stars: 2.521701539665241,
        pp: 68.1785376623302,
    },
    MapResult {
        map_id: 1974968,
        mods: 8,
        stars: 2.521701539665241,
        pp: 86.927635519471,
    },
    MapResult {
        map_id: 1974968,
        mods: 64,
        stars: 3.650649037957456,
        pp: 139.53871429136828,
    },
    MapResult {
        map_id: 1974968,
        mods: 16,
        stars: 3.566302788963401,
        pp: 143.81118258776544,
    },
    MapResult {
        map_id: 1974968,
        mods: 2,
        stars: 2.2029392066882654,
        pp: 56.44764027057014,
    },
    // -----
    MapResult {
        map_id: 2420076,
        mods: 256,
        stars: 4.791039358886245,
        pp: 258.46694642171224,
    },
    MapResult {
        map_id: 2420076,
        mods: 0,
        stars: 6.223136555625056,
        pp: 471.1417837859138,
    },
    MapResult {
        map_id: 2420076,
        mods: 8,
        stars: 6.223136555625056,
        pp: 501.7659929922609,
    },
    MapResult {
        map_id: 2420076,
        mods: 64,
        stars: 8.908315960310958,
        pp: 1138.695343583009,
    },
    MapResult {
        map_id: 2420076,
        mods: 16,
        stars: 6.54788067620051,
        pp: 531.2886608194283,
    },
    MapResult {
        map_id: 2420076,
        mods: 2,
        stars: 6.067971540209479,
        pp: 446.888877247154,
    },
    // -----
    MapResult {
        map_id: 2206596,
        mods: 256,
        stars: 4.767182611189798,
        pp: 300.15942914986067,
    },
    MapResult {
        map_id: 2206596,
        mods: 0,
        stars: 6.157660207091584,
        pp: 531.0398776668264,
    },
    MapResult {
        map_id: 2206596,
        mods: 8,
        stars: 6.157660207091584,
        pp: 573.5230526869998,
    },
    MapResult {
        map_id: 2206596,
        mods: 64,
        stars: 8.93391286552717,
        pp: 1315.2112887084272,
    },
    MapResult {
        map_id: 2206596,
        mods: 16,
        stars: 6.8639096665110735,
        pp: 684.8296373011866,
    },
    MapResult {
        map_id: 2206596,
        mods: 2,
        stars: 5.60279198088948,
        pp: 447.8862884246722,
    },
];
