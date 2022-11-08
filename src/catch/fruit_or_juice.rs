use std::{iter::Map, vec::IntoIter};

use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObject, HitObjectKind, Pos2},
    Beatmap,
};

use super::{catch_object::CatchObject, CatchDifficultyAttributes};

const LEGACY_LAST_TICK_OFFSET: f64 = 36.0;
const BASE_SCORING_DISTANCE: f64 = 100.0;

#[derive(Clone, Debug)]
pub(crate) struct FruitParams<'a> {
    pub(crate) attributes: CatchDifficultyAttributes,
    pub(crate) curve_bufs: CurveBuffers,
    pub(crate) last_pos: Option<f32>,
    pub(crate) last_time: f64,
    pub(crate) map: &'a Beatmap,
    pub(crate) ticks: Vec<(Pos2, f64)>,
    pub(crate) with_hr: bool,
}

type JuiceStream = Map<IntoIter<(Pos2, f64)>, fn((Pos2, f64)) -> CatchObject>;

#[derive(Clone, Debug)]
pub(crate) enum FruitOrJuice {
    Fruit(Option<CatchObject>),
    Juice(JuiceStream),
}

impl FruitOrJuice {
    pub(crate) fn new(h: &HitObject, params: &mut FruitParams<'_>) -> Option<Self> {
        match &h.kind {
            HitObjectKind::Circle => {
                let mut h = CatchObject::new((h.pos, h.start_time));

                if params.with_hr {
                    h = h.with_hr(params);
                }

                params.attributes.n_fruits += 1;

                Some(FruitOrJuice::Fruit(Some(h)))
            }
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
                ..
            } => {
                // HR business
                params.last_pos = Some(h.pos.x + control_points[control_points.len() - 1].pos.x);
                params.last_time = h.start_time;

                let timing_point = params.map.timing_point_at(h.start_time);

                let difficulty_point = params
                    .map
                    .difficulty_point_at(h.start_time)
                    .unwrap_or_default();

                let vel_factor =
                    BASE_SCORING_DISTANCE * params.map.slider_mult / timing_point.beat_len;
                let tick_dist_factor =
                    BASE_SCORING_DISTANCE * params.map.slider_mult / params.map.tick_rate;

                let vel = vel_factor * difficulty_point.slider_vel;

                let mut tick_dist = tick_dist_factor * difficulty_point.slider_vel;

                let span_count = (*repeats + 1) as f64;

                // Build the curve w.r.t. the control points
                let curve = Curve::new(control_points, *pixel_len, &mut params.curve_bufs);

                let total_duration = span_count * curve.dist() / vel;
                let span_duration = total_duration / span_count;

                // * A very lenient maximum length of a slider for ticks to be generated.
                // * This exists for edge cases such as /b/1573664 where the beatmap has
                // * been edited by the user, and should never be reached in normal usage.
                let max_len = 100_000.0;

                let len = curve.dist().min(max_len);
                tick_dist = tick_dist.clamp(0.0, len);
                let min_dist_from_end = vel * 10.0;

                let mut curr_dist = tick_dist;
                let pixel_len = pixel_len.unwrap_or(0.0);

                let target = pixel_len - tick_dist / 8.0;

                let mut slider_objects = vec![(h.pos, h.start_time)];

                if tick_dist > 0.0 {
                    params.ticks.reserve((target / tick_dist) as usize);

                    // Tick of the first span
                    while curr_dist < len - min_dist_from_end {
                        let progress = curr_dist / len;
                        let pos = h.pos + curve.position_at(progress);
                        let time = h.start_time + progress * span_duration;
                        params.ticks.push((pos, time));
                        curr_dist += tick_dist;
                    }

                    if pixel_len > 0.0 {
                        let time_add = total_duration * tick_dist / (pixel_len * span_count);

                        params.attributes.n_tiny_droplets += tiny_droplet_count(
                            h.start_time,
                            time_add,
                            total_duration,
                            span_count as usize,
                            &params.ticks,
                        );
                    }

                    slider_objects.reserve(span_count as usize * (params.ticks.len()));

                    // Other spans
                    if *repeats == 0 {
                        slider_objects.append(&mut params.ticks); // automatically empties buffer for next slider
                    } else {
                        slider_objects.extend(&params.ticks);

                        for span_idx in 1..=*repeats {
                            let progress = (span_idx % 2 == 1) as u8 as f64;
                            let pos = h.pos + curve.position_at(progress);
                            let time_offset = span_duration * span_idx as f64;

                            // Reverse tick
                            slider_objects.push((pos, h.start_time + time_offset));

                            // Actual ticks
                            if span_idx & 1 == 1 {
                                let tick_iter = params
                                    .ticks
                                    .iter()
                                    .rev()
                                    .zip(params.ticks.iter())
                                    .map(|((pos, _), (_, time))| (*pos, *time + time_offset));

                                slider_objects.extend(tick_iter);
                            } else {
                                let tick_iter = params
                                    .ticks
                                    .iter()
                                    .map(|(pos, time)| (*pos, *time + time_offset));

                                slider_objects.extend(tick_iter);
                            }
                        }

                        params.ticks.clear();
                    }
                }

                // Slider tail
                let progress = (*repeats % 2 == 0) as u8 as f64;
                let pos = h.pos + curve.position_at(progress);
                slider_objects.push((pos, h.start_time + total_duration));

                let new_fruits = 2 + (tick_dist > 0.0) as usize * *repeats;
                params.attributes.n_fruits += new_fruits;
                params.attributes.n_droplets += slider_objects.len() - new_fruits;

                let iter = slider_objects
                    .into_iter()
                    .map(CatchObject::new as fn(_) -> _);

                Some(FruitOrJuice::Juice(iter))
            }
            HitObjectKind::Spinner { .. } | HitObjectKind::Hold { .. } => None,
        }
    }
}

