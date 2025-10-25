use std::cmp::Ordering;

use rosu_map::{section::general::GameMode, util::Pos};

use crate::{
    GameMods,
    mania::object::ManiaObject,
    model::{
        beatmap::Beatmap,
        control_point::TimingPoint,
        hit_object::{HitObject, HitObjectKind, HoldNote, Spinner},
    },
    util::{
        limited_queue::LimitedQueue,
        random::{csharp::Random as CsharpRandom, osu::Random as OsuRandom},
        sort,
    },
};

use self::{
    pattern::Pattern,
    pattern_generator::{
        end_time_object::EndTimeObjectPatternGenerator, hit_object::HitObjectPatternGenerator,
        path_object::PathObjectPatternGenerator,
    },
    pattern_type::PatternType,
};

mod pattern;
mod pattern_generator;
mod pattern_type;

const MAX_NOTES_FOR_DENSITY: usize = 7;

pub fn convert(map: &mut Beatmap, mods: &GameMods) {
    let seed = (map.hp + map.cs).round_ties_even() as i32 * 20
        + (map.od * 41.2) as i32
        + map.ar.round_ties_even() as i32;

    let mut random = OsuRandom::new(seed);

    map.cs = target_columns(map, mods);

    let mut prev_note_times = LimitedQueue::<f64, MAX_NOTES_FOR_DENSITY>::new();
    let mut density = f64::from(i32::MAX);

    let mut compute_density = |new_note_time: f64, d: &mut f64| {
        prev_note_times.push(new_note_time);

        if let ([first, ..], [.., last]) | ([], [first, .., last]) | ([first, .., last], []) =
            prev_note_times.as_slices()
        {
            *d = (last - first) / prev_note_times.len() as f64;
        }
    };

    let total_columns = map.cs as i32;
    let mut last_values = PrevValues::default();

    // mean=668.7 | median=512
    let mut new_hit_objects = Vec::with_capacity(512);

    for (obj, sound) in map.hit_objects.iter().zip(map.hit_sounds.iter().copied()) {
        match obj.kind {
            HitObjectKind::Circle => {
                compute_density(obj.start_time, &mut density);

                let mut generator = HitObjectPatternGenerator::new(
                    &mut random,
                    obj,
                    sound,
                    total_columns,
                    &last_values,
                    density,
                    map,
                );

                let new_pattern = generator.generate();

                last_values.stair = generator.stair_type;
                last_values.time = obj.start_time;
                last_values.pos = obj.pos;

                let new_hit_objects_iter = new_pattern.hit_objects.iter().cloned();
                new_hit_objects.extend(new_hit_objects_iter);

                last_values.pattern = new_pattern;
            }
            HitObjectKind::Slider(ref slider) => {
                let mut generator = PathObjectPatternGenerator::new(
                    &mut random,
                    obj,
                    sound,
                    total_columns,
                    &last_values.pattern,
                    map,
                    slider.repeats,
                    slider.expected_dist,
                    &slider.node_sounds,
                );

                let segment_duration = f64::from(generator.segment_duration);

                for i in 0..=slider.repeats as i32 + 1 {
                    let time = obj.start_time + segment_duration * f64::from(i);

                    last_values.time = time;
                    last_values.pos = obj.pos;

                    compute_density(time, &mut density);
                }

                for new_pattern in generator.generate() {
                    new_hit_objects.extend_from_slice(&new_pattern.hit_objects);
                    last_values.pattern = new_pattern;
                }
            }
            HitObjectKind::Spinner(Spinner { duration })
            | HitObjectKind::Hold(HoldNote { duration }) => {
                let end_time = obj.start_time + duration;

                let mut generator = EndTimeObjectPatternGenerator::new(
                    &mut random,
                    obj,
                    end_time,
                    sound,
                    total_columns,
                    &last_values.pattern,
                    map,
                );

                last_values.time = end_time;
                last_values.pos = Pos::new(256.0, 192.0);

                compute_density(end_time, &mut density);

                let new_pattern = generator.generate();
                new_hit_objects.extend(new_pattern.hit_objects);
            }
        }
    }

    map.hit_sounds.clear();
    map.hit_objects = new_hit_objects;
    map.hit_objects.sort_by(cmp_by_start_time);
    sort::osu_legacy(&mut map.hit_objects);

    map.mode = GameMode::Mania;
    map.is_convert = true;
}

pub struct PrevValues {
    time: f64,
    pos: Pos,
    pattern: Pattern,
    stair: PatternType,
}

impl Default for PrevValues {
    fn default() -> Self {
        Self {
            time: 0.0,
            pos: Pos::default(),
            pattern: Pattern::default(),
            stair: PatternType::STAIR,
        }
    }
}

fn target_columns(map: &Beatmap, mods: &GameMods) -> f32 {
    if let Some(keys) = mods.mania_keys() {
        return keys;
    }

    let rounded_cs = map.cs.round_ties_even();
    let rounded_od = map.od.round_ties_even();

    if !map.hit_objects.is_empty() {
        let count_slider_or_spinner = map
            .hit_objects
            .iter()
            .filter(|h| matches!(h.kind, HitObjectKind::Slider(_) | HitObjectKind::Spinner(_)))
            .count();

        let len = map.hit_objects.len();

        // * In osu!stable, this division appears as if it happens on floats, but due to release-mode
        // * optimisations, it actually ends up happening on doubles.
        let percent_slider_or_spinner = count_slider_or_spinner as f64 / len as f64;

        if percent_slider_or_spinner < 0.2 {
            return 7.0;
        } else if percent_slider_or_spinner < 0.3 || rounded_cs >= 5.0 {
            return f32::from(6 + u8::from(rounded_od > 5.0));
        } else if percent_slider_or_spinner > 0.6 {
            return f32::from(4 + u8::from(rounded_od > 4.0));
        }
    }

    #[allow(clippy::manual_clamp, reason = "keeping it in-sync with lazer")]
    {
        ((rounded_od as i32) + 1).min(7).max(4) as f32
    }
}

