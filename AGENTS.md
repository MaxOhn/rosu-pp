# rosu-pp

Rust port of osu!'s difficulty & performance calculators, **bit-exact** against C# `osu!` / `osu-tools PerformanceCalculator`.

The two C# repositories can be expected to be at `../osu/` and `../osu-tools/`.

## Test suite

| Test | What |
|------|------|
| `tests/difficulty.rs::basic_osu` | Single map, different mods, trivial fields only (ar, hp, hit windows, object counts) |
| `tests/difficulty.rs::ext_osu` | **n maps × m mods, bit-exact difficulty gate** |
| `tests/performance.rs::ext_osu` | **n maps × m mods, bit-exact performance gate** |
| `tests/performance.rs::ext_acc_osu` | **n maps × m mods, non-SS (non-full-combo) gate** |
| `tests/difficulty.rs::ext_mania` | **n maps × m mods, bit-exact mania difficulty gate** (star rating + max combo) |
| `tests/performance.rs::ext_mania` | **n maps × m mods, bit-exact mania performance gate** (pp + pp difficulty) |
| `tests/difficulty.rs::ext_catch` | **n maps × m mods, bit-exact catch difficulty gate** (star rating + max combo) |
| `tests/performance.rs::ext_catch` | **n maps × m mods, bit-exact catch performance gate** (pp) |
| `tests/decode.rs` | Beatmap decoding |
| `src/lib.rs` | Unit tests (decode, performance, etc.) |

All gates use **`to_bits()` equality** (no tolerance). Run: `cargo nextest run`.

## Reference gates

- Regenerate C# refs: `python scripts/gen_ext_refs.py <mode> scripts/manifest[.mode].txt tests/data/ext_refs[mode].rs` where `<mode>` is `osu` (manifest `scripts/manifest.txt`), `mania` (`scripts/manifest_mania.txt`) or `catch` (`scripts/manifest_catch.txt`), plus `python scripts/gen_ext_acc_refs.py scripts/manifest.txt tests/data/ext_acc_refs.rs`. All shell out to `dotnet run --no-build -- simulate <mode> ...` in the sibling `../osu-tools/PerformanceCalculator` repo.
- Map selection: `python scripts/select_maps.py <beatmaps_dir> [max_samples] [mode]` buckets candidates by difficulty/rate/structure per mode (mania columns come from `CircleSize`, same as both decoders).
- A gate failure means: assume the Rust port is wrong. Only regenerate refs after confirming the C# CLI output is correct by hand.
- All gates cover the same 7 fixed mod combos (NM, HR, DT, HD, FL, HD+EZ, HD+FL); the combo list is mirrored in `gen_ext_refs.py` and `gen_ext_acc_refs.py` — change both together.
- `gen_ext_acc_refs.py`'s `MAPS` dict caches `(obj, sliders, large_ticks, max_combo)` probed from C#. Replacing a `resources/*.osu` requires re-probing that row, not just re-running the generator; the `assert st["great"] == g...` sanity checks exist to catch exactly this.

## Style

- Comments from C# should be copied over as-is and prefixed with `* `; do not use `* ` as prefix for own comments
- C#'s usage of `Math.<func>` should be mirrored with `f64::<func>` instead of `<var>.<func>` (e.g. `f64::max(my_var, 1.0)` instead of `my_var.max(1.0)`)
- Files should be in LF - *not* CRLF
- Do not use em-dashes, use simple "-" instead

## Git

- Do *not* commit changes yourself - leave it to the user.
