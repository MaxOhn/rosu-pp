use crate::model::beatmap::BeatmapAttributes;

pub fn calculate_difficulty_peppy_stars(
    map_attrs: &BeatmapAttributes,
    object_count: i32,
    drain_len: i32,
) -> i32 {
    /*
     * WARNING: DO NOT TOUCH IF YOU DO NOT KNOW WHAT YOU ARE DOING
     *
     * It so happens that in stable, due to .NET Framework internals, float math would be performed
     * using x87 registers and opcodes.
     * .NET (Core) however uses SSE instructions on 32- and 64-bit words.
     * x87 registers are _80 bits_ wide. Which is notably wider than _both_ float and double.
     * Therefore, on a significant number of beatmaps, the rounding would not produce correct values.
     *
     * Thus, to crudely - but, seemingly *mostly* accurately, after checking across all ranked maps - emulate this,
     * use `decimal`, which is slow, but has bigger precision than `double`.
     * At the time of writing, there is _one_ ranked exception to this - namely https://osu.ppy.sh/beatmapsets/1156087#osu/2625853 -
     * but it is considered an "acceptable casualty", since in that case scores aren't inflated by _that_ much compared to others.
     */

    // NOTE: we use f64 instead of C#'s decimal type for simplicity reasons and
    // sacrifice precision while doing so

    let object_to_drain_ratio = if drain_len != 0 {
        (f64::from(object_count) / f64::from(drain_len) * 8.0).clamp(0.0, 16.0)
    } else {
        16.0
    };

    /*
     * Notably, THE `double` CASTS BELOW ARE IMPORTANT AND MUST REMAIN.
     * Their goal is to trick the compiler / runtime into NOT promoting from single-precision float, as doing so would prompt it
     * to attempt to "silently" fix the single-precision values when converting to decimal,
     * which is NOT what the x87 FPU does.
     */

    let drain_rate = map_attrs.hp;
    let overall_difficulty = map_attrs.od;
    let circle_size = map_attrs.cs;

    ((drain_rate + overall_difficulty + circle_size + object_to_drain_ratio) / 38.0 * 5.0)
        .round_ties_even() as i32
}
