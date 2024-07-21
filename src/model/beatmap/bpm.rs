use std::collections::HashMap;

use crate::model::{control_point::TimingPoint, hit_object::HitObject};

pub fn bpm(last_hit_object: Option<&HitObject>, timing_points: &[TimingPoint]) -> f64 {
    // This is incorrect if the last object is a slider since there
    // is no reasonable way to get the slider end time at this point.
    let last_time = last_hit_object
        .map(HitObject::end_time)
        .or_else(|| timing_points.last().map(|t| t.time))
        .unwrap_or(0.0);

    let mut bpm_points = BeatLenDuration::new(last_time);

    // * osu-stable forced the first control point to start at 0.
    // * This is reproduced here to maintain compatibility around
    // * osu!mania scroll speed and song select display.
    match timing_points {
        [curr] => bpm_points.add(curr.beat_len, 0.0, last_time),
        [curr, next, ..] => bpm_points.add(curr.beat_len, 0.0, next.time),
        [] => {}
    }

    timing_points
        .iter()
        .skip(1)
        .zip(timing_points.iter().skip(2).map(|t| t.time))
        .for_each(|(curr, next_time)| bpm_points.add(curr.beat_len, curr.time, next_time));

    if let [.., _, curr] = timing_points {
        bpm_points.add(curr.beat_len, curr.time, last_time);
    }

    let most_common_beat_len = bpm_points
        .map
        .into_iter()
        // * Get the most common one, or 0 as a suitable default
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map_or(0.0, |(beat_len, _)| f64::from_bits(beat_len));

    60_000.0 / most_common_beat_len
}

/// Maps `beat_len` to a cumulative duration
struct BeatLenDuration {
    last_time: f64,
    map: HashMap<u64, f64>,
}

impl BeatLenDuration {
    fn new(last_time: f64) -> Self {
        Self {
            last_time,
            map: HashMap::default(),
        }
    }

    fn add(&mut self, beat_len: f64, curr_time: f64, next_time: f64) {
        let beat_len = (1000.0 * beat_len).round() / 1000.0;
        let entry = self.map.entry(beat_len.to_bits()).or_default();

        if curr_time <= self.last_time {
            *entry += next_time - curr_time;
        }
    }
}
