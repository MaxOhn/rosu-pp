use std::{
    cmp,
    f64::consts::{E, PI},
};

use crate::{
    any::difficulty::{
        object::IDifficultyObject,
        skills::{strain_decay, ISkill, Skill},
    },
    osu::difficulty::object::OsuDifficultyObject,
    util::strains_vec::StrainsVec,
    GameMods,
};

use super::strain::{DifficultyValue, OsuStrainSkill, UsedOsuStrainSkills};

const SKILL_MULTIPLIER: f64 = 1.430;
const STRAIN_DECAY_BASE: f64 = 0.3;

const REDUCED_SECTION_COUNT: usize = 5;

#[derive(Clone)]
pub struct Speed {
    curr_strain: f64,
    curr_rhythm: f64,
    hit_window: f64,
    has_autopilot_mod: bool,
    inner: OsuStrainSkill,
}

impl Speed {
    pub fn new(hit_window: f64, mods: &GameMods) -> Self {
        Self {
            curr_strain: 0.0,
            curr_rhythm: 0.0,
            hit_window,
            has_autopilot_mod: mods.ap(),
            inner: OsuStrainSkill::default(),
        }
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks().strains()
    }

    pub fn difficulty_value(self) -> UsedOsuStrainSkills<DifficultyValue> {
        Self::static_difficulty_value(self.inner)
    }

    /// Use [`difficulty_value`] instead whenever possible because
    /// [`as_difficulty_value`] clones internally.
    pub fn as_difficulty_value(&self) -> UsedOsuStrainSkills<DifficultyValue> {
        Self::static_difficulty_value(self.inner.clone())
    }

    fn static_difficulty_value(skill: OsuStrainSkill) -> UsedOsuStrainSkills<DifficultyValue> {
        skill.difficulty_value(
            REDUCED_SECTION_COUNT,
            OsuStrainSkill::REDUCED_STRAIN_BASELINE,
            OsuStrainSkill::DECAY_WEIGHT,
        )
    }

    pub fn relevant_note_count(&self) -> f64 {
        self.inner
            .object_strains
            .iter()
            .copied()
            .max_by(f64::total_cmp)
            .filter(|&n| n > 0.0)
            .map_or(0.0, |max_strain| {
                self.inner.object_strains.iter().fold(0.0, |sum, strain| {
                    sum + (1.0 + (-(strain / max_strain * 12.0 - 6.0)).exp()).recip()
                })
            })
    }
}

impl ISkill for Speed {
    type DifficultyObjects<'a> = [OsuDifficultyObject<'a>];
}

impl<'a> Skill<'a, Speed> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        let prev_start_time = curr
            .previous(0, self.diff_objects)
            .map_or(0.0, |prev| prev.start_time);

        (self.inner.curr_strain * self.inner.curr_rhythm)
            * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    fn curr_section_peak(&self) -> f64 {
        self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_end(&self) -> f64 {
        self.inner.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &'a OsuDifficultyObject<'a>) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / OsuStrainSkill::SECTION_LEN).ceil()
                * OsuStrainSkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += OsuStrainSkill::SECTION_LEN;
        }

        let strain_value_at = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }

    fn strain_value_at(&mut self, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        self.inner.curr_strain *= strain_decay(curr.strain_time, STRAIN_DECAY_BASE);
        self.inner.curr_strain += SpeedEvaluator::evaluate_diff_of(
            curr,
            self.diff_objects,
            self.inner.hit_window,
            self.inner.has_autopilot_mod,
        ) * SKILL_MULTIPLIER;
        self.inner.curr_rhythm =
            RhythmEvaluator::evaluate_diff_of(curr, self.diff_objects, self.inner.hit_window);

        let total_strain = self.inner.curr_strain * self.inner.curr_rhythm;
        self.inner.inner.object_strains.push(total_strain);

        total_strain
    }
}

struct SpeedEvaluator;

impl SpeedEvaluator {
    const SINGLE_SPACING_THRESHOLD: f64 = 125.0; // 1.25 circlers distance between centers
    const MIN_SPEED_BONUS: f64 = 75.0; // ~200BPM
    const SPEED_BALANCING_FACTOR: f64 = 40.0;
    const DIST_MULTIPLIER: f64 = 0.94;

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        hit_window: f64,
        has_autopilot_mod: bool,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        // * derive strainTime for calculation
        let osu_curr_obj = curr;
        let osu_prev_obj = curr.previous(0, diff_objects);
        let osu_next_obj = curr.next(0, diff_objects);