impl Iterator for FruitOrJuice {
    type Item = CatchObject;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Fruit(fruit) => fruit.take(),
            Self::Juice(slider) => slider.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for FruitOrJuice {
    #[inline]
    fn len(&self) -> usize {
        match self {
            FruitOrJuice::Fruit(Some(_)) => 1,
            FruitOrJuice::Fruit(None) => 0,
            FruitOrJuice::Juice(slider) => slider.len(),
        }
    }
}

// BUG: Sometimes there are off-by-one errors,
// presumably caused by floating point inaccuracies
fn tiny_droplet_count(
    start_time: f64,
    time_between_ticks: f64,
    duration: f64,
    span_count: usize,
    ticks: &[(Pos2, f64)],
) -> usize {
    // tiny droplets preceeding a _tick_
    let per_tick = if !ticks.is_empty() && time_between_ticks > 80.0 {
        let time_between_tiny = shrink_down(time_between_ticks);

        // add a little for floating point inaccuracies
        let start = time_between_tiny + 0.001;

        count_iterations(start, time_between_tiny, time_between_ticks)
    } else {
        0
    };

    // tiny droplets preceeding a _reverse_
    let last = ticks.last().map_or(start_time, |(_, last)| *last);
    let repeat_time = start_time + duration / span_count as f64;
    let since_last_tick = repeat_time - last;

    let span_last_section = if since_last_tick > 80.0 {
        let time_between_tiny = shrink_down(since_last_tick);

        count_iterations(time_between_tiny, time_between_tiny, since_last_tick)
    } else {
        0
    };

    // tiny droplets preceeding the slider tail
    // necessary to handle distinctly because of the legacy last tick
    let last = ticks.last().map_or(start_time, |(_, last)| *last);
    let end_time = start_time + duration / span_count as f64 - LEGACY_LAST_TICK_OFFSET;
    let since_last_tick = end_time - last;

    let last_section = if since_last_tick > 80.0 {
        let time_between_tiny = shrink_down(since_last_tick);

        count_iterations(time_between_tiny, time_between_tiny, since_last_tick)
    } else {
        0
    };

    // Combine tiny droplets counts
    per_tick * ticks.len() * span_count
        + span_last_section * (span_count.saturating_sub(1))
        + last_section
}

#[inline]
fn shrink_down(mut val: f64) -> f64 {
    while val > 100.0 {
        val /= 2.0;
    }

    val
}

#[inline]
fn count_iterations(mut start: f64, step: f64, end: f64) -> usize {
    let mut count = 0;

    while start < end {
        count += 1;
        start += step;
    }

    count
}
