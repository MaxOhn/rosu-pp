use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind},
    util::TandemSorter,
    Beatmap, GameMode,
};

const LEGACY_TAIKO_VELOCITY_MULTIPLIER: f32 = 1.4;
const OSU_BASE_SCORING_DIST: f32 = 100.0;

impl Beatmap {
    pub(in crate::beatmap) fn convert_to_taiko(&self) -> Self {
        let mut map = self.clone_without_hit_objects(true);
        let mut curve_bufs = CurveBuffers::default();

        map.slider_mult *= LEGACY_TAIKO_VELOCITY_MULTIPLIER as f64;

        for (obj, sound) in self.hit_objects.iter().zip(self.sounds.iter()) {
            match obj.kind {
                HitObjectKind::Circle => {
                    map.hit_objects.push(obj.to_owned());
                    map.sounds.push(*sound);
                    map.n_circles += 1;
                }
                HitObjectKind::Slider {
                    pixel_len,
                    repeats,
                    ref control_points,
                    ref edge_sounds,
                } => {
                    let curve = Curve::new(control_points, pixel_len, &mut curve_bufs);
                    let mut params = SliderParams::new(obj.start_time, repeats, &curve);

                    if map.should_convert_slider_to_taiko_hits(&mut params) {
                        let mut i = 0;
                        let mut j = obj.start_time;

                        let edge_sound_count = edge_sounds.len().max(1);

                        while j
                            <= obj.start_time + params.duration as f64 + params.tick_spacing / 8.0
                        {
                            let h = HitObject {
                                pos: Default::default(),
                                start_time: j,
                                kind: HitObjectKind::Circle,
                            };

                            map.hit_objects.push(h);
                            map.sounds.push(*edge_sounds.get(i).unwrap_or(sound));
                            map.n_circles += 1;

                            if params.tick_spacing.abs() <= f64::EPSILON {
                                break;
                            }

                            j += params.tick_spacing;
                            i = (i + 1) % edge_sound_count;
                        }
                    } else {
                        map.hit_objects.push(obj.to_owned());
                        map.sounds.push(*sound);
                        map.n_sliders += 1;
                    }
                }
                HitObjectKind::Spinner { .. } => {
                    map.hit_objects.push(obj.to_owned());
                    map.sounds.push(*sound);
                    map.n_spinners += 1;
                }
                // Pathological case; shouldn't realistically happen
                HitObjectKind::Hold { end_time } => {
                    let obj = HitObject {
                        pos: obj.pos,
                        start_time: obj.start_time,
                        kind: HitObjectKind::Spinner { end_time },
                    };

                    map.hit_objects.push(obj);
                    map.sounds.push(*sound);
                    map.n_spinners += 1;
                }
            }
        }

        // We only convert STD to TKO so we don't need to remove objects
        // with the same timestamp that would appear only in MNA

        let mut sorter = TandemSorter::new(&map.hit_objects, true);
        sorter.sort(&mut map.hit_objects);
        sorter.toggle_marks();
        sorter.sort(&mut map.sounds);

        map.mode = GameMode::Taiko;

        map
    }

    fn should_convert_slider_to_taiko_hits(&self, params: &mut SliderParams<'_>) -> bool {
        let SliderParams {
            curve,
            duration,
            repeats,
            start_time,
            tick_spacing,
        } = params;

        // * The true distance, accounting for any repeats. This ends up being the drum roll distance later
        let spans = (*repeats + 1) as f64;
        let dist = curve.dist() * spans * LEGACY_TAIKO_VELOCITY_MULTIPLIER as f64;

        let timing_point = self.timing_point_at(*start_time);
        let difficulty_point = self.difficulty_point_at(*start_time).unwrap_or_default();

        let mut beat_len = timing_point.beat_len * difficulty_point.bpm_mult;

        let slider_scoring_point_dist =
            OSU_BASE_SCORING_DIST as f64 * self.slider_mult / self.tick_rate;

        // * The velocity and duration of the taiko hit object - calculated as the velocity of a drum roll.
        let taiko_vel = slider_scoring_point_dist * self.tick_rate;
        *duration = (dist / taiko_vel * beat_len) as u32;

        let osu_vel = taiko_vel * (1000.0_f32 as f64 / beat_len);

        // * osu-stable always uses the speed-adjusted beatlength to determine the osu! velocity, but only uses it for conversion if beatmap version < 8
        if self.version >= 8 {
            beat_len = timing_point.beat_len;
        }

        // * If the drum roll is to be split into hit circles, assume the ticks are 1/8 spaced within the duration of one beat
        *tick_spacing = (beat_len / self.tick_rate).min(*duration as f64 / spans);

        *tick_spacing > 0.0 && dist / osu_vel * 1000.0 < 2.0 * beat_len
    }
}

struct SliderParams<'c> {
    curve: &'c Curve<'c>,
    duration: u32,
    repeats: usize,
    start_time: f64,
    tick_spacing: f64,
}

impl<'c> SliderParams<'c> {
    fn new(start_time: f64, repeats: usize, curve: &'c Curve<'c>) -> Self {
        Self {
            curve,
            repeats,
            start_time,
            duration: 0,
            tick_spacing: 0.0,
        }
    }
}
