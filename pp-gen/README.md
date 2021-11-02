# pp-gen

Small script to calculate difficulty and performance points related values for a given list of map ids.

### How to use

- Create 4 files in `/input/`: `osu.csv`, `taiko.csv`, `catch.csv`, and `mania.csv`. For all beatmaps that you want to calculate the values for, add the map's id into the file that corresponds to the map's mode. The ids must be seperated either by comma (",") or by whitespace (" ", new line, tab).
- Rename `.env.example` to `.env` and put proper values for both of its variables:
  - `PERF_CALC_PATH` is going to be something like `C:/osu-tools/PerformanceCalculator/bin/Release/net5.0/PerformanceCalculator.dll`
  - `MAP_PATH` is the path to the folder containing all relevant `.osu` files that were specified in the `.csv` files
- Since `PerformanceCalculator.dll` won't print the stars for a map simulation by default, you will need to make a tiny adjustment to a file in the osu-tools repo. In `osu-tools/PerformanceCalculator/Simulate/SimulateCommand.cs`, add the following line to the other lines that look similar:
    ```cs
    o["Stars"] = difficultyAttributes.StarRating;
    ```
    Don't forget to recompile osu-tools via `dotnet build -c Release` after making the change.
- Run `cargo run --release`. The program will use the `PerformanceCalculator.dll` to calculate the values and then store them in `/output/{mode}.json`.

### Output format

The calculated values for each map for multiple different mod combinations will be stored in a JSON array. The array elements will be of the following form:
- osu: 
  ```js
    {
        "mode": 0,
        "map_id": integer,
        "aim": float,
        "speed": float,
        "accuracy": float,
        "flashlight": float,
        "od": float,
        "ar": float,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": float,
        "pp": float,
    }
  ```
- taiko: 
  ```js
    {
        "mode": 1,
        "map_id": integer,
        "accuracy": float,
        "strain": float,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": float,
        "pp": float,
    }
  ```
- catch: 
  ```js
    {
        "mode": 2,
        "map_id": integer,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": float,
        "pp": float,
    }
  ```
- mania: 
  ```js
    {
        "mode": 3,
        "map_id": integer,
        "accuracy": float,
        "strain": float,
        "mods": "HD, HR", // comma + whitespace separated string of mod abbreviations ("None" for nomod)
        "stars": float,
        "pp": float,
    }
  ```