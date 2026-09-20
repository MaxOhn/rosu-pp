# rosu-pp

Rust port of osu!'s difficulty & performance calculators, **bit-exact** against C# `osu!` / `osu-tools PerformanceCalculator`.

## Test suite

| Test | What |
|------|------|
| `tests/difficulty.rs::basic_osu` | Single map, different mods, trivial fields only (ar, hp, hit windows, object counts) |
| `tests/difficulty.rs::ext_osu` | **n maps × m mods, bit-exact difficulty gate** |
| `tests/performance.rs::ext_osu` | **n maps × m mods, bit-exact performance gate** |
| `tests/performance.rs::ext_acc_osu` | **n maps × m mods, non-SS (non-full-combo) gate** |
| `tests/decode.rs` | Beatmap decoding |
| `src/lib.rs` | Unit tests (decode, performance, etc.) |

All gates use **`to_bits()` equality** (no tolerance). Run: `cargo nextest run`.

## Reference gates

- Regenerate C# refs: `python scripts/gen_ext_refs.py scripts/manifest.txt tests/data/ext_refs.rs` and `python scripts/gen_ext_acc_refs.py scripts/manifest.txt tests/data/ext_acc_refs.rs`. Both shell out to `dotnet run --no-build -- simulate osu ...` in the sibling `../osu-tools/PerformanceCalculator` repo.
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
