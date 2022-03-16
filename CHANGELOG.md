## Upcoming

- osu: Fix panic on maps with 0 objects
- Fixed timing point parsing on some (older) maps where "uninherited" value did not coincide with beat length
- Fixed handling .osu files with missing difficulty attributes
- Added `AttributeProvider` impl for `{Mode}PerformanceAttributes`
- [BREAKING] Store `HitObject::sound` in `Beatmap::sounds` instead to reduce the struct size
- [BREAKING] Removed the mode features `osu`, `fruits`, `taiko`, and `mania`. Now all modes are always supported.

# v0.4.0

- Fixed out of bounds panic on maps with single-control-point linear sliders
- Fixed incorrect attributes on maps with only 1 or 2 hit objects for all modes
- Added method `Beatmap::from_path` so the file does not have to be created manually for `Beatmap::parse`.
- Added a bunch of documentation.
- Added method `Beatmap::bpm`
- Added method `max_combo` for `DifficultyAttributes`, `PerformanceAttributes`, and all `{Mode}PerformanceAttributes`
- Added methods `TaikoDifficultyAttributes::max_combo` and `OsuDifficultyAttributes::max_combo`
- Added structs `{Mode}GradualDifficultyAttributes` to calculate a map's difficulty after every or every few objects instead of calling the mode's `stars` function over and over.
- Added structs `{Mode}GradualPerformanceAttributes` to calculate the performance on a map after every or every few objects instead of using `{Mode}PP` over and over.
- Added `BeatmapExt::gradual_difficulty` and `BeatmapExt::gradual_performance` to gradually calculate the difficulty or performance on maps of any mode, hit object by hit object.
- Added methods `{Mode}PP::state` that take a `{Mode}ScoreState` (same for `AnyPP` and `ScoreState`) to set all parameters at once.
- [BREAKING] Removed the `ParseError` variants `InvalidPathType` and `InvalidTimingSignature` and renamed `InvalidFloatingPoint` to `InvalidDecimalNumber`.
- [BREAKING] Removed the `last_control_point` field of `HitObjectKind::Slider` when neither the `osu` nor the `fruits` feature is enabled.
- [BREAKING] Added the field `TaikoDifficultyAttributes::max_combo`
- [BREAKING] Renamed the `attributes` field to `difficulty` for all `{Mode}PerformanceAttributes` structs
- [BREAKING] Replaced field `FruitsDifficultyAttributes::max_combo` by a method with the same name

## v0.3.0

- [BREAKING] With the importance of sliders for osu!standard, the `no_sliders_no_leniency` feature became too inaccurate. Additionally, since considering sliders now inherently drags performance down a little more, the difference between `no_leniency` and `all_included` became too small. Hence, the three osu features `no_sliders_no_leniency`, `no_leniency`, and `all_included` were removed. When the `osu` feature is enabled, it will now essentially use `all_included` under the hood.
  Additionally, instead of importing through `rosu_pp::osu::{version}`, you now have to import through `rosu_pp::osu`.
- [BREAKING] Instead of returning `PpResult`, performance calculations now return `{Mode}PerformanceAttributes` and `PpResult` has been renamed to `PerformanceAttributes`.
- [BREAKING] Instead of returning `StarResult`, difficulty calculations now return `{Mode}DifficultyAttributes` and `StarResult` has been renamed to `DifficultyAttributes`.
- [BREAKING] Various fields and methods now include `f64` instead of `f32` to stay true to osu!'s original code
- Added internal binary crate `pp-gen` to calculate difficulty & pp values via `PerformanceCalculator.dll`
- Added internal binary crate `pp-plot` to plot out differences between `pp-gen`'s output and `rosu-pp` values
- osu: Updated up to commit [9fb2402781ad91c197d51aeec716b0000f52c4d1](https://github.com/ppy/osu/commit/9fb2402781ad91c197d51aeec716b0000f52c4d1) (2021-11-12)

## v0.2.3

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
