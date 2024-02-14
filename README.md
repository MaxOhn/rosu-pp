[![crates.io](https://img.shields.io/crates/v/rosu-pp.svg)](https://crates.io/crates/rosu-pp) [![docs](https://docs.rs/rosu-pp/badge.svg)](https://docs.rs/rosu-pp)

# rosu-pp

<!-- cargo-rdme start -->

Library to calculate difficulty and performance attributes for all [osu!] gamemodes.

### Description

A large part of `rosu-pp` is a port of [osu!lazer]'s difficulty and performance calculation
with emphasis on a precise translation to Rust for the most accurate results.

Another important factor is the calculation speed. Optimizations and an accurate translation
unfortunately don't always go hand-in-hand. Nonetheless, performance improvements are still
snuck in wherever possible, providing a significantly faster runtime than the native C# code.

TODO: values to compare

Additionally, `rosu-pp` allows previous values to be re-used so that they don't need to be
calculated again. For example, a beatmap needs to be decoded only once and can then be used
for any amount of attribute calculations. Similarly, previous attributes can be re-used for
later calculations (with some limitations, see the [example](#usage)).

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
let diff_attrs = map.difficulty()
    .mods(8 + 16) // HDHR
    .calculate();

let stars = diff_attrs.stars();

// Calculate performance attributes
let perf_attrs = map.performance()
    // To speed up the calculation, we can use the previous attributes.
    // **Note** that this should only be done if the map, mode, mods, 
    // clock rate, and amount of passed objects stay the same.
    // Otherwise, the final attributes will be incorrect.
    .attributes(diff_attrs)
    .mods(24) // HDHR, must be the same as before
    .combo(789)
    .accuracy(99.2)
    .misses(2)
    .calculate();

let pp = perf_attrs.pp();

// Again, we re-use the previous attributes for maximum efficiency.
// This time we do it directly instead of through the map.
let max_pp = perf_attrs.performance()
    .mods(24) // Still the same
    .calculate()
    .pp();

println!("Stars: {stars} | PP: {pp}/{max_pp}");
```

### Gradual calculation

TODO

### Features

| Flag      | Description | Dependencies
| --------- | ----------- | ------------
| `default` | No features |
| `tracing` | Any error encountered during beatmap decoding will be logged through `tracing::error`. If this feature is not enabled, errors will be ignored. | [`tracing`]

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

<!-- cargo-rdme end -->
