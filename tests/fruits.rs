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
    let star_margin = 0.005;
    let pp_margin = 0.005;

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

        let result = rosu_pp::fruits::PpCalculator::new(&map)
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
        map_id: 1977380,
        mods: 256,
        stars: 2.0564713386286573,
        pp: 43.49758286973066,
    },
    MapResult {
        map_id: 1977380,
        mods: 0,
        stars: 2.5695489769068742,
        pp: 71.54271564752817,
    },
    MapResult {
        map_id: 1977380,
        mods: 8,
        stars: 2.5695489769068742,
        pp: 78.29197525261374,
    },
    MapResult {
        map_id: 1977380,
        mods: 64,
        stars: 3.589887228221038,
        pp: 135.95326950246636,
    },
    MapResult {
        map_id: 1977380,
        mods: 16,
        stars: 3.1515873669521928,
        pp: 108.02360048697571,
    },
    MapResult {
        map_id: 1977380,
        mods: 2,
        stars: 3.0035260129778396,
        pp: 98.10009237095251,
    },
    // -----
    MapResult {
        map_id: 1974968,
        mods: 256,
        stars: 1.9544305373156605,
        pp: 40.46051204584743,
    },
    MapResult {
        map_id: 1974968,
        mods: 0,
        stars: 2.521701539665241,
        pp: 64.28153872477789,
    },
    MapResult {
        map_id: 1974968,
        mods: 8,
        stars: 2.521701539665241,
        pp: 81.9589618740918,
    },
    MapResult {
        map_id: 1974968,
        mods: 64,
        stars: 3.650649037957456,
        pp: 131.5628579590708,
    },
    MapResult {
        map_id: 1974968,
        mods: 16,
        stars: 3.566302788963401,
        pp: 135.59111737415918,
    },
    MapResult {
        map_id: 1974968,
        mods: 2,
        stars: 2.2029392066882654,
        pp: 53.2211645832911,
    },
    // -----
    MapResult {
        map_id: 2420076,
        mods: 256,
        stars: 4.791039358886245,
        pp: 226.85533170425614,
    },
    MapResult {
        map_id: 2420076,
        mods: 0,
        stars: 6.223136555625056,
        pp: 413.51912544400295,
    },
    MapResult {
        map_id: 2420076,
        mods: 8,
        stars: 6.223136555625056,
        pp: 440.3978626824246,
    },
    MapResult {
        map_id: 2420076,
        mods: 64,
        stars: 8.908315960310958,
        pp: 999.4280253427237,
    },
    MapResult {
        map_id: 2420076,
        mods: 16,
        stars: 6.54788067620051,
        pp: 466.3097817709075,
    },
    MapResult {
        map_id: 2420076,
        mods: 2,
        stars: 6.067971540209479,
        pp: 392.2324532647843,
    },
    // -----
    MapResult {
        map_id: 2206596,
        mods: 256,
        stars: 4.767182611189798,
        pp: 227.40643918013868,
    },
    MapResult {
        map_id: 2206596,
        mods: 0,
        stars: 6.157660207091584,
        pp: 402.3258172661857,
    },
    MapResult {
        map_id: 2206596,
        mods: 8,
        stars: 6.157660207091584,
        pp: 434.5118711368466,
    },
    MapResult {
        map_id: 2206596,
        mods: 64,
        stars: 8.93391286552717,
        pp: 996.4288537655079,
    },
    MapResult {
        map_id: 2206596,
        mods: 16,
        stars: 6.8639096665110735,
        pp: 518.8398368985938,
    },
    MapResult {
        map_id: 2206596,
        mods: 2,
        stars: 5.60279198088948,
        pp: 339.327091261929,
    },
];
