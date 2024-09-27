use rosu_map::{
    section::{general::GameMode, hit_objects::CurveBuffers},
    util::Pos,
};

use crate::{
    model::{
        beatmap::{Beatmap, Converted},
        hit_object::{HitObjectKind, HoldNote, Spinner},
        mode::ConvertStatus,
    },
    util::{limited_queue::LimitedQueue, random::Random, sort},
};

use self::{
    pattern::Pattern,
    pattern_generator::{
        distance_object::DistanceObjectPatternGenerator,
        end_time_object::EndTimeObjectPatternGenerator, hit_object::HitObjectPatternGenerator,
    },
    pattern_type::PatternType,
};

use super::Mania;

mod pattern;
mod pattern_generator;
mod pattern_type;

/// A [`Beatmap`] for [`Mania`] calculations.
pub type ManiaBeatmap<'a> = Converted<'a, Mania>;

const MAX_NOTES_FOR_DENSITY: usize = 7;

pub const fn check_convert(map: &Beatmap) -> ConvertStatus {
    match map.mode {
        GameMode::Osu => ConvertStatus::Conversion,
        GameMode::Mania => ConvertStatus::Noop,
        GameMode::Taiko | GameMode::Catch => ConvertStatus::Incompatible,
    }
}

pub fn try_convert(map: &mut Beatmap) -> ConvertStatus {
    match map.mode {
        GameMode::Osu => {
            convert(map);

            ConvertStatus::Conversion
        }
        GameMode::Mania => ConvertStatus::Noop,
        GameMode::Taiko | GameMode::Catch => ConvertStatus::Incompatible,
    }
}

fn convert(map: &mut Beatmap) {
    let seed = (map.hp + map.cs).round_ties_even() as i32 * 20
        + (map.od * 41.2) as i32
        + map.ar.round_ties_even() as i32;

    let mut random = Random::new(seed);

    map.cs = target_columns(map);

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
    let mut curve_bufs = CurveBuffers::default();

    // mean=668.7 | median=512
    let mut new_hit_objects = Vec::with_capacity(512);

    for (obj, sound) in map.hit_objects.iter().zip(map.hit_sounds.iter().copied()) {
        match obj.kind {
            HitObjectKind::Circle => {
                compute_density(obj.start_time, &mut density);

                let mut gen = HitObjectPatternGenerator::new(
                    &mut random,
                    obj,
                    sound,
                    total_columns,
                    &last_values,
                    density,
                    map,
                );

                let new_pattern = gen.generate();

                last_values.stair = gen.stair_type;
                last_values.time = obj.start_time;
                last_values.pos = obj.pos;

                let new_hit_objects_iter = new_pattern.hit_objects.iter().cloned();
                new_hit_objects.extend(new_hit_objects_iter);

                last_values.pattern = new_pattern;
            }
            HitObjectKind::Slider(ref slider) => {
                let curve = slider.curve(GameMode::Mania, &mut curve_bufs);

                let mut gen = DistanceObjectPatternGenerator::new(
                    &mut random,
                    obj,
                    sound,
                    total_columns,
                    &last_values.pattern,
                    map,
                    slider.repeats,
                    &curve,
                    &slider.node_sounds,
                );

                let segment_duration = f64::from(gen.segment_duration);

                for i in 0..=slider.repeats as i32 + 1 {
                    let time = obj.start_time + segment_duration * f64::from(i);

                    last_values.time = time;
                    last_values.pos = obj.pos;

                    compute_density(time, &mut density);
                }

                for new_pattern in gen.generate() {
                    new_hit_objects.extend_from_slice(&new_pattern.hit_objects);
                    last_values.pattern = new_pattern;
                }
            }
            HitObjectKind::Spinner(Spinner { duration })
            | HitObjectKind::Hold(HoldNote { duration }) => {
                let end_time = obj.start_time + duration;

                let mut gen = EndTimeObjectPatternGenerator::new(
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

                let new_pattern = gen.generate();
                new_hit_objects.extend(new_pattern.hit_objects);
            }
        }
    }

    map.hit_sounds.clear();
    map.hit_objects = new_hit_objects;
    map.hit_objects
        .sort_by(|a, b| a.start_time.total_cmp(&b.start_time));
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

fn target_columns(map: &Beatmap) -> f32 {
    let rounded_cs = map.cs.round_ties_even();
    let rounded_od = map.od.round_ties_even();

    let slider_or_spinner_count = map
        .hit_objects
        .iter()
        .filter(|h| matches!(h.kind, HitObjectKind::Slider(_) | HitObjectKind::Spinner(_)))
        .count();

    let len = map.hit_objects.len();
    let percent_slider_or_spinner = f64::from(slider_or_spinner_count as f32 / len as f32);

    if percent_slider_or_spinner < 0.2 {
        7.0
    } else if percent_slider_or_spinner < 0.3 || rounded_cs >= 5.0 {
        f32::from(6 + u8::from(rounded_od > 5.0))
    } else if percent_slider_or_spinner > 0.6 {
        f32::from(4 + u8::from(rounded_od > 4.0))
    } else {
        (rounded_od + 1.0).clamp(4.0, 7.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::float_ext::FloatExt;

    use super::*;

    #[test]
    fn convert_mania() {
        let converted = Beatmap::from_path("./resources/2785319.osu")
            .unwrap()
            .unchecked_into_converted::<Mania>();

        let map = &converted;
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
