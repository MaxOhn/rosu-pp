# pp-gen

Small script to plot value differences between `rosu-pp`'s performance attributes and `pp-gen`'s `output.json`.

### How to use

- Rename `.env.example` to `.env` and put proper values for its variables:
  - `MAP_PATH` is the path to the folder containing a bunch of `{map_id}.osu` files 
- Run `cargo run --release`. The program will read the file at `../pp-gen/output.json`, calculate `rosu-pp` values, and plot the differences in the files `accuracy_{mode}.svg`.
