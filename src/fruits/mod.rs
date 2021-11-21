#![cfg(feature = "fruits")]

mod catch_object;
mod difficulty_object;
mod movement;
mod pp;
mod slider_state;

use catch_object::CatchObject;
use difficulty_object::DifficultyObject;
use movement::Movement;
pub use pp::*;
use slider_state::SliderState;

use crate::{
    curve::{Curve, CurveBuffers},
    parse::{HitObjectKind, Pos2},
    Beatmap, Mods, Strains,
};

const SECTION_LENGTH: f64 = 750.0;
const STAR_SCALING_FACTOR: f64 = 0.153;

const ALLOWED_CATCH_RANGE: f32 = 0.8;
const CATCHER_SIZE: f32 = 106.75;

const LEGACY_LAST_TICK_OFFSET: f64 = 36.0;
const BASE_SCORING_DISTANCE: f64 = 100.0;

/// Difficulty calculation for osu!ctb maps.
///
/// In case of a partial play, e.g. a fail, one can specify the amount of passed objects.
pub fn stars(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> FruitsDifficultyAttributes {
    match calculate_movement(map, mods, passed_objects) {
        Some((mut movement, mut attributes)) => {
            attributes.stars = movement.difficulty_value().sqrt() * STAR_SCALING_FACTOR;
            attributes.max_combo = attributes.n_fruits + attributes.n_droplets;

            attributes
        }
        None => FruitsDifficultyAttributes::default(),
    }
}

/// Essentially the same as the [`stars`] function but instead of
/// evaluating the final strains, it just returns them as is.
///
/// Suitable to plot the difficulty of a map over time.
pub fn strains(map: &Beatmap, mods: impl Mods) -> Strains {
    match calculate_movement(map, mods, None) {
        Some((movement, _)) => Strains {
            section_length: SECTION_LENGTH * mods.speed(),
            strains: movement.strain_peaks,
        },
        None => Strains::default(),
    }
}

fn calculate_movement(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> Option<(Movement, FruitsDifficultyAttributes)> {
    if map.hit_objects.len() < 2 {
        return None;
    }

    let take = passed_objects.unwrap_or(usize::MAX);

    let map_attributes = map.attributes().mods(mods);
    let with_hr = mods.hr();
    let mut ticks = Vec::new(); // using the same buffer for all sliders
    let mut slider_state = SliderState::new(map);
    let mut curve_bufs = CurveBuffers::default();

    let mut attributes = FruitsDifficultyAttributes {
        ar: map_attributes.ar,
        ..Default::default()
    };

    // BUG: Incorrect object order on 2B maps that have fruits within sliders
    let mut hit_objects = map
        .hit_objects
        .iter()
        .scan((None, 0.0), |(last_pos, last_time), h| match &h.kind {
            HitObjectKind::Circle => {
                let mut h = CatchObject::new((h.pos, h.start_time));

                if with_hr {
                    h = h.with_hr(last_pos, last_time);
                }

                attributes.n_fruits += 1;

                Some(Some(FruitOrJuice::Fruit(Some(h))))
            }
            HitObjectKind::Slider {
                pixel_len,
                repeats,
                control_points,
            } => {
                // HR business
                *last_pos = Some(h.pos.x + control_points[control_points.len() - 1].pos.x);
                *last_time = h.start_time;

                // Responsible for timing point values
                slider_state.update(h.start_time);

                let span_count = (*repeats + 1) as f64;

                let mut tick_dist = 100.0 * map.slider_mult / map.tick_rate;

                if map.version >= 8 {
                    tick_dist /=
                        (100.0 / slider_state.slider_velocity).max(10.0).min(1000.0) / 100.0;
                }

                // Build the curve w.r.t. the control points
                let curve = Curve::new(control_points, *pixel_len, &mut curve_bufs);

                let velocity =
                    (BASE_SCORING_DISTANCE * map.slider_mult * slider_state.slider_velocity)
                        / slider_state.beat_len;

                let end_time = h.start_time + span_count * curve.dist() / velocity;
                let duration = end_time - h.start_time;
                let span_duration = duration / span_count;

                // * A very lenient maximum length of a slider for ticks to be generated.
                // * This exists for edge cases such as /b/1573664 where the beatmap has
                // * been edited by the user, and should never be reached in normal usage.
                let max_len = 100_000.0;

                let len = curve.dist().min(max_len);
                tick_dist = tick_dist.clamp(0.0, len);
                let min_dist_from_end = velocity * 10.0;

                let mut curr_dist = tick_dist;
                let time_add = duration * tick_dist / (*pixel_len * span_count);

                let target = *pixel_len - tick_dist / 8.0;

                ticks.reserve((target / tick_dist) as usize);

                // Tick of the first span
                while curr_dist < len - min_dist_from_end {
                    let progress = curr_dist / len;
                    let pos = h.pos + curve.position_at(progress);
                    let time = h.start_time + progress * span_duration;
                    ticks.push((pos, time));
                    curr_dist += tick_dist;
                }

                attributes.n_tiny_droplets += tiny_droplet_count(
                    h.start_time,
                    time_add,
                    duration,
                    span_count as usize,
                    &ticks,
                );

                let mut slider_objects =
                    Vec::with_capacity(span_count as usize * (ticks.len() + 1));
                slider_objects.push((h.pos, h.start_time));

                // Other spans
                if *repeats == 0 {
                    slider_objects.append(&mut ticks); // automatically empties buffer for next slider
                } else {
                    slider_objects.extend(&ticks);

                    for span_idx in 1..=*repeats {
                        let progress = (span_idx % 2 == 1) as u8 as f64;
                        let pos = h.pos + curve.position_at(progress);
                        let time_offset = span_duration * span_idx as f64;

                        // Reverse tick
                        slider_objects.push((pos, h.start_time + time_offset));

                        let new_ticks = ticks.iter().enumerate().map(|(i, (pos, time))| {
                            (*pos, *time + time_offset + time_add * i as f64)
                        });

                        // Actual ticks
                        if span_idx & 1 == 1 {
                            slider_objects.extend(new_ticks.rev());
                        } else {
                            slider_objects.extend(new_ticks);
                        }
                    }

                    ticks.clear();
                }

                // Slider tail
                let progress = (*repeats % 2 == 0) as u8 as f64;
                let pos = h.pos + curve.position_at(progress);
                slider_objects.push((pos, h.start_time + duration));

                let new_fruits = *repeats + 2;
                attributes.n_fruits += new_fruits;
                attributes.n_droplets += slider_objects.len() - new_fruits;

                let iter = slider_objects.into_iter().map(CatchObject::new);

                Some(Some(FruitOrJuice::Juice(iter)))
            }
            HitObjectKind::Spinner { .. } | HitObjectKind::Hold { .. } => Some(None),
        })
        .flatten()
        .flatten()
        .take(take);

    // Hyper dash business
    let half_catcher_width =
        (calculate_catch_width(map_attributes.cs as f32) / 2.0 / ALLOWED_CATCH_RANGE) as f64;
    let mut last_direction = 0;
    let mut last_excess = half_catcher_width;

    // Strain business
    let mut movement = Movement::new(map_attributes.cs as f32);
    let section_len = SECTION_LENGTH * map_attributes.clock_rate;
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev = hit_objects.next().unwrap();
    let mut curr = hit_objects.next().unwrap();

    prev.init_hyper_dash(
        half_catcher_width,
        &curr,
        &mut last_direction,
        &mut last_excess,
    );

    // Handle second object separately to remove later if-branching
    let next = hit_objects.next().unwrap();
    curr.init_hyper_dash(
        half_catcher_width,
        &next,
        &mut last_direction,
        &mut last_excess,
    );

    let h = DifficultyObject::new(
        &curr,
        &prev,
        movement.half_catcher_width,
        map_attributes.clock_rate,
    );

    while h.base.time > current_section_end {
        current_section_end += section_len;
    }

    movement.process(&h);

    prev = curr;
    curr = next;

    // Handle all other objects
    for next in hit_objects {
        curr.init_hyper_dash(
            half_catcher_width,
            &next,
            &mut last_direction,
            &mut last_excess,
        );

        let h = DifficultyObject::new(
            &curr,
            &prev,
            movement.half_catcher_width,
            map_attributes.clock_rate,
        );

        while h.base.time > current_section_end {
            movement.save_current_peak();
            movement.start_new_section_from(current_section_end / map_attributes.clock_rate);
            current_section_end += section_len;
        }

        movement.process(&h);

        prev = curr;
        curr = next;
    }

    // Same as in loop but without init_hyper_dash because `curr` is the last element
    let h = DifficultyObject::new(
        &curr,
        &prev,
        movement.half_catcher_width,
        map_attributes.clock_rate,
    );

    while h.base.time > current_section_end {
        movement.save_current_peak();
        movement.start_new_section_from(current_section_end / map_attributes.clock_rate);

        current_section_end += section_len;
    }

    movement.process(&h);
    movement.save_current_peak();

    Some((movement, attributes))
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

#[inline]
pub(crate) fn calculate_catch_width(cs: f32) -> f32 {
    let scale = 1.0 - 0.7 * (cs - 5.0) / 5.0;

    CATCHER_SIZE * scale.abs() * ALLOWED_CATCH_RANGE
}

enum FruitOrJuice<I> {
    Fruit(Option<CatchObject>),
    Juice(I),
}

impl<I: Iterator<Item = CatchObject>> Iterator for FruitOrJuice<I> {
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
        match self {
            Self::Fruit(Some(_)) => (1, Some(1)),
            Self::Fruit(None) => (0, Some(0)),
            Self::Juice(slider) => slider.size_hint(),
        }
    }
}

/// The result of a difficulty calculation on an osu!ctb map.
#[derive(Clone, Debug, Default)]
pub struct FruitsDifficultyAttributes {
    /// The final star rating
    pub stars: f64,
    /// The maximum combo.
    pub max_combo: usize,
    /// The approach rate.
    pub ar: f64,
    /// The amount of fruits.
    pub n_fruits: usize,
    /// The amount of droplets.
    pub n_droplets: usize,
    /// The amount of tiny droplets.
    pub n_tiny_droplets: usize,
}

/// The result of a performance calculation on an osu!ctb map.
#[derive(Clone, Debug, Default)]
pub struct FruitsPerformanceAttributes {
    /// The difficulty attributes that were used for the performance calculation
    pub difficulty: FruitsDifficultyAttributes,
    /// The final performance points.
    pub pp: f64,
}

impl FruitsPerformanceAttributes {
    /// Return the star value.
    #[inline]
    pub fn stars(&self) -> f64 {
        self.difficulty.stars
    }

    /// Return the performance point value.
    #[inline]
    pub fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    #[inline]
    pub fn max_combo(&self) -> usize {
        self.difficulty.max_combo
    }
}

impl From<FruitsPerformanceAttributes> for FruitsDifficultyAttributes {
    fn from(attributes: FruitsPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}

#[test]
// #[ignore]
fn custom_fruits() {
    use std::time::Instant;

    use crate::{Beatmap, FruitsPP};

    let path = "E:Games/osu!/beatmaps/2919116_.osu";
    let map = Beatmap::from_path(path).unwrap();

    let start = Instant::now();
    let result = FruitsPP::new(&map).mods(256).calculate();

    let iters = 100;
    let accum = start.elapsed();

    // * Tiny benchmark for pp calculation
    // let mut accum = accum;

    // for _ in 0..iters {
    //     let start = Instant::now();
    //     let _result = OsuPP::new(&map).mods(0).calculate();
    //     accum += start.elapsed();
    // }

    println!("{:#?}", result);
    println!("Calculation average: {:?}", accum / iters);
}
