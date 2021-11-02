## Upcoming

- added internal binary crate `pp-gen` to calculate difficulty & pp values via `PerformanceCalculator.dll`
- osu: Updated up to commit [6944151486e677bfd11f2390163aca9161defbbf](https://github.com/ppy/osu/commit/6944151486e677bfd11f2390163aca9161defbbf) (2021-10-27)

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
- Hide various parsing related types further inwards, i.e. `rosu_pp::parse::some_type` instead of `rosu_pp::some_type`
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
