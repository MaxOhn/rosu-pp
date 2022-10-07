use std::{cmp::Ordering, convert::identity};

use super::OsuDifficultyAttributes;

use crate::{
    beatmap::DifficultyPoint,
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind, Pos2},
    Beatmap,
};

const LEGACY_LAST_TICK_OFFSET: f64 = 36.0;
const BASE_SCORING_DISTANCE: f64 = 100.0;

#[derive(Clone, Debug)]
pub(crate) struct OsuObject {
    pub(crate) start_time: f64,
    pub(crate) pos: Pos2,
    pub(crate) stack_height: f32,
    pub(crate) time_preempt: f64,
    pub(crate) time_fade_in: f64,
    pub(crate) kind: OsuObjectKind,
}

#[derive(Clone, Debug)]
pub(crate) enum OsuObjectKind {
    Circle,
    Slider {
        end_time: f64,
        end_pos: Pos2,
        lazy_travel_time: f64,
        lazy_end_pos: Pos2,
        nested_objects: Vec<NestedObject>,
    },
    Spinner {
        end_time: f64,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct NestedObject {
    pub(crate) pos: Pos2,
    pub(crate) start_time: f64,
    pub(crate) kind: NestedObjectKind,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum NestedObjectKind {
    Repeat,
    Tail,
    Tick,
}

pub(crate) struct ObjectParameters<'a> {
    pub(crate) map: &'a Beatmap,
    pub(crate) attributes: &'a mut OsuDifficultyAttributes,
    pub(crate) ticks: Vec<(Pos2, f64)>,
    pub(crate) curve_bufs: CurveBuffers,
}

impl OsuObject {
    const PREEMPT_MIN: f64 = 450.0;
    const TIME_PREEMPT: f64 = 600.0;
    const TIME_FADE_IN: f64 = 400.0;

    pub(crate) fn new(h: &HitObject, hr: bool, params: &mut ObjectParameters<'_>) -> Option<Self> {
        let ObjectParameters {
            map,
            attributes,
            ticks,
            curve_bufs,
        } = params;

        attributes.max_combo += 1; // hitcircle, slider head, or spinner
        let mut pos = h.pos;

        if hr {
            pos.y = 384.0 - pos.y;
        }

        let obj = match &h.kind {
            HitObjectKind::Circle => {
                attributes.n_circles += 1;

                // TODO: check if ar needs to be adjusted
                let tmp_preempt =
                    difficulty_range(map.ar as f64, 1800.0, 1200.0, Self::PREEMPT_MIN) as f32;
                let time_preempt = tmp_preempt as f64;

                // * Preempt time can go below 450ms. Normally, this is achieved via the DT mod
                // * which uniformly speeds up all animations game wide regardless of AR.
                // * This uniform speedup is hard to match 1:1, however we can at least make
                // * AR>10 (via mods) feel good by extending the upper linear function above.
                // * Note that this doesn't exactly match the AR>10 visuals as they're
                // * classically known, but it feels good.
                // * This adjustment is necessary for AR>10, otherwise TimePreempt can
                // * become smaller leading to hitcircles not fully fading in.
                let time_fade_in = 400.0 * (time_preempt / Self::PREEMPT_MIN).min(1.0);

                Self {
                    start_time: h.start_time,
                    pos,
                    stack_height: 0.0,
                    time_preempt,
                    time_fade_in,
                    kind: OsuObjectKind::Circle,
                }
            }
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
                ..
            } => {
                attributes.n_sliders += 1;

                let timing_point = map.timing_point_at(h.start_time);
                let difficulty_point = map.difficulty_point_at(h.start_time).unwrap_or_default();

                let scoring_dist =
                    BASE_SCORING_DISTANCE * map.slider_mult * difficulty_point.slider_vel;

                let vel = scoring_dist / timing_point.beat_len;

                // * prior to v8, speed multipliers don't adjust for how many ticks are generated over the same distance.
                // * this results in more (or less) ticks being generated in <v8 maps for the same time duration.
                let tick_dist_mult = if map.version < 8 {
                    let first_slider_vel = map
                        .difficulty_points
                        .first()
                        .map_or(DifficultyPoint::DEFAULT_SLIDER_VEL, |point| {
                            point.slider_vel
                        });

                    first_slider_vel.recip()
                } else {
                    1.0
                };

                let mut tick_dist = if difficulty_point.generate_ticks {
                    scoring_dist / map.tick_rate * tick_dist_mult
                } else {
                    f64::INFINITY
                };

                let span_count = (*repeats + 1) as f64;

                // Build the curve w.r.t. the control points
                let curve = Curve::new(control_points, *pixel_len, curve_bufs);

                let end_time = h.start_time + span_count * curve.dist() / vel;
                let total_duration = end_time - h.start_time;
                let span_duration = total_duration / span_count;

                // * A very lenient maximum length of a slider for ticks to be generated.
                // * This exists for edge cases such as /b/1573664 where the beatmap has
                // * been edited by the user, and should never be reached in normal usage.
                let max_len = 100_000.0;

                let len = curve.dist().min(max_len);
                tick_dist = tick_dist.clamp(0.0, len);
                let min_dist_from_end = vel * 10.0;

                let mut curr_dist = tick_dist;

                // TODO: check if ar needs to be adjusted
                let tmp_preempt =
                    difficulty_range(map.ar as f64, 1800.0, 1200.0, Self::PREEMPT_MIN) as f32;
                let head_time_preempt = tmp_preempt as f64;

                // * Preempt time can go below 450ms. Normally, this is achieved via the DT mod
                // * which uniformly speeds up all animations game wide regardless of AR.
                // * This uniform speedup is hard to match 1:1, however we can at least make
                // * AR>10 (via mods) feel good by extending the upper linear function above.
                // * Note that this doesn't exactly match the AR>10 visuals as they're
                // * classically known, but it feels good.
                // * This adjustment is necessary for AR>10, otherwise TimePreempt can
                // * become smaller leading to hitcircles not fully fading in.
                let head_time_fade_in = 400.0 * (head_time_preempt / Self::PREEMPT_MIN).min(1.0);

                ticks.clear();

                let mut nested_objects = if tick_dist != 0.0 {
                    ticks.reserve((len / tick_dist) as usize);
                    let mut nested_objects =
                        Vec::with_capacity((len * span_count / tick_dist) as usize);

                    // Ticks of the first span
                    while curr_dist < len - min_dist_from_end {
                        let progress = curr_dist / len;

                        let curr_time = h.start_time + progress * span_duration;
                        let mut curr_pos = h.pos + curve.position_at(progress);

                        if hr {
                            curr_pos.y = 384.0 - curr_pos.y;
                        }

                        let tick = NestedObject {
                            pos: curr_pos,
                            start_time: curr_time,
                            kind: NestedObjectKind::Tick,
                        };

                        nested_objects.push(tick);
                        ticks.push((curr_pos, curr_time));

                        curr_dist += tick_dist;
                    }

                    // Other spans
                    for span_idx in 1..=*repeats {
                        let progress = (span_idx % 2 == 1) as u8 as f64;
                        let span_idx_f64 = span_idx as f64;

                        // Repeat point
                        let curr_time = h.start_time + span_duration * span_idx_f64;
                        let mut curr_pos = h.pos + curve.position_at(progress);

                        if hr {
                            curr_pos.y = 384.0 - curr_pos.y;
                        }

                        let repeat = NestedObject {
                            pos: curr_pos,
                            start_time: curr_time,
                            kind: NestedObjectKind::Repeat,
                        };

                        nested_objects.push(repeat);

                        // Ticks
                        if span_idx & 1 == 1 {
                            // S-------->R | Span 0
                            //  2  4  6  8 | => span_duration = 8
                            // R<--------- | Span 1
                            // 16 14 12 10 | => offset = 1 * span_duration
                            // --------->R | Span 2
                            // 18 20 22 24 | => not reverse; simple case
                            // T<--------- | Span 3
                            // 32 30 28 26 | => offset = 3 * span_duration
                            //
                            //  n = offset + tick
                            // 26 =   24   +   2
                            // 28 =   24   +   4
                            // 30 =   24   +   6
                            // 32 =   24   +   8

                            let offset = span_idx_f64 * span_duration;

                            let tick_iter = ticks.iter().rev().zip(ticks.iter()).map(
                                |((rev_pos, _), (_, time))| {
                                    let start_time = offset + time;

                                    NestedObject {
                                        pos: *rev_pos,
                                        start_time,
                                        kind: NestedObjectKind::Tick,
                                    }
                                },
                            );

                            nested_objects.extend(tick_iter);
                        } else {
                            let tick_iter = ticks.iter().map(|(pos, time)| {
                                let start_time = time + span_duration * span_idx_f64;

                                NestedObject {
                                    pos: *pos,
                                    start_time,
                                    kind: NestedObjectKind::Tick,
                                }
                            });

                            nested_objects.extend(tick_iter);
                        }
                    }

                    nested_objects
                } else {
                    Vec::new()
                };

                // Slider tail
                let final_span_start_time = h.start_time + *repeats as f64 * span_duration;
                let final_span_end_time = (h.start_time + total_duration / 2.0)
                    .max(final_span_start_time + span_duration - LEGACY_LAST_TICK_OFFSET);

                let progress = (*repeats % 2 == 0) as u8 as f64;
                let mut end_pos = h.pos + curve.position_at(progress);

                if hr {
                    end_pos.y = 384.0 - end_pos.y;
                }

                // * we need to use the LegacyLastTick here for compatibility reasons (difficulty).
                // * it is *okay* to use this because the TailCircle is not used for any meaningful purpose in gameplay.
                // * if this is to change, we should revisit this.
                let legacy_last_tick = NestedObject {
                    pos: end_pos,
                    start_time: final_span_end_time,
                    kind: NestedObjectKind::Tail,
                };

                // On very short buzz sliders it can happen that the
                // legacy last tick is not the last object time-wise
                match nested_objects.last() {
                    Some(last) if last.start_time > final_span_end_time => {
                        let idx = nested_objects
                            .binary_search_by(|nested| {
                                nested
                                    .start_time
                                    .partial_cmp(&final_span_end_time)
                                    .unwrap_or(Ordering::Equal)
                            })
                            .map_or_else(identity, identity);

                        nested_objects.insert(idx, legacy_last_tick);
                    }
                    _ => nested_objects.push(legacy_last_tick),
                };

                attributes.max_combo += nested_objects.len();

                let last_time = nested_objects
                    .last()
                    .map_or(final_span_end_time, |nested| nested.start_time);

                let lazy_travel_time = last_time - h.start_time;
                let mut end_time_min = lazy_travel_time / span_duration;

                if end_time_min % 2.0 >= 1.0 {
                    end_time_min = 1.0 - end_time_min % 1.0;
                } else {
                    end_time_min %= 1.0;
                }

                // * temporary lazy end position until a real result can be derived.
                let mut lazy_end_pos = h.pos + curve.position_at(end_time_min);

                if hr {
                    lazy_end_pos.y = 384.0 - lazy_end_pos.y;
                }

                Self {
                    start_time: h.start_time,
                    pos,
                    stack_height: 0.0,
                    time_preempt: head_time_preempt,
                    time_fade_in: head_time_fade_in,
                    kind: OsuObjectKind::Slider {
                        end_time,
                        end_pos,
                        lazy_end_pos,
                        lazy_travel_time,
                        nested_objects,
                    },
                }
            }
            HitObjectKind::Spinner { end_time } => {
                attributes.n_spinners += 1;

                Self {
                    start_time: h.start_time,
                    pos,
                    stack_height: 0.0,
                    time_preempt: Self::TIME_PREEMPT,
                    time_fade_in: Self::TIME_FADE_IN,
                    kind: OsuObjectKind::Spinner {
                        end_time: *end_time,
                    },
                }
            }
            HitObjectKind::Hold { .. } => return None,
        };

        Some(obj)
    }

    #[inline]
    pub(crate) fn end_time(&self) -> f64 {
        match &self.kind {
            OsuObjectKind::Circle => self.start_time,
            OsuObjectKind::Slider { end_time, .. } => *end_time,
            OsuObjectKind::Spinner { end_time } => *end_time,
        }
    }

    #[inline]
    pub(crate) fn end_pos(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider { end_pos, .. } => *end_pos,
        }
    }

    #[inline]
    pub(crate) fn lazy_end_pos(&self, stack_offset: Pos2) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider { lazy_end_pos, .. } => *lazy_end_pos + stack_offset,
        }
    }

    #[inline]
    pub(crate) fn is_circle(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Circle)
    }

    #[inline]
    pub(crate) fn is_slider(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Slider { .. })
    }

    #[inline]
    pub(crate) fn is_spinner(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Spinner { .. })
    }
}

// TODO: cleanup
fn difficulty_range(difficulty: f64, min: f64, mid: f64, max: f64) -> f64 {
    if difficulty > 5.0 {
        mid + (max - mid) * (difficulty - 5.0) / 5.0
    } else if difficulty < 5.0 {
        mid - (mid - min) * (5.0 - difficulty) / 5.0
    } else {
        mid
    }
}
