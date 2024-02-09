[![crates.io](https://img.shields.io/crates/v/rosu-pp.svg)](https://crates.io/crates/rosu-pp) [![docs](https://docs.rs/rosu-pp/badge.svg)](https://docs.rs/rosu-pp)

# rosu-pp

TODO: Rewrite readme

A standalone crate to calculate star ratings and performance points for all [osu!](https://osu.ppy.sh/home) gamemodes.

Async is supported through features, see below.

### Usage

```rust
use rosu_pp::{Beatmap, BeatmapExt};

// Parse the map yourself
let map = match Beatmap::from_path("/path/to/file.osu") {
    Ok(map) => map,
    Err(why) => panic!("Error while parsing map: {}", why),
};

// If `BeatmapExt` is included, you can make use of
// some methods on `Beatmap` to make your life simpler.
let result = map.pp()
    .mods(24) // HDHR
    .combo(1234)
    .accuracy(99.2)
    .misses(2)
    .calculate();

println!("PP: {}", result.pp());

// If you want to reuse the current map-mod combination, make use of the previous result!
// If attributes are given, then stars & co don't have to be recalculated.
let next_result = map.pp()
    .mods(24) // HDHR
    .attributes(result) // recycle
    .combo(543)
    .misses(5)
    .n50(3)
    .accuracy(96.5)
    .calculate();

println!("Next PP: {}", next_result.pp());

let stars = map.stars()
    .mods(16)  // HR
    .calculate()
    .stars();

let max_pp = map.max_pp(16).pp();

println!("Stars: {} | Max PP: {}", stars, max_pp);
```

### With async
If either the `async_tokio` or `async_std` feature is enabled, beatmap parsing will be async.

```rust
use rosu_pp::{Beatmap, BeatmapExt};

// Parse the map asynchronously
let map = match Beatmap::from_path("/path/to/file.osu").await {
    Ok(map) => map,
    Err(why) => panic!("Error while parsing map: {}", why),
};

// The rest stays the same
let result = map.pp()
    .mods(24) // HDHR
    .combo(1234)
    .n_misses(2)
    .accuracy(99.2)
    .calculate();

println!("PP: {}", result.pp());
```

### Gradual calculation
Sometimes you might want to calculate the difficulty of a map or performance of a score after each hit object.
This could be done by using `passed_objects` as the amount of objects that were passed so far.
However, this requires to recalculate the beginning again and again, we can be more efficient than that.

Instead, you should enable the `gradual` feature and use `GradualDifficulty` and `GradualPerformance`:

```rust
use rosu_pp::{
    Beatmap, BeatmapExt, GradualDifficulty, GradualPerformance, ScoreState,
    taiko::TaikoScoreState,
};

let map = match Beatmap::from_path("/path/to/file.osu") {
    Ok(map) => map,
    Err(why) => panic!("Error while parsing map: {}", why),
};

let mods = 8 + 64; // HDDT

// If you're only interested in the star rating or other difficulty values,
// use `GradualDifficulty`.
let gradual_difficulty = GradualDifficulty::new(&map, mods);

// Since `GradualDifficulty` implements `Iterator`, you can use
// any iterate function on it, use it in loops, collect them into a `Vec`, ...
for (i, difficulty) in gradual_difficulty.enumerate() {
    println!("Stars after object {}: {}", i, difficulty.stars());
}

// Gradually calculating performance values does the same as calculating
// difficulty attributes but it goes the extra step and also evaluates
// the state of a score for these difficulty attributes.
let mut gradual_performance = GradualPerformance::new(&map, mods);

// The default score state is kinda chunky because it considers all modes.
let state = ScoreState {
    max_combo: 1,
    n_geki: 0, // only relevant for mania
    n_katu: 0, // only relevant for mania and ctb
    n300: 1,
    n100: 0,
    n50: 0,
    n_misses: 0,
};

// Process the score state after the first object
let curr_performance = match gradual_performance.next(state) {
    Some(perf) => perf,
    None => panic!("the map has no hit objects"),
};

println!("PP after the first object: {}", curr_performance.pp());

// If you're only interested in maps of a specific mode, consider
// using the mode's gradual calculator instead of the general one.
// Let's assume it's a taiko map.
// Instead of starting off with `GradualPerformance` one could have
// used `TaikoGradualPerformance`.
let mut gradual_performance = match gradual_performance {
    GradualPerformanceAttributes::Taiko(gradual) => gradual,
    _ => panic!("the map was not taiko but {:?}", map.mode),
};

// A little simpler than the general score state.
let state = TaikoScoreState {
    max_combo: 11,
    n300: 9,
    n100: 1,
    n_misses: 1,
};

// Process the next 10 objects in one go (`nth` takes a zero-based value).
let curr_performance = match gradual_performance.nth(state, 9) {
    Some(perf) => perf,
    None => panic!("the previous `next` already processed the last object"),
};

println!("PP after the first 11 objects: {}", curr_performance.pp());
```

### Features

| Flag          | Description                                                                              |
| ------------- |------------------------------------------------------------------------------------------|
| `default`     | Beatmap parsing will be non-async                                                        |
| `async_tokio` | Beatmap parsing will be async through [tokio](https://github.com/tokio-rs/tokio)         |
| `async_std`   | Beatmap parsing will be async through [async-std](https://github.com/async-rs/async-std) |
| `gradual`     | Enable gradual difficulty and performance calculation                                    |

### Version

A large portion of this repository is a port of [osu!lazer](https://github.com/ppy/osu)'s difficulty and performance calculation.

- osu!:
  - osu!lazer: Commit `85adfc2df7d931164181e145377a6ced8db2bfb3` (Wed Sep 28 18:26:36 2022 +0300)
  - osu!tools: Commit `146d5916937161ef65906aa97f85d367035f3712` (Sat Oct 8 14:28:49 2022 +0900)
  - [Article](https://osu.ppy.sh/home/news/2022-09-30-changes-to-osu-sr-and-pp)

- taiko:
  - osu!lazer: Commit `234c6ac7998fbc6742503e1a589536255554e56a` (Wed Oct 5 20:21:15 2022 +0900)
  - osu!tools: Commit `146d5916937161ef65906aa97f85d367035f3712` (Sat Oct 8 14:28:49 2022 +0900)
  - [Article](https://osu.ppy.sh/home/news/2022-09-28-changes-to-osu-taiko-sr-and-pp)

- catch: (will be updated on the next rework)
  - osu!lazer: -
  - osu!tools: -

- mania:
  - osu!lazer: Commit `7342fb7f51b34533a42bffda89c3d6c569cc69ce` (Tue Oct 11 14:34:50 2022 +0900)
  - osu!tools: Commit `146d5916937161ef65906aa97f85d367035f3712` (Sat Oct 8 14:28:49 2022 +0900)
  - [Article](https://osu.ppy.sh/home/news/2022-10-09-changes-to-osu-mania-sr-and-pp)

### Accuracy

The difficulty and performance attributes generated by [osu-tools](https://github.com/ppy/osu-tools) itself were compared with rosu-pp's results when running on `130,000` different maps. Additionally, multiple mod combinations were tested depending on the mode:

- osu!: NM, EZ, HD, HR, DT
- taiko: NM, HD, HR, DT (+ all osu! converts)
- catch: -
- mania: NM, DT (+ all osu! converts)

For every (!) comparison of the star and pp values, the error margin was below `0.000000001`, ensuing a great accuracy.

### Benchmark

To be done

### Bindings

Using rosu-pp from other languages than Rust:
- JavaScript: [rosu-pp-js](https://github.com/MaxOhn/rosu-pp-js)
- Python: [rosu-pp-py](https://github.com/MaxOhn/rosu-pp-py)