        let mut strain_time = curr.strain_time;
        // Note: Technically `osu_next_obj` is never `None` but instead the
        // default value. This could maybe invalidate the `get_doubletapness`
        // result.
        let doubletapness = 1.0 - osu_curr_obj.get_doubletapness(osu_next_obj, hit_window);

        // * Cap deltatime to the OD 300 hitwindow.
        // * 0.93 is derived from making sure 260bpm OD8 streams aren't nerfed harshly, whilst 0.92 limits the effect of the cap.
        strain_time /= ((strain_time / hit_window) / 0.93).clamp(0.92, 1.0);

        let speed_bonus = if strain_time < Self::MIN_SPEED_BONUS {
            // * Add additional scaling bonus for streams/bursts higher than 200bpm
            let base = (Self::MIN_SPEED_BONUS - strain_time) / Self::SPEED_BALANCING_FACTOR;

            0.75 * base.powf(2.0)
        } else {
            // * speedBonus will be 0.0 for BPM < 200
            0.0
        };

        let travel_dist = osu_prev_obj.map_or(0.0, |obj| obj.travel_dist);
        let mut dist = if has_autopilot_mod {
            0.0
        } else {
            travel_dist + osu_curr_obj.min_jump_dist
        };

        // * Cap distance at single_spacing_threshold
        dist = Self::SINGLE_SPACING_THRESHOLD.min(dist);

        // * Max distance bonus is 1 * `distance_multiplier` at single_spacing_threshold
        let dist_bonus = (dist / Self::SINGLE_SPACING_THRESHOLD).powf(3.95) * Self::DIST_MULTIPLIER;

        // * Base difficulty with all bonuses
        let difficulty = (1.0 + speed_bonus + dist_bonus) * 1000.0 / strain_time;

        // * Apply penalty if there's doubletappable doubles
        difficulty * doubletapness
    }
}

struct RhythmEvaluator;

impl RhythmEvaluator {
    const HISTORY_TIME_MAX: u32 = 5 * 1000; // 5 seconds
    const HISTORY_OBJECTS_MAX: usize = 32;
    const RHYTHM_OVERALL_MULTIPLIER: f64 = 0.95;
    const RHYTHM_RATIO_MULTIPLIER: f64 = 12.0;

    #[allow(clippy::too_many_lines)]
    fn evaluate_diff_of<'a>(
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

                let curr_delta = curr_obj.strain_time;
                let prev_delta = prev_obj.strain_time;
                let last_delta = last_obj.strain_time;

                // * calculate how much current delta difference deserves a rhythm bonus
                // * this function is meant to reduce rhythm bonus for deltas that are multiples of each other (i.e 100 and 200)
                let delta_difference_ratio =
                    prev_delta.min(curr_delta) / prev_delta.max(curr_delta);
                let curr_ratio = 1.0
                    + Self::RHYTHM_RATIO_MULTIPLIER
                        * (PI / delta_difference_ratio).sin().powf(2.0).min(0.5);

                // reduce ratio bonus if delta difference is too big
                let fraction = (prev_delta / curr_delta).max(curr_delta / prev_delta);
                let fraction_multiplier = (2.0 - fraction / 8.0).clamp(0.0, 1.0);

                let window_penalty = (((prev_delta - curr_delta).abs() - delta_difference_eps)
                    .max(0.0)
                    / delta_difference_eps)
                    .min(1.0);

                let mut effective_ratio = window_penalty * curr_ratio * fraction_multiplier;

                if first_delta_switch {
                    // Keep in-sync with lazer
                    #[allow(clippy::if_not_else)]
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
                            let power = logistic(f64::from(island.delta), 2.75, 0.24, 14.0);
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
        (4.0 + rhythm_complexity_sum * Self::RHYTHM_OVERALL_MULTIPLIER).sqrt() / 2.0
    }
}

fn logistic(x: f64, max_value: f64, multiplier: f64, offset: f64) -> f64 {
    max_value / (1.0 + E.powf(offset - (multiplier * x)))
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
