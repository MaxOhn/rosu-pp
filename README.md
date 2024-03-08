[![crates.io](https://img.shields.io/crates/v/rosu-pp.svg)](https://crates.io/crates/rosu-pp) [![docs](https://docs.rs/rosu-pp/badge.svg)](https://docs.rs/rosu-pp)

# rosu-pp

<!-- cargo-rdme start -->

Library to calculate difficulty and performance attributes for all [osu!] gamemodes.

A large part of `rosu-pp` is a port of [osu!lazer]'s difficulty and performance calculation
with emphasis on a precise translation to Rust for the most [accurate results](#accuracy)
while also providing a significant [boost in performance](#speed).

Last commits of the ported code:
  - [osu!lazer] : `7342fb7f51b34533a42bffda89c3d6c569cc69ce` (2022-10-11)
  - [osu!tools] : `146d5916937161ef65906aa97f85d367035f3712` (2022-10-08)

News posts of the latest gamemode updates:
  - osu: <https://osu.ppy.sh/home/news/2022-09-30-changes-to-osu-sr-and-pp>
  - taiko: <https://osu.ppy.sh/home/news/2022-09-28-changes-to-osu-taiko-sr-and-pp>
  - catch: <https://osu.ppy.sh/home/news/2020-05-14-osucatch-scoring-updates>
  - mania: <https://osu.ppy.sh/home/news/2022-10-09-changes-to-osu-mania-sr-and-pp>

### Usage

```rust
// Decode the map
let map = rosu_pp::Beatmap::from_path("./resources/2785319.osu").unwrap();

// Calculate difficulty attributes
let diff_attrs = rosu_pp::Difficulty::new()
    .mods(8 + 16) // HDHR
    .calculate(&map);

let stars = diff_attrs.stars();

// Calculate performance attributes
let perf_attrs = rosu_pp::Performance::from_attributes(diff_attrs)
    // To speed up the calculation, we used the previous attributes.
    // **Note** that this should only be done if the map and all difficulty
    // settings stay the same, otherwise the final attributes will be incorrect!
    .mods(24) // HDHR, must be the same as before
    .combo(789)
    .accuracy(99.2)
    .misses(2)
    .calculate();

let pp = perf_attrs.pp();

// Again, we re-use the previous attributes for maximum efficiency.
let max_pp = perf_attrs.performance()
    .mods(24) // Still the same
    .calculate()
    .pp();

println!("Stars: {stars} | PP: {pp}/{max_pp}");
```

### Gradual calculation

Gradually calculating attributes provides an efficient way to process each hitobject
separately and calculate the attributes only up to that point.

For difficulty attributes, there is `GradualDifficulty` which implements `Iterator`
and for performance attributes there is `GradualPerformance` which requires the current
score state.

```rust
use rosu_pp::{Beatmap, GradualPerformance, Difficulty, any::ScoreState};

let map = Beatmap::from_path("./resources/1028484.osu").unwrap();

let mut gradual = Difficulty::new()
    .mods(16 + 64) // HRDT
    .clock_rate(1.2)
    .gradual_performance(&map);

let mut state = ScoreState::new(); // empty state, everything is on 0.

// The first 10 hitresults are 300s
for _ in 0..10 {
    state.n300 += 1;
    state.max_combo += 1;
    let attrs = gradual.next(state.clone()).unwrap();
    println!("PP: {}", attrs.pp());
}

// Fast-forward to the end
state.max_combo = ...
state.n300 = ...
state.n_katu = ...
...
let attrs = gradual.last(state).unwrap();
println!("PP: {}", attrs.pp());
```

### Accuracy

`rosu-pp` was tested against all current beatmaps on multiple mod combinations and delivered
values that matched osu!lazer perfectly down to the last decimal place.

However, there is one small caveat: the values are only this precise on debug mode.
On release mode, Rust's compiler performs optimizations that produce the tiniest discrepancies
due to floating point inaccuracies which can cascade into larger differences in the end.
With this in mind, `rosu-pp` is still as accurate as can be without targeting the
.NET compiler itself. Realistically, the inaccuracies in release mode are negligibly small.

### Speed

An important factor for `rosu-pp` is the calculation speed. Optimizations and an accurate translation
unfortunately don't always go hand-in-hand. Nonetheless, performance improvements are still
snuck in wherever possible, providing a significantly faster runtime than the native C# code.

Results of a rudimentary [benchmark] of osu!lazer and rosu-pp:
```txt
osu!lazer:
Decoding maps:            Median: 378.10ms | Mean: 381.47ms
Calculating difficulties: Median: 588.89ms | Mean: 597.11ms
Calculating performances: Median: 315.90µs | Mean: 310.60µs

rosu-pp:
Decoding maps:            Median: 46.94ms | Mean: 47.21ms
Calculating difficulties: Median: 72.90ms | Mean: 73.13ms
Calculating performances: Median: 44.13µs | Mean: 45.53µs
```

### Features

| Flag      | Description | Dependencies
| --------- | ----------- | ------------
| `default` | No features |
| `sync`    | Some gradual calculation types can only be shared across threads if this feature is enabled. This adds a performance penalty so only enable this if really needed. |
| `tracing` | Any error encountered during beatmap decoding will be logged through `tracing::error`. If this feature is **not** enabled, errors will be ignored. | [`tracing`]

### Bindings

Using `rosu-pp` from other languages than Rust:
- JavaScript: [rosu-pp-js]
- Python: [rosu-pp-py]

[osu!]: https://osu.ppy.sh/home
[osu!lazer]: https://github.com/ppy/osu
[osu!tools]: https://github.com/ppy/osu-tools
[`tracing`]: https://docs.rs/tracing
[rosu-pp-js]: https://github.com/MaxOhn/rosu-pp-js
[rosu-pp-py]: https://github.com/MaxOhn/rosu-pp-py
[benchmark]: https://gist.github.com/MaxOhn/625af10011f6d7e13a171b08ccf959ff

<!-- cargo-rdme end -->
