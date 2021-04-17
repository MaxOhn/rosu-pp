[![crates.io](https://img.shields.io/crates/v/rosu-pp.svg)](https://crates.io/crates/rosu-pp) [![docs](https://docs.rs/rosu-pp/badge.svg)](https://docs.rs/rosu-pp)

# rosu-pp

A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.

Conversions between gamemodes are generally not supported.

Async is supported through features, see below.

### Usage

```rust
use std::fs::File;
use rosu_pp::{Beatmap, BeatmapExt};

let file = match File::open("/path/to/file.osu") {
    Ok(file) => file,
    Err(why) => panic!("Could not open file: {}", why),
};

// Parse the map yourself
let map = match Beatmap::parse(file) {
    Ok(map) => map,
    Err(why) => panic!("Error while parsing map: {}", why),
};

// If `BeatmapExt` is included, you can make use of
// some methods on `Beatmap` to make your life simpler.
// If the mode is known, it is recommended to use the
// mode's pp calculator, e.g. `TaikoPP`, manually.
let result = map.pp()
    .mods(24) // HDHR
    .combo(1234)
    .misses(2)
    .accuracy(99.2)
    .calculate();

println!("PP: {}", result.pp());

// If you intend to reuse the current map-mod combination,
// make use of the previous result!
// If attributes are given, then stars & co don't have to be recalculated.
let next_result = map.pp()
    .mods(24) // HDHR
    .attributes(result) // recycle
    .combo(543)
    .misses(5)
    .n50(3)
    .passed_objects(600)
    .accuracy(96.5)
    .calculate();

println!("Next PP: {}", next_result.pp());

let stars = map.stars(16, None).stars(); // HR
let max_pp = map.max_pp(16).pp();

println!("Stars: {} | Max PP: {}", stars, max_pp);
```

### With async
If either the `async_tokio` or `async_std` feature is enabled, beatmap parsing will be async.

```rust
use async_std::fs::File;
// use tokio::fs::File;

let file = match File::open("/path/to/file.osu").await {
    Ok(file) => file,
    Err(why) => panic!("Could not open file: {}", why),
};

// Parse the map asynchronously
let map = match Beatmap::parse(file).await {
    Ok(map) => map,
    Err(why) => panic!("Error while parsing map: {}", why),
};

// The rest stays the same
let result = map.pp()
    .mods(24) // HDHR
    .combo(1234)
    .misses(2)
    .accuracy(99.2)
    .calculate();

println!("PP: {}", result.pp());
```

### osu!standard versions

- `all_included`: Both stack leniency & slider paths are considered so that the difficulty and pp calculation immitates osu! as close as possible. Pro: Most precise; Con: Least performant.
- `no_leniency`: The positional offset of notes created by stack leniency is not considered. This means the jump distance inbetween notes might be slightly off, resulting in small inaccuracies. Since calculating these offsets is relatively expensive though, this version is considerably faster than `all_included`.
- `no_slider_no_leniency` (i.e. [oppai](https://github.com/Francesco149/oppai-ng)): In addition to not considering the positional offset caused by stack leniency, slider paths are also ignored. This means the travel distance of notes is completely omitted which may cause further inaccuracies. Since the slider paths don't have to be computed though, it is generally faster than `no_leniency`.

**Note**: If the `fruits` feature is enabled, sliders will be parsed regardless, resulting in a reduced performance advantage of `no_sliders_no_leniency`.

### Features

| Flag | Description |
|-----|-----|
| `default` | Enable all modes and choose the `no_leniency` version for osu!standard. |
| `taiko` | Enable osu!taiko. |
| `fruits` | Enable osu!ctb. |
| `mania` | Enable osu!mania. |
| `osu` | Enable osu!standard. Requires to also enable exactly one of the features `no_leniency`, `no_sliders_no_leniency`, or `all_included`. |
| `no_leniency` | When calculating difficulty attributes in osu!standard, ignore stack leniency but consider sliders. Solid middleground between performance and precision, hence the default version. |       
| `no_sliders_no_leniency` | When calculating difficulty attributes in osu!standard, ignore stack leniency and sliders. Best performance but slightly less precision than `no_leniency`. |
| `all_included` | When calculating difficulty attributes in osu!standard, consider both stack leniency and sliders. Best precision but significantly worse performance than `no_leniency`. |
| `async_tokio` | Beatmap parsing will be async through [tokio](https://github.com/tokio-rs/tokio) |
| `async_std` | Beatmap parsing will be async through [async-std](https://github.com/async-rs/async-std) |

### Benchmarks

Comparing the PP calculation speed between [osu-perf](https://gitlab.com/JackRedstonia/osu-perf/), an [oppai-ng](https://github.com/Francesco149/oppai-ng) rust binding, and rosu-pp's `no_sliders_no_leniency`:

<img src="./benchmark_results/crates_pp_calc.svg">

Comparing the PP calculation speed between rosu-pp's `all_included`, `no_leniency`, and `no_sliders_no_leniency` versions:

<img src="./benchmark_results/rosu_pp_calc.svg">

Comparing the PP (in)accuracy between rosu-pp's `all_included`, `no_leniency`, and `no_sliders_no_leniency` versions:

<img src="./benchmark_results/pp_inaccuracy.svg">

Comparing the stars (in)accuracy between rosu-pp's `all_included`, `no_leniency`, and `no_sliders_no_leniency` versions:

<img src="./benchmark_results/stars_inaccuracy.svg">

### Roadmap

- osu sr versions
  - \[x\] all included
  - \[x\] no_leniency
  - \[x\] no_sliders_no_leniency
- \[x\] taiko sr
- \[x\] ctb sr
- \[x\] mania sr
---
- \[x\] osu pp
- \[x\] taiko pp
- \[x\] ctb pp
- \[x\] mania pp
---
- \[x\] refactoring
- \[x\] benchmarking
- \[x\] async