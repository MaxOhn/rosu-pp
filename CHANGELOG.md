## Upcoming

Updated all modes' difficulty and performance calculation. See osu!'s newspost for more info: <https://osu.ppy.sh/home/news/2024-10-28-performance-points-star-rating-updates>

- __Breaking changes:__
  - Removed the `Converted` type. Only using `Beatmap` now.
  - Converting a `Beatmap` is now done through the methods `convert`, `convert_ref`, or `convert_mut`
  - Replaced `Difficulty::with_mode` with the methods `Difficulty::*_for_mode` to calculate for a specific mode
  - Multiple methods are now fallible with the error type `ConvertError` in case the given beatmap
    had to be converted but conversion failed. These methods include:
      - `[Mode]Performance::generate_state`
      - `[Mode]Performance::calculate`
      - `[Mode]GradualDifficulty::new`
      - `[Mode]GradualPerformance::new`
  - `OsuScoreState` no longer implements `Copy` and now has the additional fields `large_tick_hits`, 
    `small_tick_hits`, and `slider_end_hits` which are important to specify for lazer scores.
    Similarly, `ScoreState` has the additional fields `osu_large_tick_hits`, `osu_small_tick_hits`,
    and `slider_end_hits`.
  - Removed the trait methods `check_convert` and `try_convert` from `IGameMode`
  - The field `HitWindows::od` has been renamed to `od_great` and the field `HitWindows::od_ok` has been added
  - Added the field `TaikoStrains::single_color_stamina`
  - Added multiple fields to difficulty and performance attribute types:
    - `ManiaDifficultyAttributes::n_hold_notes`
    - `OsuDifficultyAttributes::aim_difficult_strain_count`
    - `OsuDifficultyAttributes::speed_difficult_strain_count`
    - `OsuDifficultyAttributes::n_large_ticks`
    - `TaikoPerformanceAttributes::estimated_unstable_rate`
    - `TaikoDifficultyAttributes::mono_stamina_factor`
    - `TaikoDifficultyAttributes::ok_hit_window`
    - Renamed `TaikoDifficultyAttributes::hit_window` to `great_hit_window`
  - The method `OsuScoreState::accuracy` now takes an `OsuScoreOrigin` as argument
  - Bumped both the dependencies `rosu-map` and `rosu-mods` to their version `0.2`

- __Additions:__
  - osu!standard and osu!mania performance calculation now differs between lazer and stable so there
    are methods like `OsuPerformance::lazer`, `Difficulty::lazer`, ... to specify a boolean. **Defaults to `true`**
  - Added the methods `large_tick_hits`, `small_tickhits`, and `slider_end_hits` for `OsuPerformance` and
    `Performance`. These are important to be specified for lazer scores.

# v1.1.0 (2024-07-10)

