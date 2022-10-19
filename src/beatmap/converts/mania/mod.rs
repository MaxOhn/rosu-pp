use std::cmp::Ordering;

use crate::{
    curve::{Curve, CurveBuffers},
    parse::{legacy_sort, HitObjectKind, Pos2},
    util::{FloatExt, LimitedQueue},
    Beatmap, GameMode,
};

use self::{
    legacy_random::Random,
    pattern::Pattern,
    pattern_generator::{
        distance_object::DistanceObjectPatternGenerator,
        end_time_object::EndTimeObjectPatternGenerator, hit_object::HitObjectPatternGenerator,
    },
    pattern_type::PatternType,
};

mod legacy_random;
mod pattern;
mod pattern_generator;
mod pattern_type;

const MAX_NOTES_FOR_DENSITY: usize = 7;

impl Beatmap {
    pub(in crate::beatmap) fn convert_to_mania(&self) -> Self {
        let mut map = self.clone_without_hit_objects(false);

        let mut n_circles = 0;
        let mut n_sliders = 0;

        let seed = (map.hp + map.cs).round_even() as i32 * 20
            + (map.od * 41.2) as i32
            + map.ar.round_even() as i32;

        let mut random = Random::new(seed);

        let rounded_cs = map.cs.round_even();
        let rounded_od = map.od.round_even();

        let slider_or_spinner_count = self
            .hit_objects
            .iter()
            .filter(|h| {
                matches!(
                    h.kind,
                    HitObjectKind::Slider { .. } | HitObjectKind::Spinner { .. }
                )
            })
            .count();

        let percent_slider_or_spinner =
            (slider_or_spinner_count as f32 / self.hit_objects.len() as f32) as f64;

        let target_columns = if percent_slider_or_spinner < 0.2 {
            7.0
        } else if percent_slider_or_spinner < 0.3 || rounded_cs >= 5.0 {
            (6 + (rounded_od > 5.0) as u8) as f32
        } else if percent_slider_or_spinner as f64 > 0.6 {
            (4 + (rounded_od > 4.0) as u8) as f32
        } else {
            (rounded_od + 1.0).clamp(4.0, 7.0)
        };

        map.cs = target_columns;

        let mut prev_note_times: LimitedQueue<f64, MAX_NOTES_FOR_DENSITY> = LimitedQueue::new();
        let mut density = i32::MAX as f64;

        let mut compute_density = |new_note_time: f64, d: &mut f64| {
            prev_note_times.push(new_note_time);

            if prev_note_times.len() >= 2 {
                *d = (prev_note_times.last().unwrap() - prev_note_times[0])
                    / prev_note_times.len() as f64;
            }
        };

        let total_columns = map.cs as i32;
        let mut last_values = PrevValues::default();
        let mut curve_bufs = CurveBuffers::default();

        for (obj, sound) in self.hit_objects.iter().zip(self.sounds.iter()) {
            match obj.kind {
                HitObjectKind::Circle => {
                    compute_density(obj.start_time, &mut density);

                    let mut gen = HitObjectPatternGenerator::new(
                        &mut random,
                        obj,
                        *sound,
                        total_columns,
                        &last_values,
                        density,
                        self,
                    );

                    let new_pattern = gen.generate();

                    last_values.stair = gen.stair_type;
                    last_values.time = obj.start_time;
                    last_values.pos = obj.pos;

                    let new_hit_objects = new_pattern.hit_objects.iter().cloned();
                    map.hit_objects.extend(new_hit_objects);

                    n_circles += new_pattern.hit_objects.len();
                    last_values.pattern = new_pattern;
                }
                HitObjectKind::Slider {
                    pixel_len,
                    repeats,
                    ref control_points,
                    ref edge_sounds,
                } => {
                    let curve = Curve::new(control_points, pixel_len, &mut curve_bufs);

                    let mut gen = DistanceObjectPatternGenerator::new(
                        &mut random,
                        obj,
                        *sound,
                        total_columns,
                        &last_values.pattern,
                        self,
                        repeats,
                        &curve,
                        edge_sounds,
                    );

                    let segment_duration = gen.segment_duration as f64;

                    for i in 0..=repeats as i32 + 1 {
                        let time = obj.start_time + segment_duration * i as f64;

                        last_values.time = time;
                        last_values.pos = obj.pos;

                        compute_density(time, &mut density);
                    }

                    for new_pattern in gen.generate() {
                        let new_objects = new_pattern.hit_objects.iter().map(|h| {
                            if h.is_circle() {
                                n_circles += 1;
                            } else {
                                n_sliders += 1;
                            }

                            h.to_owned()
                        });

                        map.hit_objects.extend(new_objects);

                        last_values.pattern = new_pattern;
                    }
                }
                HitObjectKind::Spinner { end_time } | HitObjectKind::Hold { end_time } => {
                    let mut gen = EndTimeObjectPatternGenerator::new(
                        &mut random,
                        obj,
                        end_time,
                        *sound,
                        total_columns,
                        &last_values.pattern,
                    );

                    last_values.time = end_time;
                    last_values.pos = Pos2 { x: 256.0, y: 192.0 };

                    compute_density(end_time, &mut density);

                    let new_pattern = gen.generate();

                    let new_objects = new_pattern.hit_objects.into_iter().inspect(|h| {
                        if h.is_circle() {
                            n_circles += 1;
                        } else {
                            n_sliders += 1;
                        }
                    });

                    map.hit_objects.extend(new_objects);
                }
            }
        }

        map.n_circles = n_circles as u32;
        map.n_sliders = n_sliders;

        map.hit_objects
            .sort_by(|p1, p2| p1.partial_cmp(p2).unwrap_or(Ordering::Equal));

        legacy_sort(&mut map.hit_objects);

        map.mode = GameMode::Mania;

        map
    }
}

pub(crate) struct PrevValues {
    time: f64,
    pos: Pos2,
    pattern: Pattern,
    stair: PatternType,
}

impl Default for PrevValues {
    fn default() -> Self {
        Self {
            time: 0.0,
            pos: Pos2::default(),
            pattern: Pattern::default(),
            stair: PatternType::STAIR,
        }
    }
}
