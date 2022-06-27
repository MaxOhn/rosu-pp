use std::iter;

use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind, PathControlPoint, Pos2},
    Beatmap, GameMode,
};

const LEGACY_TAIKO_VELOCITY_MULTIPLIER: f32 = 1.4;
const OSU_BASE_SCORING_DIST: f32 = 100.0;

impl Beatmap {
    pub(in crate::beatmap) fn convert_to_taiko(&self) -> Self {
        let mut map = self.clone_without_hit_objects();
        let mut curve_bufs = CurveBuffers::default();

        for (i, (obj, sound)) in self.hit_objects.iter().zip(self.sounds.iter()).enumerate() {
            match obj.kind {
                HitObjectKind::Slider {
                    pixel_len,
                    repeats,
                    ref control_points,
                } => {
                    let mut params =
                        SliderParams::new(obj.start_time, pixel_len, repeats, control_points);

                    if self.should_convert_slider_to_taiko_hits(&mut curve_bufs, &mut params) {
                        let mut j = obj.start_time;
                        let mut count: usize = 0;

                        while j <= obj.start_time + params.duration + params.tick_spacing / 8.0 {
                            let h = HitObject {
                                pos: Pos2::default(),
                                start_time: j,
                                kind: HitObjectKind::Circle,
                            };

                            map.hit_objects.push(h);
                            count += 1;

                            // TODO: put this pre-loop
                            if params.tick_spacing.abs() <= 1e07 {
                                break;
                            }

                            j += params.tick_spacing;
                        }

                        if let Some(count) = count.checked_sub(1) {
                            map.sounds.extend(iter::repeat(*sound).take(count));
                            map.sounds[i + 1..].rotate_right(count);
                        } else {
                            map.sounds.remove(i);
                        }
                    } else {
                        map.hit_objects.push(obj.to_owned())
                    }
                }
                _ => map.hit_objects.push(obj.to_owned()),
            }
        }

        if self.mode == GameMode::MNA {
            // * Post processing step to transform mania hit objects with the same start time into strong hits
            // Difficulty calculation doesn't care about strong hits so we just delete objects with the same start time
            let mut i = map.hit_objects.len() - 1;

            while i > 0 {
                let curr_time = map.hit_objects[i].start_time;
                let mut j = i;

                while (map.hit_objects[j - 1].start_time - curr_time).abs() <= f64::EPSILON && j > 1
                {
                    j -= 1;
                }

                let to_delete = i - j;

                if to_delete > 0 {
                    map.hit_objects[j + 1..].rotate_left(to_delete);
                    map.hit_objects.truncate(map.hit_objects.len() - to_delete);

                    map.sounds[j + 1..].rotate_left(to_delete);
                    map.sounds.truncate(map.sounds.len() - to_delete);

                    i -= to_delete;
                }

                i -= 1;
            }
        }

        map
    }

    fn should_convert_slider_to_taiko_hits(
        &self,
        curve_bufs: &mut CurveBuffers,
        params: &mut SliderParams<'_>,
    ) -> bool {
        let SliderParams {
            control_points,
            duration,
            pixel_len,
            repeats,
            start_time,
            tick_spacing,
        } = params;

        let curve = Curve::new(*control_points, *pixel_len, curve_bufs);

        // * The true distance, accounting for any repeats. This ends up being the drum roll distance later
        let spans = (*repeats + 1) as f64;
        let dist = curve.dist() * spans * LEGACY_TAIKO_VELOCITY_MULTIPLIER as f64;

        let timing_point = self.timing_point_at(*start_time).unwrap_or_default();
        let difficulty_point = self.difficulty_point_at(*start_time).unwrap_or_default();

        let mut beat_len = timing_point.beat_len / difficulty_point.speed_multiplier;

        let slider_scoring_point_dist =
            OSU_BASE_SCORING_DIST as f64 * self.slider_mult / self.tick_rate;

        // * The velocity and duration of the taiko hit object - calculated as the velocity of a drum roll.
        let taiko_vel = slider_scoring_point_dist * self.tick_rate;
        *duration = (dist / taiko_vel * beat_len).floor();

        let osu_vel = taiko_vel * (1000.0_f32 as f64 / beat_len);

        // * osu-stable always uses the speed-adjusted beatlength to determine the osu! velocity, but only uses it for conversion if beatmap version < 8
        if self.version >= 8 {
            beat_len = timing_point.beat_len;
        }

        // * If the drum roll is to be split into hit circles, assume the ticks are 1/8 spaced within the duration of one beat
        *tick_spacing = (beat_len / self.tick_rate).min(*duration / spans);

        *tick_spacing > 0.0 && dist / osu_vel * 1000.0 < 2.0 * beat_len
    }
}

struct SliderParams<'c> {
    control_points: &'c [PathControlPoint],
    duration: f64,
    pixel_len: f64,
    repeats: usize,
    start_time: f64,
    tick_spacing: f64,
}

impl<'c> SliderParams<'c> {
    fn new(
        start_time: f64,
        pixel_len: f64,
        repeats: usize,
        control_points: &'c [PathControlPoint],
    ) -> Self {
        Self {
            control_points,
            pixel_len,
            repeats,
            start_time,
            duration: 0.0,
            tick_spacing: 0.0,
        }
    }
}
