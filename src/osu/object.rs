use std::borrow::Cow;

use rosu_map::{
    section::{
        general::GameMode,
        hit_objects::{CurveBuffers, SliderEvent, SliderEventType, SliderEventsIter},
    },
    util::Pos,
};

use crate::{
    Beatmap,
    model::{
        control_point::{DifficultyPoint, TimingPoint},
        hit_object::{HitObject, HitObjectKind, HoldNote, Slider, Spinner},
    },
    util::{get_precision_adjusted_beat_len, sort},
};

use super::PLAYFIELD_BASE_SIZE;

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
        map: &Beatmap,
        curve_bufs: &mut CurveBuffers,
        ticks_buf: &mut Vec<SliderEvent>,
    ) -> Self {
        let kind = match h.kind {
            HitObjectKind::Circle => OsuObjectKind::Circle,
            HitObjectKind::Slider(ref slider) => {
                OsuObjectKind::Slider(OsuSlider::new(h, slider, map, curve_bufs, ticks_buf))
            }
            HitObjectKind::Spinner(spinner) => OsuObjectKind::Spinner(spinner),
            HitObjectKind::Hold(HoldNote { duration }) => {
                OsuObjectKind::Spinner(Spinner { duration })
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

            for nested in slider.nested_objects.iter_mut() {
                let mut nested_pos = self.pos; // already reflected at this point
                nested_pos += Pos::new(nested.pos.x, -nested.pos.y);
                nested.pos = nested_pos;
            }
        }
    }

    pub fn reflect_horizontally(&mut self) {
        fn reflect_x(x: &mut f32) {
            *x = PLAYFIELD_BASE_SIZE.x - *x;
        }

        reflect_x(&mut self.pos.x);

        if let OsuObjectKind::Slider(ref mut slider) = self.kind {
            // Requires `stack_offset` so we can't add `h.pos` just yet
            slider.lazy_end_pos.x = -slider.lazy_end_pos.x;

            for nested in slider.nested_objects.iter_mut() {
                let mut nested_pos = self.pos; // already reflected at this point
                nested_pos += Pos::new(-nested.pos.x, nested.pos.y);
                nested.pos = nested_pos;
            }
        }
    }

    pub fn reflect_both_axes(&mut self) {
        fn reflect(pos: &mut Pos) {
            pos.x = PLAYFIELD_BASE_SIZE.x - pos.x;
            pos.y = PLAYFIELD_BASE_SIZE.y - pos.y;
        }

        reflect(&mut self.pos);

        if let OsuObjectKind::Slider(ref mut slider) = self.kind {
            // Requires `stack_offset` so we can't add `h.pos` just yet
            slider.lazy_end_pos.x = -slider.lazy_end_pos.x;
            slider.lazy_end_pos.y = -slider.lazy_end_pos.y;

            for nested in slider.nested_objects.iter_mut() {
                let mut nested_pos = self.pos; // already reflected at this point
                nested_pos += Pos::new(-nested.pos.x, -nested.pos.y);
                nested.pos = nested_pos;
            }
        }
    }

    pub fn finalize_nested(&mut self) {
        if let OsuObjectKind::Slider(ref mut slider) = self.kind {
            for nested in slider.nested_objects.iter_mut() {
                nested.pos += self.pos;
            }
        }
    }

    pub fn end_time(&self) -> f64 {
        match self.kind {
            OsuObjectKind::Circle => self.start_time,
            OsuObjectKind::Slider(ref slider) => slider.end_time,
            OsuObjectKind::Spinner(ref spinner) => self.start_time + spinner.duration,
        }
    }

    pub const fn stacked_pos(&self) -> Pos {
        // Performed manually for const-ness
        // self.pos + self.stack_offset

        Pos::new(
            self.pos.x + self.stack_offset.x,
            self.pos.y + self.stack_offset.y,
        )
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

    pub const fn lazy_travel_time(&self) -> f64 {
        match self.kind {
            OsuObjectKind::Circle | OsuObjectKind::Spinner(_) => 0.0,
            OsuObjectKind::Slider(ref slider) => slider.lazy_travel_time,
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
    pub lazy_travel_time: f64,
    pub nested_objects: Vec<NestedSliderObject>,
}

impl OsuSlider {
    fn new(
        h: &HitObject,
        slider: &Slider,
        map: &Beatmap,
        curve_bufs: &mut CurveBuffers,
        ticks_buf: &mut Vec<SliderEvent>,
    ) -> Self {
        let start_time = h.start_time;
        let slider_multiplier = map.slider_multiplier;
        let slider_tick_rate = map.slider_tick_rate;

        let beat_len = map
            .timing_point_at(start_time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

        let (slider_velocity, generate_ticks) = map.difficulty_point_at(start_time).map_or(
            (
                DifficultyPoint::DEFAULT_SLIDER_VELOCITY,
                DifficultyPoint::DEFAULT_GENERATE_TICKS,
            ),
            |point| (point.slider_velocity, point.generate_ticks),
        );

        let path = slider.curve(GameMode::Osu, curve_bufs);

        let span_count = slider.span_count() as f64;

        let velocity = f64::from(OsuObject::BASE_SCORING_DIST) * slider_multiplier
            / get_precision_adjusted_beat_len(slider_velocity, beat_len);
        let scoring_dist = velocity * beat_len;

        let end_time = start_time + span_count * path.dist() / velocity;

        let duration = end_time - start_time;
        let span_duration = duration / span_count;

        let tick_dist_multiplier = if map.version < 8 {
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
                        pos: path.position_at(e.path_progress),
                        start_time: e.time,
                        kind: NestedSliderObjectKind::Tick,
                    },
                    SliderEventType::Repeat => NestedSliderObject {
                        pos: path.position_at(e.path_progress),
                        start_time: start_time + f64::from(e.span_idx + 1) * span_duration,
                        kind: NestedSliderObjectKind::Repeat,
                    },
                    SliderEventType::Tail => NestedSliderObject {
                        pos: end_path_pos, // no `h.pos` yet to keep order of float operations
                        start_time: e.time,
                        kind: NestedSliderObjectKind::Tail,
                    },
                    SliderEventType::Head | SliderEventType::LastTick => return None,
                };

                Some(obj)
            })
            .collect();

        sort::csharp(&mut nested_objects, |a, b| {
            a.start_time.total_cmp(&b.start_time)
        });

        let mut nested = Cow::Borrowed(nested_objects.as_slice());
        let lazy_travel_time = OsuSlider::lazy_travel_time(start_time, duration, &mut nested);

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
            lazy_travel_time,
            nested_objects,
        }
    }

    pub fn lazy_travel_time(
        start_time: f64,
        duration: f64,
        nested_objects: &mut Cow<'_, [NestedSliderObject]>,
    ) -> f64 {
        const TAIL_LENIENCY: f64 = -36.0;

        let mut tracking_end_time =
            (start_time + duration + TAIL_LENIENCY).max(start_time + duration / 2.0);

        let last_real_tick = nested_objects
            .iter()
            .enumerate()
            .rfind(|(_, nested)| nested.is_tick());

        if let Some((idx, last_real_tick)) =
            last_real_tick.filter(|(_, tick)| tick.start_time > tracking_end_time)
        {
            tracking_end_time = last_real_tick.start_time;

            // * When the last tick falls after the tracking end time, we need to re-sort the nested objects
            // * based on time. This creates a somewhat weird ordering which is counter to how a user would
            // * understand the slider, but allows a zero-diff with known diffcalc output.
            // *
            // * To reiterate, this is definitely not correct from a difficulty calculation perspective
            // * and should be revisited at a later date (likely by replacing this whole code with the commented
            // * version above).
            nested_objects.to_mut()[idx..].rotate_left(1);
        }

        tracking_end_time - start_time
    }

    pub fn repeat_count(&self) -> usize {
        self.nested_objects
            .iter()
            .filter(|nested| matches!(nested.kind, NestedSliderObjectKind::Repeat))
            .count()
    }

    /// Counts both ticks and repeats
    pub fn large_tick_count(&self) -> usize {
        self.nested_objects
            .iter()
            .filter(|nested| {
                matches!(
                    nested.kind,
                    NestedSliderObjectKind::Tick | NestedSliderObjectKind::Repeat
                )
            })
            .count()
    }

    pub fn tail(&self) -> Option<&NestedSliderObject> {
        self.nested_objects
            .iter()
            // The tail is not necessarily the last nested object, e.g. on very
            // short and fast buzz sliders (/b/1001757)
            .rfind(|nested| matches!(nested.kind, NestedSliderObjectKind::Tail))
    }
}

#[derive(Clone, Debug)]
pub struct NestedSliderObject {
    pub pos: Pos,
    pub start_time: f64,
    pub kind: NestedSliderObjectKind,
}

impl NestedSliderObject {
    pub const fn is_repeat(&self) -> bool {
        matches!(self.kind, NestedSliderObjectKind::Repeat)
    }

    pub const fn is_tick(&self) -> bool {
        matches!(self.kind, NestedSliderObjectKind::Tick)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum NestedSliderObjectKind {
    Repeat,
    Tail,
    Tick,
}
