use std::cmp::Ordering;

use crate::{
    curve::{Curve, CurveBuffers},
    limited_queue::LimitedQueue,
    parse::{legacy_sort, HitObjectKind, Pos2},
    Beatmap,
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

mod byte_hasher;
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

        let seed =
            (map.hp + map.cs).round() as i32 * 20 + (map.od * 41.2) as i32 + map.ar.round() as i32;
        let mut random = Random::new(seed);

        let rounded_cs = map.cs.round();
        let rounded_od = map.od.round();

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
            slider_or_spinner_count as f32 / self.hit_objects.len() as f32;

        let target_columns = if percent_slider_or_spinner < 0.2 {
            7.0
        } else if percent_slider_or_spinner < 0.3 || rounded_cs >= 5.0 {
            (6 + (rounded_od > 5.0) as u8) as f32
        } else if percent_slider_or_spinner > 0.6 {
            (4 + (rounded_od > 4.0) as u8) as f32
        } else {
            (rounded_od + 1.0).clamp(4.0, 7.0)
        };

        map.cs = target_columns;

        let mut prev_note_times = LimitedQueue::new(MAX_NOTES_FOR_DENSITY);
        let mut density = i32::MAX as f64;

        let mut compute_density = |new_note_time: f64, d: &mut f64| {
            prev_note_times.push(new_note_time);

            if prev_note_times.len() >= 2 {
                *d = (prev_note_times.last().unwrap() - prev_note_times[0])
                    / prev_note_times.len() as f64;
            }
        };

        let mut last_time = 0.0;
        let mut last_pos = Pos2::default();
        let mut last_stair = PatternType::STAIR;
        let mut last_pattern = Pattern::default();

        let mut curve_bufs = CurveBuffers::default();

        for (obj, sound) in self.hit_objects.iter().zip(self.sounds.iter()) {
            match obj.kind {
                HitObjectKind::Circle => {
                    compute_density(obj.start_time, &mut density);

                    let mut gen = HitObjectPatternGenerator::new(
                        &mut random,
                        obj,
                        *sound,
                        &map,
                        &last_pattern,
                        last_time,
                        last_pos,
                        density,
                        last_stair,
                        self,
                    );

                    let new_pattern = gen.generate();

                    last_time = obj.start_time;
                    last_pos = obj.pos;
                    last_stair = gen.stair_type;

                    map.hit_objects
                        .extend(new_pattern.hit_objects.iter().cloned());

                    n_circles += new_pattern.hit_objects.len();
                    last_pattern = new_pattern;
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
                        &map,
                        &last_pattern,
                        self,
                        repeats,
                        &curve,
                        edge_sounds,
                    );

                    let segment_duration = gen.segment_duration as f64;

                    for i in 0..=repeats as i32 + 1 {
                        let time = obj.start_time + segment_duration * i as f64;

                        last_time = time;
                        last_pos = obj.pos;

                        compute_density(time, &mut density);
                    }

                    for new_pattern in gen.generate() {
                        let new_objects = new_pattern.hit_objects.iter().map(|h| {
                            if obj.is_circle() {
                                n_circles += 1;
                            } else {
                                n_sliders += 1;
                            }

                            h.to_owned()
                        });

                        map.hit_objects.extend(new_objects);

                        last_pattern = new_pattern;
                    }
                }
                HitObjectKind::Spinner { end_time } | HitObjectKind::Hold { end_time } => {
                    let mut gen = EndTimeObjectPatternGenerator::new(
                        &mut random,
                        obj,
                        end_time,
                        *sound,
                        &map,
                        &last_pattern,
                    );

                    last_time = obj.start_time;
                    last_pos = obj.pos;

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

        // println!("Pre-sort:");

        // for h in map.hit_objects.iter() {
        //     println!("[{}] {}", h.start_time, h.column(map.cs));
        // }

        map.hit_objects
            .sort_by(|p1, p2| p1.partial_cmp(p2).unwrap_or(Ordering::Equal));

        legacy_sort(&mut map.hit_objects);

        // println!("Post-sort:");

        // for h in map.hit_objects.iter() {
        //     println!("[{}] {}", h.start_time, h.column(map.cs));
        // }

        map
    }
}
