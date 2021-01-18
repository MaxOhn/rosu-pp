# rosu-pp

A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.

Conversions are generally not supported.

#### Usage
```rust
use std::fs::File;
use rosu_pp::{Beatmap, BeatmapExt, GameMode, OsuPP, TaikoPP};

fn main() {
    let file = match File::open("/path/to/file.osu") {
        Ok(file) => file,
        Err(why) => panic!("Could not open file: {}", why),
    };

    // Parse the map yourself
    let map = match Beatmap::parse(file) {
        Ok(map) => map,
        Err(why) => panic!("Error while parsing map: {}", why),
    };

    // The different modes make things annoying because their
    // pp calculations require different parameters.
    // For now, you will have to match on the mode yourself
    // to be able to set all options for pp calculation.
    match map.mode {
        GameMode::STD => {
            let result = OsuPP::new(&map)
                .mods(24) // HDHR
                .combo(1234)
                .misses(2)
                .accuracy(99.2)
                // `no_leniency` is the suggested default
                .calculate(rosu_pp::osu::no_leniency::stars);

            println!("PP: {}", result.pp());

            // If you intend to reuse the current map-mod combination,
            // make use of the previous result!
            // If attributes are given, then stars & co don't have to be recalculated.
            let next_result = OsuPP::new(&map)
                .mods(24) // HDHR
                .attributes(result)
                .combo(543)
                .misses(5)
                .n50(3)
                .accuracy(97.5)
                .calculate(rosu_pp::osu::no_leniency::stars);

            println!("Next PP: {}", next_result.pp());
        },
        GameMode::TKO => {
            let result = TaikoPP::new(&map)
                .mods(64) // DT
                .combo(555)
                .misses(10)
                .passed_objects(600)
                .accuracy(95.12345)
                .calculate();

            println!("Stars: {} | PP: {}", result.stars(), result.pp());
        }
        GameMode::MNA | GameMode::CTB => panic!("do your thing"),
    }

    // If all you want is the map's stars or max pp,
    // you can make use of the BeatmapExt trait.
    let stars = map.stars(16, None); // HR
    let max_pp = map.max_pp(16);

    println!("Stars: {} | Max PP: {}", stars, max_pp);
}
```

#### osu!standard versions
- `all_included`: WIP
- `no_leniency`: The positional offset of notes created by stack leniency is not considered. This means the jump distance inbetween notes might be slightly off, resulting in small inaccuracies. Since calculating these offsets is relatively expensive though, this version is considerably faster than `all_included`.
- `no_slider_no_leniency` (i.e. oppai): In addtion to not considering the positional offset caused by stack leniency, slider paths are also ignored. This means the travel distance of notes is completely omitted which may cause further inaccuracies. Since the slider paths don't have to be computed though, it should generally be faster than `no_leniency`.

#### Roadmap
- osu sr versions
  - [ ] all included
  - [x] no_leniency
  - [x] no_sliders_no_leniency (i.e. oppai)
- [x] taiko sr
- [x] ctb sr
- [x] mania sr
---
- [x] osu pp
- [x] taiko pp
- [x] ctb pp
- [x] mania pp
---
- [x] refactoring
- [ ] benchmarking