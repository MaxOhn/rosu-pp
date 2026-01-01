use std::cmp;

use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::difficulty::{logistic, smoothstep_bell_curve},
};

pub struct RhythmEvaluator;

impl RhythmEvaluator {
    const HISTORY_TIME_MAX: u32 = 5 * 1000; // 5 seconds
    const HISTORY_OBJECTS_MAX: usize = 32;
    const RHYTHM_OVERALL_MULTIPLIER: f64 = 1.0;
    const RHYTHM_RATIO_MULTIPLIER: f64 = 15.0;

    #[expect(clippy::too_many_lines, reason = "staying in-sync with lazer")]
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        hit_window: f64,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let mut rhythm_complexity_sum = 0.0;

        let delta_difference_eps = hit_window * 0.3;

        let mut island = RhythmIsland::new(delta_difference_eps);
        let mut prev_island = RhythmIsland::new(delta_difference_eps);

        // * we can't use dictionary here because we need to compare island with a tolerance
        // * which is impossible to pass into the hash comparer
        let mut island_counts = Vec::<IslandCount>::new();

        // * store the ratio of the current start of an island to buff for tighter rhythms
        let mut start_ratio = 0.0;

        let mut first_delta_switch = false;

        let historical_note_count = cmp::min(curr.idx, Self::HISTORY_OBJECTS_MAX);

        let mut rhythm_start = 0;

        while curr
            .previous(rhythm_start, diff_objects)
            .filter(|prev| {
                rhythm_start + 2 < historical_note_count
                    && curr.start_time - prev.start_time < f64::from(Self::HISTORY_TIME_MAX)
            })
            .is_some()
        {
            rhythm_start += 1;
        }

        if let Some((mut prev_obj, mut last_obj)) = curr
            .previous(rhythm_start, diff_objects)
            .zip(curr.previous(rhythm_start + 1, diff_objects))
        {
            // * we go from the furthest object back to the current one
            for i in (1..=rhythm_start).rev() {
                let Some(curr_obj) = curr.previous(i - 1, diff_objects) else {
                    break;
                };

                // * scales note 0 to 1 from history to now
                let time_decay = (f64::from(Self::HISTORY_TIME_MAX)
                    - (curr.start_time - curr_obj.start_time))
                    / f64::from(Self::HISTORY_TIME_MAX);
                let note_decay = (historical_note_count - i) as f64 / historical_note_count as f64;

                // * either we're limited by time or limited by object count.
                let curr_historical_decay = note_decay.min(time_decay);

                // * Use custom cap value to ensure that at this point delta time is actually zero
                let curr_delta = curr_obj.delta_time.max(1e-7);
                let prev_delta = prev_obj.delta_time.max(1e-7);
                let last_delta = last_obj.delta_time.max(1e-7);

                // * calculate how much current delta difference deserves a rhythm bonus
                // * this function is meant to reduce rhythm bonus for deltas that are multiples of each other (i.e 100 and 200)
                let delta_difference = prev_delta.max(curr_delta) / prev_delta.min(curr_delta);

                // * Take only the fractional part of the value since we're only interested in punishing multiples
                let delta_difference_fraction = delta_difference - delta_difference.trunc();

                let curr_ratio = 1.0
                    + Self::RHYTHM_RATIO_MULTIPLIER
                        * smoothstep_bell_curve(delta_difference_fraction, 0.5, 0.5).min(0.5);

                // * reduce ratio bonus if delta difference is too big
                let difference_multiplier = (2.0 - delta_difference / 8.0).clamp(0.0, 1.0);

                let window_penalty = (((prev_delta - curr_delta).abs() - delta_difference_eps)
                    .max(0.0)
                    / delta_difference_eps)
                    .min(1.0);

                let mut effective_ratio = window_penalty * curr_ratio * difference_multiplier;

                if first_delta_switch {
                    if (prev_delta - curr_delta).abs() < delta_difference_eps {
                        // * island is still progressing
                        island.add_delta(curr_delta as i32);
                    } else {
                        // * bpm change is into slider, this is easy acc window
                        if curr_obj.base.is_slider() {
                            effective_ratio *= 0.125;
                        }

                        // * bpm change was from a slider, this is easier typically than circle -> circle
                        // * unintentional side effect is that bursts with kicksliders at the ends might have lower difficulty than bursts without sliders
                        if prev_obj.base.is_slider() {
                            effective_ratio *= 0.3;
                        }

                        // * repeated island polarity (2 -> 4, 3 -> 5)
                        if island.is_similar_polarity(&prev_island) {
                            effective_ratio *= 0.5;
                        }

                        // * previous increase happened a note ago, 1/1->1/2-1/4, dont want to buff this.
                        if last_delta > prev_delta + delta_difference_eps
                            && prev_delta > curr_delta + delta_difference_eps
                        {
                            effective_ratio *= 0.125;
                        }

                        // * repeated island size (ex: triplet -> triplet)
                        // * TODO: remove this nerf since its staying here only for balancing purposes because of the flawed ratio calculation
                        if prev_island.delta_count == island.delta_count {
                            effective_ratio *= 0.5;
                        }

                        if let Some(island_count) = island_counts
                            .iter_mut()
                            .find(|entry| entry.island == island)
                            .filter(|entry| !entry.island.is_default())
                        {
                            // * only add island to island counts if they're going one after another
                            if prev_island == island {
                                island_count.count += 1;
                            }

                            // * repeated island (ex: triplet -> triplet)
                            let power = logistic(f64::from(island.delta), 58.33, 0.24, Some(2.75));
                            effective_ratio *= (3.0 / island_count.count as f64)
                                .min((island_count.count as f64).recip().powf(power));
                        } else {
                            island_counts.push(IslandCount { island, count: 1 });
                        }

                        // * scale down the difficulty if the object is doubletappable
                        let doubletapness = prev_obj.get_doubletapness(Some(curr_obj), hit_window);
                        effective_ratio *= 1.0 - doubletapness * 0.75;

                        rhythm_complexity_sum +=
                            (effective_ratio * start_ratio).sqrt() * curr_historical_decay;

                        start_ratio = effective_ratio;

                        prev_island = island;

                        // * we're slowing down, stop counting
                        if prev_delta + delta_difference_eps < curr_delta {
                            // * if we're speeding up, this stays true and we keep counting island size.
                            first_delta_switch = false;
                        }

                        island =
                            RhythmIsland::new_with_delta(curr_delta as i32, delta_difference_eps);
                    }
                } else if prev_delta > curr_delta + delta_difference_eps {
                    // * we're speeding up.
                    // * Begin counting island until we change speed again.
                    first_delta_switch = true;

                    // * bpm change is into slider, this is easy acc window
                    if curr_obj.base.is_slider() {
                        effective_ratio *= 0.6;
                    }

                    // * bpm change was from a slider, this is easier typically than circle -> circle
                    // * unintentional side effect is that bursts with kicksliders at the ends might have lower difficulty than bursts without sliders
                    if prev_obj.base.is_slider() {
                        effective_ratio *= 0.6;
                    }

                    start_ratio = effective_ratio;

                    island = RhythmIsland::new_with_delta(curr_delta as i32, delta_difference_eps);
                }

                last_obj = prev_obj;
                prev_obj = curr_obj;
            }
        }

