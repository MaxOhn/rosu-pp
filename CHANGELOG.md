## Upcoming

- [BREAKING] With the importance of sliders for osu!standard, the `no_sliders_no_leniency` feature became too inaccurate. Additionally, since considering sliders now inherently drags performance down a little more, the difference between `no_leniency` and `all_included` became too small. Hence, the three osu features `no_sliders_no_leniency`, `no_leniency`, and `all_included` were removed. When the `osu` feature is enabled, it will now essentially use `all_included` under the hood.
  Additionally, instead of importing through `rosu_pp::osu::{version}`, you now have to import through `rosu_pp::osu`.
- [BREAKING] Instead of returning `PpResult`, performance calculations now return `PerformanceAttributes` depending on the mode.
- [BREAKING] Instead of returning `StarResult`, difficulty calculations now return `DifficultyAttributes` depending on the mode.
- [BREAKING] Various fields and methods now include `f64` instead of `f32` to stay true to osu!'s original code
- added internal binary crate `pp-gen` to calculate difficulty & pp values via `PerformanceCalculator.dll`
- osu: Updated up to commit [baa5285b59911efa1433a298f365133254a96874](https://github.com/ppy/osu/commit/baa5285b59911efa1433a298f365133254a96874) (2021-11-09)

# v0.2.3

- Reduced amount of required features of `async_std` and `async_tokio`
- Fixed a panic for some mania difficulty calculations on converts
- Updated the difficulty & pp changes from 21.07.27
- Fixed dead loop when reading empty `.osu` files ([#2] - [@Pure-Peace])
- Updated osu's clockrate bugfix for all modes

## v0.2.2

- osu & fruits:
  - Fixed specific slider patterns
  - Optimized Bezier, Catmull, and other small things

    Benchmarking for osu!standard showed a 25%+ improvement for performance aswell as accuracy

- fruits:
  - Fixed tick timing for reverse sliders

- taiko:
  - Micro optimizations

## v0.2.1

- parse & osu:
  - Cleanup and tiny optimizations

## v0.2.0

- Async beatmap parsing through features `async_tokio` or `async_std` ([#1] - [@Pure-Peace])
- [BREAKING] Hide various parsing related types further inwards, i.e. `rosu_pp::parse::some_type` instead of `rosu_pp::some_type`
  - Affected types: `DifficultyPoint`, `HitObject`, `Pos2`, `TimingPoint`, `HitObjectKind`, `PathType`, `HitSound`

## v0.1.1

- parse:
  - Efficiently handle huge amounts of curvepoints

- osu:
  - Fixed panic on unwrapping unavailable hit results
  - Fixed occasional underflow when calculating pp with passed_objects

- taiko:
  - Fixed missing flooring of hitwindow for pp calculation

- fruits:
  - Fixed passed objects in star calculation

- mania:
  - Fixed pp calculation on HR

[@Pure-Peace]: https://github.com/Pure-Peace

[#1]: https://github.com/MaxOhn/rosu-pp/pull/1
[#2]: https://github.com/MaxOhn/rosu-pp/pull/2
