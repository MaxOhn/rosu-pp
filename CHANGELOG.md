## Upcoming

Nothing as of now

# v0.9.2 (2022-11-08)

- __Adjustments:__
  - When passing an osu!std map to `TaikoGradualDifficultyAttributes` or `ManiaGradualDifficultyAttributes`, it now automatically converts the map internally. For osu!catch is was already happening trivially.

- __Fixes:__
  - Fixed passed object count for taiko by ignoring non-circles
  - Fixed a niche panic on UTF-16 encoded maps ([#18])
  - Fixed an occasional underflow when calculating accuracy pp
  - Fixed an infinite loop on special ctb maps

## v0.9.1 (2022-10-26)

- __Adjustments:__
  - When passing an osu!std map to `TaikoPP` or `ManiaPP`, it now automatically converts the map internally. For osu!catch it was already happening trivially.

- __Fixes:__
  - The fields `section_len` for all strain structs no longer depends on the clock rate.

## v0.9.0 (2022-10-24)

Big changes including the most recent [osu!](https://osu.ppy.sh/home/news/2022-09-30-changes-to-osu-sr-and-pp), [taiko](https://osu.ppy.sh/home/news/2022-09-28-changes-to-osu-taiko-sr-and-pp), and [mania](https://osu.ppy.sh/home/news/2022-10-09-changes-to-osu-mania-sr-and-pp) updates, aswell as various breaking changes.

- __Breaking changes:__
  - `TimingPoint` and `DifficultyPoint` no longer contain a `kiai` field
  - `DifficultyPoint` now has the additional fields `bpm_mult` and `generate_ticks`
  - `Beatmap` now stores timing- and difficulty points in a `SortedVec`
  - `Beatmap` now has the additional field `effect_points`
  - For the performance calculators `OsuPP`, `TaikoPP`, `ManiaPP`, and `AnyPP` the method `misses` has been renamed to `n_misses`
  - The accuracy method for `OsuPP`, `TaikoPP`, and `ManiaPP` is no longer required to be called last
  - `ManiaPP` no longer has a `score` method. Instead it has `n320`, `n300`. `n200`, `n100`, `n50`, and `n_misses` methods, aswell as a `state` method
  - Gradual performance calculation for mania now requires a `ManiaScoreState` instead of `score`
  - `ManiaDifficultyAttributes` now have a `max_combo` field and method
  - `OsuDifficultyAttributes` now have a `speed_note_count` field
  - `OsuPerformanceAttributes` and `TaikoPerformanceAttributes` now have a `effective_miss_count` field
  - `TaikoDifficultyAttributes` now have a `peak` and `hit_window` field
  - Some other things I likely forgot about :S

- __Additions:__
  - The performance calculators `OsuPP`, `TaikoPP`, `ManiaPP`, and `AnyPP` now have a `hitresult_priority` method to specify how hitresults should be generated

- __Fixes:__
  - Fixed a bunch of fringe yet significant bugs for taiko and mania converts
  - Fixed various floating point inaccuracies for osu!standard
  - Fixed parsing difficulty points from .osu files
  - Instead of throwing an error, invalid lines during parsing will just be ignored in some cases
  - Fixed an unsafe transmute between incompatible types while parsing sliders

## v0.8.0 (2022-08-02)

- __Fixes:__
  - Fixed stack overflow bug when allocating ticks for some sliders on converted catch maps ([#14])
- __Breaking changes:__
  - `Beatmap::attributes` now returns a new type `BeatmapAttributesBuilder` to allow for more
  fine-grained calculations. `BeatmapAttributes` now contains expected values and also includes
  a `BeatmapHitWindows` field containing the AR (preempt) and OD (great) hit windows in 
  milliseconds. ([#15])

## v0.7.1 (2022-07-12)

- __Fixes:__
  - Parsing edge sounds is now mindful about overflowing a byte (ref. ranked map id 80799)
  - Parsing the event section now attempts to read non-ASCII before eventually failing (ref. ranked map id 49374)

## v0.7.0 (2022-07-06)

- __Fixes:__
  - Slider velocity is now adjusted properly for taiko converts
  - Fixed missing slider sounds for taiko converts
- __Breaking changes:__
  - Replaced the simple `Strains` struct with a new struct `{Mode}Strains` that contains more detail w.r.t. the mode.
  - Renamed all `GameMode` variants to more idiomatic names
  - Renamed `ParseError::IOError` to `ParseError::IoError`

## v0.6.0 (2022-07-05)

- __Additions__:
  - Added the `ControlPoint` and `ControlPointerIter` types to the public interface
  - `TimingPoint` and `DifficultyPoint` now implement `Default`
  - Added new methods to `Beatmap`:
    - `convert_mode`: Convert a map into another mode. (doesn't do anything if the starting map is not osu!standard)
    - `control_points`: Return an iterator over all control points of a map
    - `total_break_time`: Return the accumulated break time in milliseconds
    - `timing_point_at`: Return the timing point for the given timestamp
    - `difficulty_point_at`: Return the difficulty point for the given timestamp if available
- __Breaking changes:__
  - Moved some types to a different module. The following types can now be found in `rosu_pp::beatmap`:
    - `Beatmap`
    - `BeatmapAttributes`
    - `ControlPoint`
    - `ControlPointIter`
    - `DifficultyPoint`
    - `GameMode`
    - `TimingPoint`
  - Added a new field `kiai: bool` to both `TimingPoint` and `DifficultyPoint` to denote whether the current timing section is in kiai mode
  - Added a new field `breaks: Vec<Break>` to `Beatmap` that contains all breaks throughout the map
  - Added a new field `edge_sounds: Vec<u8>` to the `Slider` variant of `HitObjectKind` to denote the sample played on slider heads, ends, and repeats
- __Other:__
  - Small performance improvements for osu!taiko calculations

## v0.5.2 (2022-06-14)

- __Fixes:__
  - Fixed parsing non-UTF-8 encoded files and improved parse performance overall ([#9])
  - Handle missing approach rate properly this time

## v0.5.1 (2022-03-21)

- __Fixes:__
  - Performance calculation for taiko & mania now considers custom clock rates properly

## v0.5.0 (2022-03-21)

- __Fixes:__
  - Fixed panic on maps with 0 objects
  - Fixed droplet timings on juicestreams with span count >1
  - Fixed timing point parsing on some (older) maps where "uninherited" value did not coincide with beat length
  - Fixed handling .osu files with missing difficulty attributes
  - Fixed huge memory allocations caused by incorrectly parsing .osu files
- __Breaking changes:__
  - The `stars` and `strains` functions for all modes were removed. Instead use the `{Mode}Stars` builder pattern which is similar to `{Mode}PP`.
  - `BeatmapExt::stars`'s definition was adjusted to use the `AnyStars` builder struct
  - Store `HitObject::sound` in `Beatmap::sounds` instead to reduce the struct size
  - Removed the mode features `osu`, `fruits`, `taiko`, and `mania`. Now all modes are always supported.
  - Renamed the `rosu_pp::fruits` module to `rosu_pp::catch`. Similarly, all structs `Fruits{Name}` were renamed to `Catch{Name}` and enums over the mode have their `Fruits` variant renamed to `Catch`
  - Renamed `Mods`' method `speed` to `clock_rate`
- __Additions:__
  - Added `AttributeProvider` impl for `{Mode}PerformanceAttributes`
  - Added the method `clock_rate` to `{Mode}PP` and `{Mode}Stars` to consider a custom clock rate instead of the one dictated by mods.

## v0.4.0 (2021-11-25)

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

## v0.3.0 (2021-11-14)

- [BREAKING] With the importance of sliders for osu!standard, the `no_sliders_no_leniency` feature became too inaccurate. Additionally, since considering sliders now inherently drags performance down a little more, the difference between `no_leniency` and `all_included` became too small. Hence, the three osu features `no_sliders_no_leniency`, `no_leniency`, and `all_included` were removed. When the `osu` feature is enabled, it will now essentially use `all_included` under the hood.
  Additionally, instead of importing through `rosu_pp::osu::{version}`, you now have to import through `rosu_pp::osu`.
- [BREAKING] Instead of returning `PpResult`, performance calculations now return `{Mode}PerformanceAttributes` and `PpResult` has been renamed to `PerformanceAttributes`.
- [BREAKING] Instead of returning `StarResult`, difficulty calculations now return `{Mode}DifficultyAttributes` and `StarResult` has been renamed to `DifficultyAttributes`.
- [BREAKING] Various fields and methods now include `f64` instead of `f32` to stay true to osu!'s original code
- Added internal binary crate `pp-gen` to calculate difficulty & pp values via `PerformanceCalculator.dll`
- Added internal binary crate `pp-plot` to plot out differences between `pp-gen`'s output and `rosu-pp` values
- osu: Updated up to commit [9fb2402781ad91c197d51aeec716b0000f52c4d1](https://github.com/ppy/osu/commit/9fb2402781ad91c197d51aeec716b0000f52c4d1) (2021-11-12)

## v0.2.3 (2021-08-09)

- Reduced amount of required features of `async_std` and `async_tokio`
- Fixed a panic for some mania difficulty calculations on converts
- Updated the difficulty & pp changes from 21.07.27
- Fixed dead loop when reading empty `.osu` files ([#2] - [@Pure-Peace])
- Updated osu's clockrate bugfix for all modes

## v0.2.2 (2021-05-05)

- osu & fruits:
  - Fixed specific slider patterns
  - Optimized Bezier, Catmull, and other small things

    Benchmarking for osu!standard showed a 25%+ improvement for performance aswell as accuracy

- fruits:
  - Fixed tick timing for reverse sliders

- taiko:
  - Micro optimizations

## v0.2.1 (2021-04-17)

- parse & osu:
  - Cleanup and tiny optimizations

## v0.2.0 (2021-02-25)

- Async beatmap parsing through features `async_tokio` or `async_std` ([#1] - [@Pure-Peace])
- [BREAKING] Hide various parsing related types further inwards, i.e. `rosu_pp::parse::some_type` instead of `rosu_pp::some_type`
  - Affected types: `DifficultyPoint`, `HitObject`, `Pos2`, `TimingPoint`, `HitObjectKind`, `PathType`, `HitSound`

## v0.1.1 (2021-02-15)

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
[#9]: https://github.com/MaxOhn/rosu-pp/pull/9
[#14]: https://github.com/MaxOhn/rosu-pp/pull/14
[#15]: https://github.com/MaxOhn/rosu-pp/pull/15
[#18]: https://github.com/MaxOhn/rosu-pp/pull/18