        // * produces multiplier that can be applied to strain. range [1, infinity) (not really though)
        let mut rhythm_difficulty =
            (4.0 + rhythm_complexity_sum * Self::RHYTHM_OVERALL_MULTIPLIER).sqrt() / 2.0;
        rhythm_difficulty *= 1.0 - curr.get_doubletapness(curr.next(0, diff_objects), hit_window);

        rhythm_difficulty
    }
}

#[derive(Copy, Clone)]
struct RhythmIsland {
    delta_difference_eps: f64,
    delta: i32,
    delta_count: i32,
}

const MIN_DELTA_TIME: i32 = 25;

// Compile-time check in case `OsuDifficultyObject::MIN_DELTA_TIME` changes
// but we forget to update this value.
const _: [(); 0 - !{ MIN_DELTA_TIME - OsuDifficultyObject::MIN_DELTA_TIME as i32 == 0 } as usize] =
    [];

impl RhythmIsland {
    const fn new(delta_difference_eps: f64) -> Self {
        Self {
            delta_difference_eps,
            delta: 0,
            delta_count: 0,
        }
    }

    fn new_with_delta(delta: i32, delta_difference_eps: f64) -> Self {
        Self {
            delta_difference_eps,
            delta: delta.max(MIN_DELTA_TIME),
            delta_count: 1,
        }
    }

    fn add_delta(&mut self, delta: i32) {
        if self.delta == i32::MAX {
            self.delta = delta.max(MIN_DELTA_TIME);
        }

        self.delta_count += 1;
    }

    const fn is_similar_polarity(&self, other: &Self) -> bool {
        // * TODO: consider islands to be of similar polarity only if they're having the same average delta (we don't want to consider 3 singletaps similar to a triple)
        // *       naively adding delta check here breaks _a lot_ of maps because of the flawed ratio calculation
        self.delta_count % 2 == other.delta_count % 2
    }

    fn is_default(&self) -> bool {
        self.delta_difference_eps.abs() < f64::EPSILON
            && self.delta == i32::MAX
            && self.delta_count == 0
    }
}

impl PartialEq for RhythmIsland {
    fn eq(&self, other: &Self) -> bool {
        f64::from((self.delta - other.delta).abs()) < self.delta_difference_eps
            && self.delta_count == other.delta_count
    }
}

struct IslandCount {
    island: RhythmIsland,
    count: usize,
}
