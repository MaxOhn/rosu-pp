use std::vec::Drain;

use rosu_map::section::{
    general::GameMode,
    hit_objects::{CurveBuffers, PathControlPoint, SliderEvent, SliderEventType, SliderEventsIter},
};

use crate::{
    Beatmap,
    catch::attributes::ObjectCountBuilder,
    model::{
        control_point::{DifficultyPoint, TimingPoint},
        hit_object::Slider,
    },
    util::get_precision_adjusted_beat_len,
};

pub struct JuiceStream<'a> {
    pub control_points: &'a [PathControlPoint], // needed for applying hr offset
    pub nested_objects: Drain<'a, NestedJuiceStreamObject>,
}

impl<'a> JuiceStream<'a> {
    pub const BASE_SCORING_DIST: f64 = 100.0;

    pub fn new(
        effective_x: f32,
        start_time: f64,
        slider: &'a Slider,
        map: &Beatmap,
        count: &mut ObjectCountBuilder,
        bufs: &'a mut JuiceStreamBufs,
    ) -> Self {
        let slider_multiplier = map.slider_multiplier;
        let slider_tick_rate = map.slider_tick_rate;

        let beat_len = map
            .timing_point_at(start_time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

        let slider_velocity = map
            .difficulty_point_at(start_time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            });

        let path = slider.borrowed_curve(GameMode::Catch, &mut bufs.curve);

        let velocity = JuiceStream::BASE_SCORING_DIST * slider_multiplier
            / get_precision_adjusted_beat_len(slider_velocity, beat_len);
        let scoring_dist = velocity * beat_len;

        let tick_dist_multiplier = if map.version < 8 {
            slider_velocity.recip()
        } else {
            1.0
        };

        let tick_dist = scoring_dist / slider_tick_rate * tick_dist_multiplier;

        let span_count = slider.span_count() as f64;
        let duration = span_count * path.dist() / velocity;
        let span_duration = duration / span_count;

        let events = SliderEventsIter::new(
            start_time,
            span_duration,
            velocity,
            tick_dist,
            path.dist(),
            slider.span_count() as i32,
            &mut bufs.ticks,
        );

        let mut last_event_time = None;

        for e in events {
            if let Some(last_event_time) = last_event_time {
                let mut tiny_droplets = 0;
                let since_last_tick = f64::from(e.time as i32 - last_event_time as i32);

                if since_last_tick > 80.0 {
                    let mut time_between_tiny = since_last_tick;

                    while time_between_tiny > 100.0 {
                        time_between_tiny /= 2.0;
                    }

                    let mut t = time_between_tiny;

                    while t < since_last_tick {
                        tiny_droplets += 1;

                        let nested = NestedJuiceStreamObject {
                            pos: 0.0,        // not important
                            start_time: 0.0, // not important
                            kind: NestedJuiceStreamObjectKind::TinyDroplet,
                        };

                        bufs.nested_objects.push(nested);

                        t += time_between_tiny;
                    }
                }

                count.record_tiny_droplets(tiny_droplets);
            }

            last_event_time = Some(e.time);

            let kind = match e.kind {
                SliderEventType::Tick => {
                    count.record_droplet();

                    NestedJuiceStreamObjectKind::Droplet
                }
                SliderEventType::Head | SliderEventType::Repeat | SliderEventType::Tail => {
                    count.record_fruit();

                    NestedJuiceStreamObjectKind::Fruit
                }
                SliderEventType::LastTick => continue,
            };

            let nested = NestedJuiceStreamObject {
                pos: effective_x + path.position_at(e.path_progress).x,
                start_time: e.time,
                kind,
            };

            bufs.nested_objects.push(nested);
        }

        Self {
            control_points: slider.control_points.as_ref(),
            nested_objects: bufs.nested_objects.drain(..),
        }
    }
}

#[derive(Debug)]
pub struct NestedJuiceStreamObject {
    pub pos: f32,
    pub start_time: f64,
    pub kind: NestedJuiceStreamObjectKind,
}

#[derive(Debug)]
pub enum NestedJuiceStreamObjectKind {
    Fruit,
    Droplet,
    TinyDroplet,
}

pub struct JuiceStreamBufs {
    pub nested_objects: Vec<NestedJuiceStreamObject>,
    pub curve: CurveBuffers,
    pub ticks: Vec<SliderEvent>,
}
