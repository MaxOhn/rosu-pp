# Introduction

rosu-pp is a port of the osu! performance calculator. Its goal is to provide results that are as close as possible to the original C# implementation, while also being fast and efficient.

osu! is a rhythm game where users play on a "beatmap" which consists of music and hitobjects. Users must interact with the hitobjects based on the rhythm. Depending on how many hitobjects they successfully interacted with, how precise they were, and how hard the beatmap is in general, their performance is measured and evaluated as "performance attributes". The part of the performance attributes that describes the overall difficulty of a beatmap is called "difficulty attributes".

# Project structure

Each gamemode (osu, taiko, catch, mania) has its module. Performance calculation generally includes three parts: Beatmap parsing, difficulty attribute calculation, and performance attribute calculation.

Both difficulty- and performance calculation have analogous "gradual" calculation methods which provide the same result through an iterator-like approach.

# Hitresult generation

Performance calculation requires a "score state", part of which are "hitresults". Each gamemode has a different set of hitresults:
- osu: n300, n100, n50, misses
- taiko: n300, n100, misses
- catch: fruits, droplets, tiny droplets, tiny droplet misses, misses
- mania: n320, n300, n200, n100, n50, misses

Each mode has an accuracy calculation based on the hitresults. Users are able to just provide accuracy so that rosu-pp needs to generate matching hitresults. Hitresults that match an accuracy are not necessarily unique.

Hitresult generation may have different priorities:
- Fast generation but not necessarily accurate
- Closest possible accuracy
- Statistical approach for approximation
- etc.

All kinds of hitresult generations should be tested thoroughly.

# Tooling

- `cargo nextest run` for tests
- `cargo clippy --all-targets` should always be satisfied
- `cargo +nightly fmt --check` should always be satisfied
- `divan` dependency for benchmarks
- No other dependencies

# Testing

- Private functionality requires thorough unit testing.
- Public functionality should be tested in integration tests.

# Coding conventions

- Empty line before keywords such as `break`, `continue`, `return`.
- Empty line before returned values at the end of a scope.
- Empty line before and after multi-line statements such as blocks, if-else, match, etc.
- For methods on primitive numbers, prefer the explicit notation, e.g.
  - `f64::abs(x)` instead of `x.abs()`
  - `cmp::min(my_u32, other_u32)` instead of `my_u32.min(other_u32)`
  - `i64::from(my_i32)` instead of `my_i32 as i64`
- Prefer passing methods or functions themselves rather than closures, e.g.
  - `vec_of_options.iter().filter(Option::is_some)` instead of `vec_of_options.iter().filter(|opt| opt.is_some())`
- While formatting, prefer inlining variables, e.g.
  - `write!(f, "abc {x}")` instead of `write!(f, "abc {}", x)`
  - `println!("val1={val1:?} | val2={val2}")` instead of `println!("val1={:?} | val2={}", val1, val2)`
