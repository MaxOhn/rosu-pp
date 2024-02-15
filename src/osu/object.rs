use rosu_map::{
    section::hit_objects::{CurveBuffers, SliderEvent, SliderEventType, SliderEventsIter},
    util::Pos,
};

use crate::{
    model::{
        control_point::{DifficultyPoint, TimingPoint},
        hit_object::{HitObject, HitObjectKind, HoldNote, Slider, Spinner},
    },
    util::sort,
};

use super::{convert::OsuBeatmap, PLAYFIELD_BASE_SIZE};

pub struct OsuObject {
    pub pos: Pos,
    pub start_time: f64,
    pub stack_height: i32,
    pub stack_offset: Pos,
    pub kind: OsuObjectKind,
}

impl OsuObject {
    pub const OBJECT_RADIUS: f32 = 64.0;
    pub const PREEMPT_MIN: f64 = 450.0;

    const BASE_SCORING_DIST: f32 = 100.0;

    pub fn new(
        h: &HitObject,
        converted: &OsuBeatmap<'_>,
        curve_bufs: &mut CurveBuffers,
        ticks_buf: &mut Vec<SliderEvent>,
    ) -> Self {
        let kind = match h.kind {
            HitObjectKind::Circle => OsuObjectKind::Circle,
            HitObjectKind::Slider(ref slider) => {
                OsuObjectKind::Slider(OsuSlider::new(h, slider, converted, curve_bufs, ticks_buf))
            }
            HitObjectKind::Spinner(spinner) => OsuObjectKind::Spinner(spinner),
            HitObjectKind::Hold(HoldNote { end_time }) => {
                OsuObjectKind::Spinner(Spinner { end_time })
            }
        };

        Self {
            pos: h.pos,
            start_time: h.start_time,
            stack_height: 0,
            stack_offset: Pos::default(),
            kind,
        }
    }

    pub fn reflect_vertically(&mut self) {
        fn reflect_y(y: &mut f32) {
            *y = PLAYFIELD_BASE_SIZE.y - *y;
        }

        reflect_y(&mut self.pos.y);

        if let OsuObjectKind::Slider(ref mut slider) = self.kind {
            // Requires `stack_offset` so we can't add `h.pos` just yet
            slider.lazy_end_pos.y = -slider.lazy_end_pos.y;

            let mut nested_iter = slider.nested_objects.iter_mut();

            // Since the tail is handled differently but it's not necessarily
            // the last object, we first search for it, and then handle the
            // other nested objects
            for nested in nested_iter.by_ref().rev() {
                if let NestedSliderObjectKind::Tail = nested.kind {
                    let mut tail_pos = self.pos; // already reflected at this point
                    tail_pos += Pos::new(nested.pos.x, -nested.pos.y);
                    nested.pos = tail_pos;

                    break;
                }

                reflect_y(&mut nested.pos.y);
            }

            for nested in nested_iter {
                reflect_y(&mut nested.pos.y);
            }
        }
    }

    pub fn finalize_tail(&mut self) {
        if let OsuObjectKind::Slider(ref mut slider) = self.kind {
            if let Some(tail) = slider.tail_mut() {
                tail.pos += self.pos;
            }
        }
    }

    pub const fn end_time(&self) -> f64 {
        match self.kind {
            OsuObjectKind::Circle => self.start_time,
            OsuObjectKind::Slider(ref slider) => slider.end_time,
            OsuObjectKind::Spinner(ref spinner) => spinner.end_time,
        }
    }

    pub fn stacked_pos(&self) -> Pos {
        self.pos + self.stack_offset
    }

    pub fn end_pos(&self) -> Pos {
        match self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner(_) => self.pos,
            OsuObjectKind::Slider(ref slider) => {
                slider.tail().map_or(Pos::default(), |nested| nested.pos)
            }
        }
    }

    pub fn stacked_end_pos(&self) -> Pos {
        self.end_pos() + self.stack_offset
    }

    pub fn lazy_travel_time(&self) -> f64 {
        match self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner(_) => 0.0,
            OsuObjectKind::Slider(ref slider) => slider
                .nested_objects
                // Here we really want the last nested object which is not
                // necessarily the tail
                .last()
                .map_or(0.0, |nested| nested.start_time - self.start_time),
        }
    }

    pub const fn is_circle(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Circle)
    }

    pub const fn is_slider(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Slider { .. })
    }

    pub const fn is_spinner(&self) -> bool {
        matches!(self.kind, OsuObjectKind::Spinner(_))
    }
}

pub enum OsuObjectKind {
    Circle,
    Slider(OsuSlider),
    Spinner(Spinner),
}

pub struct OsuSlider {
    pub end_time: f64,
    pub lazy_end_pos: Pos,
    pub lazy_travel_dist: f32,
    pub nested_objects: Vec<NestedSliderObject>,
}

