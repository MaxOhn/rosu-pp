# pp-gen

Small script to calculate difficulty and performance points related values for a given folder containing `.osu` files.

### How to use

- Rename `.env.example` to `.env` and put proper values for both of its variables:
  - `PERF_CALC_PATH` is going to be something like `C:/osu-tools/PerformanceCalculator/bin/Release/net5.0/PerformanceCalculator.dll`
  - `MAP_PATH` is the path to the folder containing a bunch of `{map_id}.osu` files
  - `MAP_TAKE` is the amount of maps for which values will be calculated
  - `MAP_SKIP` is the amount of maps that will be skipped in the `MAP_PATH` folder after each calculation
- Run `cargo run --release`. The program will use the `PerformanceCalculator.dll` to calculate the values and then store them in `output.json`.

### Output format

The calculated values for each map for multiple different mod combinations will be stored in a JSON array. The array elements will be of the following form:
```js
{
    "score": {
        "mode": integer,
        "map_id": integer,
        "mods": [string],
        "total_score": integer,
        "acc": double,
        "combo": integer,
        "stats": {
            "perfect": integer?,
            "great": integer,
            "good": integer?,
            "ok": integer,
            "meh": integer,
            "miss": integer,
        }
    },
    "performance": {
        "aim": double?,
        "speed": double?,
        "acc": double?,
        "flashlight": double?,
        "effective_miss_count": double?,
        "scaled_score": double?,
        "difficulty": double?,
        "pp": double,
    },
    "difficulty": {
        "stars": double,
        "max_combo": integer,
        "aim": double?,
        "speed": double?,
        "flashlight": double?,
        "slider_factor": double?,
        "stamina": double?,
        "rhythm": double?,
        "colour": double?,
        "ar": double?,
        "od": double?,
        "great_hit_window": double?,
        "score_multiplier": double?,
    }
}
```
