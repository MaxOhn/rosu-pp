# rosu-pp

A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.

Conversions are generally not supported.

### Usage
```rust
use std::fs::File;
use rosu_pp::{Beatmap, BeatmapExt, GameMode, OsuPP, TaikoPP};

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
// You will have to match on the mode yourself
// to be able to set all options for pp calculation.
match map.mode {
    GameMode::STD => {
        let result = OsuPP::new(&map)
            .mods(24) // HDHR
            .combo(1234)
            .misses(2)
            .accuracy(99.2)
            .calculate();

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
            .calculate();

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
let stars = map.stars(16, None).stars(); // HR
let max_pp = map.max_pp(16).pp();

println!("Stars: {} | Max PP: {}", stars, max_pp);
```

### osu!standard versions
- `all_included`: WIP
- `no_leniency`: The positional offset of notes created by stack leniency is not considered. This means the jump distance inbetween notes might be slightly off, resulting in small inaccuracies. Since calculating these offsets is relatively expensive though, this version is considerably faster than `all_included`.
- `no_slider_no_leniency` (i.e. [oppai](https://github.com/Francesco149/oppai-ng)): In addtion to not considering the positional offset caused by stack leniency, slider paths are also ignored. This means the travel distance of notes is completely omitted which may cause further inaccuracies. Since the slider paths don't have to be computed though, it should generally be faster than `no_leniency`.

### Features

| Flag | Description |
|-----|-----|
| `default` | Enable all modes and choose the `no_leniency` version for osu!standard. |
| `taiko` | Enable osu!taiko. |
| `fruits` | Enable osu!ctb. |
| `mania` | Enable osu!mania. |
| `osu` | Enable osu!standard. Requires to also enable exactly one of the features `no_leniency`, `no_sliders_no_leniency`, or `all_included`. |
| `no_leniency` | When calculating difficulty attributes in osu!standard, ignore stack leniency but consider sliders. Solid middleground between performance and precision, suggested default version. |       
| `no_sliders_no_leniency` | When calculating difficulty attributes in osu!standard, ignore stack leniency and sliders. Best performance but slightly less precision than `no_leniency`. |
| `all_included` | When calculating difficulty attributes in osu!standard, consider both stack leniency and sliders. Best precision but significantly worse performance than `no_leniency`. |        

### Roadmap
- osu sr versions
  - [ ] all included
  - [x] no_leniency
  - [x] no_sliders_no_leniency (i.e. [oppai](https://github.com/Francesco149/oppai-ng))
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