pub(super) fn apply_hold_off_to_beatmap(map: &mut Beatmap) {
    let new_hit_objects_iter = map.hit_objects.iter().filter_map(|h| {
        if h.is_hold_note() {
            Some(HitObject {
                pos: h.pos,
                start_time: h.start_time,
                kind: HitObjectKind::Circle,
            })
        } else {
            None
        }
    });

    let old_hit_objects_iter = map.hit_objects.iter().filter_map(|h| {
        if h.is_circle() {
            Some(HitObject {
                pos: h.pos,
                start_time: h.start_time,
                kind: HitObjectKind::Circle,
            })
        } else {
            None
        }
    });

    let mut new_hit_objects = Vec::with_capacity(map.hit_objects.len());
    new_hit_objects.extend(old_hit_objects_iter.chain(new_hit_objects_iter));
    map.hit_objects = new_hit_objects;
    map.hit_sounds.clear();
    map.hit_objects.sort_by(cmp_by_start_time);
}

pub(super) fn apply_invert_to_beatmap(map: &mut Beatmap) {
    let mut new_objects = Vec::with_capacity(map.hit_objects.len());
    let mut column_buf = Vec::new();
    let mut locations = Vec::new();

    let total_columns = map.cs;

    for column in 0..total_columns as usize {
        let iter = map
            .hit_objects
            .iter()
            .filter(|h| ManiaObject::column(h.pos.x, total_columns) == column);

        column_buf.clear();
        column_buf.extend(iter);

        let notes = column_buf
            .iter()
            .filter_map(|h| h.is_circle().then_some(h.start_time));

        let hold_notes = column_buf
            .iter()
            .filter_map(|h| match h.kind {
                HitObjectKind::Hold(hold) => Some([h.start_time, h.start_time + hold.duration]),
                _ => None,
            })
            .flatten();

        locations.clear();
        locations.extend(notes.chain(hold_notes));
        locations.sort_by(f64::total_cmp);

        let iter = locations.windows(2).map(|window| {
            let [start_time, end_time] = *window else {
                unreachable!()
            };

            // * Full duration of the hold note.
            let mut duration = end_time - start_time;

            // * Beat length at the end of the hold note.
            let beat_length = map
                .timing_point_at(end_time)
                .map_or(TimingPoint::DEFAULT_BEAT_LEN, |tp| tp.beat_len);

            // * Decrease the duration by at most a 1/4 beat to ensure there's no instantaneous notes.
            duration = f64::max(duration / 2.0, duration - beat_length / 4.0);

            HitObject {
                pos: column_buf[0].pos,
                start_time,
                kind: HitObjectKind::Hold(HoldNote { duration }),
            }
        });

        new_objects.extend(iter);
    }

    map.hit_objects = new_objects;
    map.hit_sounds.clear();
    map.hit_objects.sort_by(cmp_by_start_time);

    // * No breaks
    map.breaks.clear();
}

pub(super) fn apply_random_to_beatmap(map: &mut Beatmap, seed: i32) {
    let mut rng = CsharpRandom::new(seed);

    let total_columns = map.cs;
    let available_columns = total_columns as u8;
    let mut shuffled_columns: Vec<_> = (0..available_columns).collect();

    shuffled_columns.sort_by_cached_key(|_| rng.next());
    let divisor = 512.0 / total_columns;

    for h in map.hit_objects.iter_mut() {
        let old_column = ManiaObject::column(h.pos.x, total_columns);
        let new_column = shuffled_columns[old_column];
        h.pos.x = f32::ceil(f32::from(new_column) * divisor);
    }
}

fn cmp_by_start_time(a: &HitObject, b: &HitObject) -> Ordering {
    a.start_time.total_cmp(&b.start_time)
}

#[cfg(test)]
mod tests {
    use crate::util::float_ext::FloatExt;

    use super::*;

    #[test]
    fn convert_mania() {
        let map = Beatmap::from_path("./resources/2785319.osu").unwrap();
        let map = map.convert(GameMode::Mania, &GameMods::default()).unwrap();
        assert!(map.is_convert);

        assert_eq!(map.mode, GameMode::Mania);
        assert_eq!(map.version, 14);
        assert!(map.ar.eq(9.3), "{} != 9.3", map.ar);
        assert!(map.od.eq(8.8), "{} != 8.8", map.od);
        assert!(map.cs.eq(7.0), "{} != 7.0", map.cs);
        assert!(map.hp.eq(5.0), "{} != 5.0", map.hp);
        assert!(
            map.slider_multiplier.eq(1.7),
            "{} != 1.7",
            map.slider_multiplier
        );
        assert!(
            map.slider_tick_rate.eq(1.0),
            "{} != 1.0",
            map.slider_tick_rate
        );
        assert_eq!(map.hit_objects.len(), 1046);
        assert_eq!(map.hit_sounds.len(), 0);
        assert_eq!(map.timing_points.len(), 1);
        assert_eq!(map.difficulty_points.len(), 50);
        assert_eq!(map.effect_points.len(), 0);
        assert!(map.stack_leniency.eq(0.5), "{} != 0.5", map.stack_leniency);
        assert_eq!(map.breaks.len(), 1);
    }
}