- __Additions:__
  - Mods no longer need to be specified through their legacy bitflags. Instead, [`rosu-mods`] is being used to accept any type that's convertible into the new `rosu-pp` type "`GameMods`". Currently, those types are:
    - `u32`
    - [`rosu_mods::GameModsLegacy`](https://docs.rs/rosu-mods/0.1.0/rosu_mods/struct.GameModsLegacy.html)
    - [`rosu_mods::GameMods`](https://docs.rs/rosu-mods/0.1.0/rosu_mods/struct.GameMods.html)
    - [`rosu_mods::GameModsIntermode`](https://docs.rs/rosu-mods/0.1.0/rosu_mods/struct.GameModsIntermode.html)
    - `&rosu_mods::GameModsIntermode`
  
  This also means that settings of mods like `DoubleTime` or `DifficultyAdjust` can now be used without having to specify clock rate or beatmap attributes manually. Additionally, the `Blinds` mod is now considered in performance calculation.

- __Performance:__
  - The `generate_state` method now stores the resulting state internally so calling it multiple times is faster. ([#34])

- __Fixes:__
  - The `od_with_mods` argument is now being used properly ([#35])

## v1.0.0 (2024-04-02)

The `rosu-pp` interface and internal structure has been rewritten completely. Fields have been
modified, builders work differently, function arguments have changed, and some types are no longer
exposed.

Additionally, osu!catch has finally been updated to match osu!lazer as closely as possible, just
like the other modes already did.

- Beatmap converts
  - Each mode now has a [ZST], i.e. `Osu`, `Taiko`, `Catch`, and `Mania`, which are used to specify
  a mode at compile-time. Most of their utility comes from the new `IGameMode` trait.
  - Each mode's calculators now require `Converted` beatmaps to make sure they're valid for the
  mode, e.g. the `TaikoPerformance` calculator no longer takes a simple `Beatmap` but a
  `Converted<'_, Taiko>` (or its alias `TaikoBeatmap<'_>`).
  - Conversion between `Beatmap` and `Converted<'_, M>` either
    - is essentially a free operation if the map's mode already matches `M`,
    - or performs the required conversion by modifying hitobjects & co,
    - or indicates an error due to incompatible mode conversion, e.g. a mania map cannot be
    converted to a taiko map
- Difficulty calculation
  - The `Difficulty` type is the core of all difficulty calculations and acts as a builder to
  specify various parameters.
  - To calculate values for a specific mode, the method `Difficulty::with_mode` will produce a
  builder to calculate values for that mode.
  - Recycled attributes are only valid if *all* difficulty parameters match, i.e. it must be on
  the same map, use the same mods, clock rate, custom beatmap attributes, and passed object count
  (and hardrock offset for osu!catch).
  - Clockrate is now always clamped between 0.01 and 100 and custom beatmap attributes are clamped
  between -20 and 20.
- Performance calculation
  - Each mode's performance calculator was renamed from `[Mode]PP` to `[Mode]Performance` and
  `AnyPP` is now called `Performance`.
  - Difficulty attributes now contain a few more values so that the beatmap is no longer necessary
  for performance calculation as long as difficulty attributes are available.
  - The functions `[Mode]Performance::new` now take an argument implementing the trait
  `Into(Mode)Performance`, i.e. either a beatmap (as before), or attributes (difficulty or
  performance). Since attributes speed up the calculation, they should be prefered whenever
  available. However, be mindful that they have been calculated for the same map and difficulty
  settings. Otherwise, the final attributes will be incorrect.
- Features
  - The `tracing` feature has been added. Its sole functionality is that errors during beatmap
  parsing will emit a `tracing` event on the `ERROR` level. If this features is not explicitely 
  enabled, parsing errors will be ignored entirely.
  - The `gradual` features has been removed. Types for gradual calculation are now always available.
  - The `sync` feature has been added. Taiko's gradual calculation types contain types that are not
  thread-safe by default. Enabling this feature will add some performance penalty but use types
  that *do* allow moving gradual calculation across threads.
  - The `compact_strains` feature is now enabled by default and causes strain values during
  difficulty calculation to be stored in a space-efficient way to prevent out-of-memory issues on
  maliciously long maps. This comes at a small performance cost.
- Misc
  - Async is no longer supported. Beatmap parsing now works through [`rosu-map`]
  which does not support async since evidently it's generally slower than regular sequential code.
  - Errors while *parsing* a beatmap will never be propagated. The only errors that will be
  propagated are those occuring while *decoding*, e.g. a file couldn't be read or other IO errors.
  Notably, this means that some content is now parsed successfully into a `Beatmap` whereas in
  previous `rosu-pp` versions it would error, e.g. an empty file is now a valid `Beatmap`.
  - Although new lazer mods such as `DifficultyAdjust` or `DoubleTime` with a custom clockrate are
  essentially supported by providing methods to specify their parameters, mods themselves are still
  specified by their bit value. This means:
    - `Daycore` will not be considered unless its clockrate change is explicitly specified
    - /!\\ `Blinds` will not be considered for osu! performance calculation /!\\
  - Most `usize` types are now `u32`, e.g. fields of `ScoreState` or `max_combo` in attributes.
  - `n_misses` has generally been renamed to `misses` for both fields and methods.
  - Types such as `NestedObjectKind`, `ManiaObject`, or `Mods` that were only used for internal
  calculations are no longer publicly exposed.

# v0.10.0 (2023-11-19)

Essentially only adjustments to the API so bindings won't need an update.

- __Additions:__
  - Added `From<u8>` impl for `GameMode`
  - Added the method `AnyPP::hitresult_priority`
  - Added the method `[Mode]PP::generate_state` which returns the score state that will be used for performance calculation ([#23])
  - The struct `SortedVec` has now an improved public interface so it can be constructed and pushed onto ([#22])
  
- __Breaking adjustments:__
  - Removed the method `HitObject::end_time` from the public api. ([#25])
  - The fields `control_points` and `edge_sounds` of `HitObjectKind::Slider` are now stored in a `Box` rather than a `Vec`. ([#26])
  - Overhauled gradual calculation. All relevant types are now gated behind the `gradual` feature which must be enabled. ([#24])
  - `*GradualDifficultyAttributes` has been renamed to `*GradualDifficulty` and `*GradualPerformanceAttributes`
    has been renamed to `*GradualPerformance`.
  - Types for gradual calculation that depend on a lifetime now have a counterpart without a lifetime that might clone
    underlying data along the way. E.g. now there is `CatchOwnedGradualDifficulty` and `[Mode]OwnedGradualPerformance`.
  - `OsuGradualDifficulty` and thus `GradualDifficulty` no longer implement `Clone`.
  - Gradual performance calculators' method `process_next_object` has been renamed to `next` and `process_next_n_objects`
    has been renamed to `nth`. They now also have the new method `last`.
  - Similar to `Iterator::nth`, gradual performance calculators' method `nth` is now zero-indexed i.e. passing `n=0`
    will process 1 object, `n=1` will process 2, and so on.

## v0.9.5 (2023-09-06)

- __Additions:__
  - Added the method `{AnyStars/AnyPP}::is_convert` which **needs** to be used if the map was manually converted beforehand

- __Adjustments:__
  - Specified clock rates can go below 0.001 again

- __Fixes:__
  - Fixed underflow for osu!std scores that were FCs but quit mid-slider
  - Fixed panic on incorrect file headers ([#21])

## v0.9.4 (2023-02-09)

- __Additions:__
  - Added the method `{TaikoPP/ManiaPP}::is_convert` which **needs** to be used if the map was manually converted beforehand

- __Adjustments:__
  - Specified clock rates can no longer go below 0.001 to prevent crashing due to memory hogging.
  - (technically breaking) The only reasons for parsing to fail are now IO errors or invalid file headers. All other `ParseError` variants have been removed and instead of throwing an error the line is just ignored.

- __Fixes:__
  - The `Beatmap::bpm` method now works properly by considering the most common beat length instead of just the first one

## v0.9.3 (2023-01-28)

- __Additions:__
  - Added the method `ScoreState::total_hits`
  - Added the trait methods `BeatmapExt::{mode}_hitobjects` which return a list of mode-specific processed `HitObject`s ([#20])

- __Fixes:__
  - Lines with invalid curve points are now ignored instead of throwing an error while parsing
  - Fixed a niche capacity overflow in curve generation

## v0.9.2 (2022-11-08)

- __Adjustments:__
  - When passing an osu!std map to `TaikoGradualDifficultyAttributes` or `ManiaGradualDifficultyAttributes`, it now automatically converts the map internally. For osu!catch it was already happening trivially.

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
[#20]: https://github.com/MaxOhn/rosu-pp/pull/20
[#21]: https://github.com/MaxOhn/rosu-pp/pull/21
[#22]: https://github.com/MaxOhn/rosu-pp/pull/22
[#23]: https://github.com/MaxOhn/rosu-pp/pull/23
[#24]: https://github.com/MaxOhn/rosu-pp/pull/24
[#25]: https://github.com/MaxOhn/rosu-pp/pull/25
[#26]: https://github.com/MaxOhn/rosu-pp/pull/26
[#34]: https://github.com/MaxOhn/rosu-pp/pull/34
[#35]: https://github.com/MaxOhn/rosu-pp/pull/35
[#36]: https://github.com/MaxOhn/rosu-pp/pull/36

[ZST]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
[`rosu-map`]: https://github.com/MaxOhn/rosu-map
[`rosu-mods`]: https://github.com/MaxOhn/rosu-mods