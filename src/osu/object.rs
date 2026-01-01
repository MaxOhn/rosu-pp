use rosu_map::{
    section::{
        general::GameMode,
        hit_objects::{Curve, CurveBuffers, SliderEvent, SliderEventType, SliderEventsIter},
    },
    util::Pos,
};

use crate::{
    Beatmap,
    model::{
        control_point::{DifficultyPoint, TimingPoint},
        hit_object::{HitObject, HitObjectKind, HoldNote, Slider, Spinner},
        mods::Reflection,
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
        reflection: Reflection,
        curve_bufs: &mut CurveBuffers,
        ticks_buf: &mut Vec<SliderEvent>,
    ) -> Self {
        let kind = match h.kind {
            HitObjectKind::Circle => OsuObjectKind::Circle,
            HitObjectKind::Slider(ref slider) => OsuObjectKind::Slider(OsuSlider::new(
                h, slider, map, reflection, curve_bufs, ticks_buf,
            )),
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
        self.finalize_nested();
    }

    pub fn reflect_horizontally(&mut self) {
        fn reflect_x(x: &mut f32) {
            *x = PLAYFIELD_BASE_SIZE.x - *x;
        }

        reflect_x(&mut self.pos.x);
        self.finalize_nested();
    }

    pub fn reflect_both_axes(&mut self) {
        fn reflect(pos: &mut Pos) {
            pos.x = PLAYFIELD_BASE_SIZE.x - pos.x;
            pos.y = PLAYFIELD_BASE_SIZE.y - pos.y;
        }

        reflect(&mut self.pos);
        self.finalize_nested();
    }

    pub fn finalize_nested(&mut self) {
        if let OsuObjectKind::Slider(ref mut slider) = self.kind {
            for nested in slider.nested_objects.iter_mut() {
                nested.pos = self.pos + nested.pos;
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
    pub span_count: f64,
    pub path: Curve,
    pub nested_objects: Vec<NestedSliderObject>,
}

impl OsuSlider {
    fn new(
        h: &HitObject,
        slider: &Slider,
        map: &Beatmap,
        reflection: Reflection,
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

        let path = slider.curve(GameMode::Osu, reflection, curve_bufs);

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

        Self {
            end_time,
            span_count,
            path,
            nested_objects,
        }
    }

    pub fn repeat_count(&self) -> usize {
        self.nested_objects
            .iter()
            .filter(|nested| matches!(nested.kind, NestedSliderObjectKind::Repeat))
            .count()
    }

    pub fn tick_count(&self) -> usize {
        self.nested_objects
            .iter()
            .filter(|nested| matches!(nested.kind, NestedSliderObjectKind::Tick))
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
