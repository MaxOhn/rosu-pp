# pp-gen

Small script to calculate difficulty and performance points related values for a given list of map ids.

### How to use

- Rename `.env.example` to `.env` and put proper values for both of its variables:
  - `PERF_CALC_PATH` is going to be something like `C:/osu-tools/PerformanceCalculator/bin/Release/net5.0/PerformanceCalculator.dll`
  - `MAP_PATH` is the path to the folder containing a bunch of `{map_id}.osu` files 
- Since `PerformanceCalculator.dll` won't print the stars for a map simulation by default, you will need to make a tiny adjustment to a file in the osu-tools repo. In `osu-tools/PerformanceCalculator/Simulate/SimulateCommand.cs`, add the following line to the other lines that look similar:
    ```cs
    o["Stars"] = difficultyAttributes.StarRating;
    ```
    Don't forget to recompile osu-tools via `dotnet build -c Release` after making the change.
- Run `cargo run --release`. The program will use the `PerformanceCalculator.dll` to calculate the values and then store them in `output.json`.

### Output format

The calculated values for each map for multiple different mod combinations will be stored in a JSON array. The array elements will be of the following form:
- osu: 
  ```js
    {
        "mode": 0,
        "map_id": integer,
        "aim": double,
        "speed": double,
        "accuracy": double,
        "flashlight": double,
        "od": double,
        "ar": double,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": double,
        "pp": double,
    }
  ```
- taiko: 
  ```js
    {
        "mode": 1,
        "map_id": integer,
        "accuracy": double,
        "strain": double,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": double,
        "pp": double,
    }
  ```
- catch: 
  ```js
    {
        "mode": 2,
        "map_id": integer,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": double,
        "pp": double,
    }
  ```
- mania: 
  ```js
    {
        "mode": 3,
        "map_id": integer,
        "accuracy": double,
        "strain": double,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": double,
        "pp": double,
    }
  ```