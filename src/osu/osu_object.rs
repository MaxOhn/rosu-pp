use std::slice::Iter;

use super::{scaling_factor::ScalingFactor, OsuDifficultyAttributes, PLAYFIELD_BASE_SIZE};

use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind, Pos2},
    Beatmap,
};

const LEGACY_LAST_TICK_OFFSET: f64 = 36.0;
const BASE_SCORING_DISTANCE: f64 = 100.0;

#[derive(Clone, Debug)]
pub(crate) struct OsuObject {
    pos: Pos2,
    pub(crate) start_time: f64,
    pub(crate) stack_offset: Pos2,
    pub(crate) stack_height: f32,
    pub(crate) kind: OsuObjectKind,
}

#[derive(Clone, Debug)]
pub(crate) enum OsuObjectKind {
    Circle,
    Slider(OsuSlider),
    Spinner { end_time: f64 },
}

#[derive(Clone, Debug)]
pub(crate) struct OsuSlider {
    pub(crate) end_time: f64,
    pub(crate) lazy_end_pos: Pos2,
    nested_objects: Vec<NestedObject>,
}

impl OsuSlider {
    pub(crate) fn nested_len(&self) -> usize {
        self.nested_objects.len()
    }

    pub(crate) fn nested_iter(&self) -> Iter<'_, NestedObject> {
        self.nested_objects.iter()
    }

    pub(crate) fn repeat_count(&self) -> usize {
        self.nested_objects.iter().fold(0, |count, nested| {
            count + matches!(nested.kind, NestedObjectKind::Repeat) as usize
        })
    }

    pub(crate) fn end_pos(&self) -> Option<Pos2> {
        self.tail().map(|tail| tail.pos)
    }

    pub(crate) fn tail(&self) -> Option<&NestedObject> {
        self.nested_objects
            .iter()
            .rev()
            .find(|nested| matches!(nested.kind, NestedObjectKind::Tail))
    }

    pub(crate) fn tail_mut(&mut self) -> Option<(usize, &mut NestedObject)> {
        self.nested_objects
            .iter_mut()
            .enumerate()
            .rev()
            .find(|(_, nested)| matches!(nested.kind, NestedObjectKind::Tail))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NestedObject {
    /// Note: `pos` does not include stacking!
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
    pub(crate) attrs: &'a mut OsuDifficultyAttributes,
    pub(crate) ticks: Vec<(Pos2, f64)>,
    pub(crate) curve_bufs: CurveBuffers,
}

impl OsuObject {
    pub(crate) fn new(h: &HitObject, params: &mut ObjectParameters<'_>) -> Self {
        let ObjectParameters {
            map,
            attrs,
            ticks,
            curve_bufs,
        } = params;

        attrs.max_combo += 1; // hitcircle, slider head, or spinner
        let pos = h.pos;

        match &h.kind {
            HitObjectKind::Circle => {
                attrs.n_circles += 1;

                Self {
                    start_time: h.start_time,
                    pos,
                    stack_offset: Pos2::default(),
                    stack_height: 0.0,
                    kind: OsuObjectKind::Circle,
                }
            }
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
                ..
            } => {
                attrs.n_sliders += 1;

                let timing_point = map.timing_point_at(h.start_time);
                let difficulty_point = map.difficulty_point_at(h.start_time).unwrap_or_default();

                let scoring_dist =
                    BASE_SCORING_DISTANCE * map.slider_mult * difficulty_point.slider_vel;

                let vel = scoring_dist / timing_point.beat_len;

                // * prior to v8, speed multipliers don't adjust for how many ticks are generated over the same distance.
                // * this results in more (or less) ticks being generated in <v8 maps for the same time duration.
                let tick_dist_mult = if map.version < 8 {
                    difficulty_point.slider_vel.recip()
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

                ticks.clear();

                let mut nested_objects = if tick_dist != 0.0 {
                    ticks.reserve((len / tick_dist) as usize);
                    let mut nested_objects =
                        Vec::with_capacity((len * span_count / tick_dist) as usize);

                    // Ticks of the first span
                    while curr_dist < len - min_dist_from_end {
                        let progress = curr_dist / len;

                        let curr_time = h.start_time + progress * span_duration;
                        let curr_pos = h.pos + curve.position_at(progress);

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
                        let curr_pos = h.pos + curve.position_at(progress);

                        let repeat = NestedObject {
                            pos: curr_pos,
                            start_time: curr_time,
                            kind: NestedObjectKind::Repeat,
                        };

                        nested_objects.push(repeat);
                        let span_offset = span_idx_f64 * span_duration;

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

                            let base = h.start_time + h.start_time + span_duration;

                            let tick_iter = ticks.iter().rev().map(|(pos, time)| NestedObject {
                                pos: *pos,
                                start_time: span_offset + base - time,
                                kind: NestedObjectKind::Tick,
                            });

                            nested_objects.extend(tick_iter);
                        } else {
                            let tick_iter = ticks.iter().map(|(pos, time)| NestedObject {
                                pos: *pos,
                                start_time: time + span_offset,
                                kind: NestedObjectKind::Tick,
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
                let end_pos = curve.position_at(progress);

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
                            .iter()
                            .rev()
                            .position(|nested| nested.start_time <= final_span_end_time)
                            .map_or(0, |i| nested_objects.len() - i);

                        nested_objects.insert(idx, legacy_last_tick);
                    }
                    _ => nested_objects.push(legacy_last_tick),
                };

                attrs.max_combo += nested_objects.len();

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
                // The position is added after the stacking for the correct order of
                // floating point operations.
                let lazy_end_pos = curve.position_at(end_time_min);

                let slider = OsuSlider {
                    end_time,
                    lazy_end_pos,
                    nested_objects,
                };

                Self {
                    start_time: h.start_time,
                    pos,
                    stack_offset: Pos2::default(),
                    stack_height: 0.0,
                    kind: OsuObjectKind::Slider(slider),
                }
            }
            HitObjectKind::Spinner { end_time } | HitObjectKind::Hold { end_time } => {
                attrs.n_spinners += 1;

                Self {
                    start_time: h.start_time,
                    pos,
                    stack_offset: Pos2::default(),
                    stack_height: 0.0,
                    kind: OsuObjectKind::Spinner {
                        end_time: *end_time,
                    },
                }
            }
        }
    }

    pub(crate) fn end_time(&self) -> f64 {
        match &self.kind {
            OsuObjectKind::Circle => self.start_time,
            OsuObjectKind::Slider(slider) => slider.end_time,
            OsuObjectKind::Spinner { end_time } => *end_time,
        }
    }

    pub(crate) fn end_pos(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider(slider) => slider.end_pos().unwrap_or(self.pos),
        }
    }

    pub(crate) fn pre_stacked_end_pos(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider(slider) => slider
                .end_pos()
                .map_or(self.pos, |end_pos| self.pos + end_pos),
        }
    }

    pub(crate) fn old_stacking_pos2(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.pos,
            OsuObjectKind::Slider(slider) => {
                // Old stacking requires the path end position
                // instead of slider end position
                let repeat_count = slider.repeat_count();

                if repeat_count % 2 == 0 {
                    slider
                        .end_pos()
                        .map_or(self.pos, |end_pos| self.pos + end_pos)
                } else {
                    slider
                        .nested_iter()
                        .find(|nested| matches!(nested.kind, NestedObjectKind::Repeat))
                        .map_or(self.pos, |repeat| repeat.pos)
                }
            }
        }
    }

    pub(crate) const fn pos(&self) -> Pos2 {
        self.pos
    }

    pub(crate) fn stacked_pos(&self) -> Pos2 {
        self.pos + self.stack_offset
    }

    pub(crate) fn stacked_end_pos(&self) -> Pos2 {
        self.end_pos() + self.stack_offset
    }

    pub(crate) fn lazy_end_pos(&self) -> Pos2 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => self.stacked_pos(),
            OsuObjectKind::Slider(slider) => slider.lazy_end_pos,
        }
    }

    pub(crate) fn lazy_travel_time(&self) -> f64 {
        match &self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => 0.0,
            OsuObjectKind::Slider(slider) => slider
                .nested_objects
                .last()
                .map_or(0.0, |nested| nested.start_time - self.start_time),
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

    /// Applies stack offset, flips playfield for HR,
    /// and adjusts slider tails and lazy_end_positions.
    pub(crate) fn post_process(&mut self, hr: bool, scaling_factor: &ScalingFactor) {
        self.stack_offset = scaling_factor.stack_offset(self.stack_height);
        let pos = self.pos();

        if let OsuObjectKind::Slider(slider) = &mut self.kind {
            if hr {
                let mut lazy_end_pos = pos;
                lazy_end_pos.y = PLAYFIELD_BASE_SIZE.y - lazy_end_pos.y;
                lazy_end_pos += self.stack_offset;

                lazy_end_pos += Pos2 {
                    x: slider.lazy_end_pos.x,
                    y: -slider.lazy_end_pos.y,
                };

                slider.lazy_end_pos = lazy_end_pos;

                let tail_idx = slider.tail_mut().map(|(tail_idx, tail)| {
                    let mut tail_pos = pos;
                    tail_pos.y = PLAYFIELD_BASE_SIZE.y - tail_pos.y;

                    tail_pos += Pos2 {
                        x: tail.pos.x,
                        y: -tail.pos.y,
                    };

                    tail.pos = tail_pos;

                    tail_idx
                });

                if let Some(tail_idx) = tail_idx {
                    for nested in slider.nested_objects[..tail_idx].iter_mut() {
                        nested.pos.y = PLAYFIELD_BASE_SIZE.y - nested.pos.y;
                    }

                    for nested in slider.nested_objects[tail_idx + 1..].iter_mut() {
                        nested.pos.y = PLAYFIELD_BASE_SIZE.y - nested.pos.y;
                    }
                } else {
                    // Should never happen since sliders are bound to have a tail
                    for nested in slider.nested_objects.iter_mut() {
                        nested.pos.y = PLAYFIELD_BASE_SIZE.y - nested.pos.y;
                    }
                }
            } else {
                slider.lazy_end_pos += pos + self.stack_offset;

                if let Some((_, tail)) = slider.tail_mut() {
                    tail.pos += pos;
                }
            }
        }

        if hr {
            self.pos.y = PLAYFIELD_BASE_SIZE.y - pos.y
        }
    }
}