impl OsuSlider {
    fn new(
        h: &HitObject,
        slider: &Slider,
        converted: &OsuBeatmap<'_>,
        curve_bufs: &mut CurveBuffers,
        ticks_buf: &mut Vec<SliderEvent>,
    ) -> Self {
        let start_time = h.start_time;
        let slider_multiplier = converted.map.slider_multiplier;
        let slider_tick_rate = converted.map.slider_tick_rate;

        let beat_len = converted
            .map
            .timing_point_at(start_time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

        let (slider_velocity, generate_ticks) =
            converted.map.difficulty_point_at(start_time).map_or(
                (
                    DifficultyPoint::DEFAULT_SLIDER_VELOCITY,
                    DifficultyPoint::DEFAULT_GENERATE_TICKS,
                ),
                |point| (point.slider_velocity, point.generate_ticks),
            );

        let path = slider.curve(curve_bufs);

        let span_count = slider.span_count() as f64;

        let scoring_dist =
            f64::from(OsuObject::BASE_SCORING_DIST) * slider_multiplier * slider_velocity;
        let velocity = scoring_dist / beat_len;

        let end_time = start_time + span_count * path.dist() / velocity;

        let duration = end_time - start_time;
        let span_duration = duration / span_count;

        let tick_dist_multiplier = if converted.map.version < 8 {
            slider_velocity.recip()
        } else {
            1.0
        };

        let tick_dist = if generate_ticks {
            scoring_dist / slider_tick_rate * tick_dist_multiplier
        } else {
            f64::INFINITY
        };

        let events = SliderEventsIter::new(
            start_time,
            span_duration,
            velocity,
            tick_dist,
            path.dist(),
            slider.span_count() as i32,
            ticks_buf,
        );

        let span_at = |progress: f64| (progress * span_count) as i32;

        let obj_progress_at = |progress: f64| {
            let p = progress * span_count % 1.0;

            if span_at(progress) % 2 == 1 {
                1.0 - p
            } else {
                p
            }
        };

        let end_path_pos = path.position_at(obj_progress_at(1.0));

        let mut nested_objects: Vec<_> = events
            .filter_map(|e| {
                let obj = match e.kind {
                    SliderEventType::Tick => NestedSliderObject {
                        pos: h.pos + path.position_at(e.path_progress),
                        start_time: e.time,
                        kind: NestedSliderObjectKind::Tick,
                    },
                    SliderEventType::Repeat => NestedSliderObject {
                        pos: h.pos + path.position_at(e.path_progress),
                        start_time: start_time + f64::from(e.span_idx + 1) * span_duration,
                        kind: NestedSliderObjectKind::Repeat,
                    },
                    SliderEventType::LastTick => NestedSliderObject {
                        pos: end_path_pos, // no `h.pos` yet to keep order of float operations
                        start_time: e.time,
                        kind: NestedSliderObjectKind::Tail,
                    },
                    SliderEventType::Head | SliderEventType::Tail => return None,
                };

                Some(obj)
            })
            .collect();

        sort::csharp(&mut nested_objects, |a, b| {
            a.start_time.total_cmp(&b.start_time)
        });

        let lazy_travel_time = nested_objects
            .last()
            .map_or(0.0, |nested| nested.start_time - h.start_time);

        let mut end_time_min = lazy_travel_time / span_duration;

        if end_time_min % 2.0 >= 1.0 {
            end_time_min = 1.0 - end_time_min % 1.0;
        } else {
            end_time_min %= 1.0;
        }

        let lazy_end_pos = path.position_at(end_time_min);

        Self {
            end_time,
            lazy_end_pos,
            lazy_travel_dist: 0.0,
            nested_objects,
        }
    }

    pub fn repeat_count(&self) -> usize {
        self.nested_objects
            .iter()
            .filter(|nested| matches!(nested.kind, NestedSliderObjectKind::Repeat))
            .count()
    }

    pub fn tail(&self) -> Option<&NestedSliderObject> {
        self.nested_objects
            .iter()
            // The tail is not necessarily the last nested object, e.g. on very
            // short and fast buzz sliders (/b/1001757)
            .rfind(|nested| matches!(nested.kind, NestedSliderObjectKind::Tail))
    }

    fn tail_mut(&mut self) -> Option<&mut NestedSliderObject> {
        self.nested_objects
            .iter_mut()
            // The tail is not necessarily the last nested object, e.g. on very
            // short and fast buzz sliders (/b/1001757)
            .rfind(|nested| matches!(nested.kind, NestedSliderObjectKind::Tail))
    }
}

pub struct NestedSliderObject {
    pub pos: Pos,
    pub start_time: f64,
    pub kind: NestedSliderObjectKind,
}

impl NestedSliderObject {
    pub const fn is_repeat(&self) -> bool {
        matches!(self.kind, NestedSliderObjectKind::Repeat)
    }
}

pub enum NestedSliderObjectKind {
    Repeat,
    Tail,
    Tick,
}
