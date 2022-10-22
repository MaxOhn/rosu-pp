use std::{cmp::Ordering, f64::consts::PI};

use crate::osu::difficulty_object::OsuDifficultyObject;

use super::{next, previous, previous_start_time, OsuStrainSkill, Skill, StrainSkill};

#[derive(Clone, Debug)]
pub(crate) struct Speed {
    curr_strain: f64,
    curr_section_peak: f64,
    curr_section_end: f64,
    curr_rhythm: f64,
    pub(crate) strain_peaks: Vec<f64>,
    object_strains: Vec<f64>,
    hit_window: f64,
}

impl Speed {
    const SKILL_MULTIPLIER: f64 = 1375.0;
    const STRAIN_DECAY_BASE: f64 = 0.3;

    pub(crate) fn new(hit_window: f64) -> Self {
        Self {
            curr_strain: 0.0,
            curr_section_peak: 0.0,
            curr_section_end: 0.0,
            curr_rhythm: 0.0,
            strain_peaks: Vec::new(),
            object_strains: Vec::new(),
            hit_window,
        }
    }

    fn strain_decay(ms: f64) -> f64 {
        Self::STRAIN_DECAY_BASE.powf(ms / 1000.0)
    }

    pub(crate) fn relevant_note_count(&self) -> f64 {
        self.object_strains
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .copied()
            .filter(|&n| n > 0.0)
            .map_or(0.0, |max_strain| {
                self.object_strains.iter().fold(0.0, |sum, strain| {
                    sum + (1.0 + (-(strain / max_strain * 12.0 - 6.0)).exp()).recip()
                })
            })
    }
}

impl Skill for Speed {
    #[inline]
    fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        <Self as StrainSkill>::process(self, curr, diff_objects)
    }

    #[inline]
    fn difficulty_value(&mut self) -> f64 {
        <Self as OsuStrainSkill>::difficulty_value(self)
    }
}

impl StrainSkill for Speed {
    #[inline]
    fn strain_peaks_mut(&mut self) -> &mut Vec<f64> {
        &mut self.strain_peaks
    }

    #[inline]
    fn curr_section_peak(&mut self) -> &mut f64 {
        &mut self.curr_section_peak
    }

    #[inline]
    fn curr_section_end(&mut self) -> &mut f64 {
        &mut self.curr_section_end
    }

    #[inline]
    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.curr_strain *= Self::strain_decay(curr.strain_time);
        self.curr_strain += SpeedEvaluator::evaluate_diff_of(curr, diff_objects, self.hit_window)
            * Self::SKILL_MULTIPLIER;
        self.curr_rhythm = RhythmEvaluator::evaluate_diff_of(curr, diff_objects, self.hit_window);

        let total_strain = self.curr_strain * self.curr_rhythm;
        self.object_strains.push(total_strain);

        total_strain
    }

    #[inline]
    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        (self.curr_strain * self.curr_rhythm)
            * Self::strain_decay(time - previous_start_time(diff_objects, curr.idx, 0))
    }

    #[inline]
    fn difficulty_value(&mut self) -> f64 {
        <Self as OsuStrainSkill>::difficulty_value(self)
    }
}

impl OsuStrainSkill for Speed {
    const REDUCED_SECTION_COUNT: usize = 5;
    const DIFFICULTY_MULTIPLER: f64 = 1.04;
}

struct SpeedEvaluator;

impl SpeedEvaluator {
    const SINGLE_SPACING_THRESHOLD: f64 = 125.0;
    const MIN_SPEED_BONUS: f64 = 75.0; // ~200BPM
    const SPEED_BALANCING_FACTOR: f64 = 40.;

    fn evaluate_diff_of(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        hit_window: f64,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        // * derive strainTime for calculation
        let osu_curr_obj = curr;
        let osu_prev_obj = previous(diff_objects, curr.idx, 0);
        let osu_next_obj = next(diff_objects, curr.idx, 0);

        let mut strain_time = curr.strain_time;
        let mut doubletapness = 1.0;

        // * Nerf doubletappable doubles.
        if let Some(osu_next_obj) = osu_next_obj {
            let curr_delta_time = osu_curr_obj.delta_time.max(1.0);
            let next_delta_time = osu_next_obj.delta_time.max(1.0);
            let delta_diff = (next_delta_time - curr_delta_time).abs();
            let speed_ratio = curr_delta_time / curr_delta_time.max(delta_diff);
            let window_ratio_base = (curr_delta_time / hit_window).min(1.0);
            let window_ratio = window_ratio_base * window_ratio_base;
            doubletapness = speed_ratio.powf(1.0 - window_ratio);
        }

        // * Cap deltatime to the OD 300 hitwindow.
        // * 0.93 is derived from making sure 260bpm OD8 streams aren't nerfed harshly, whilst 0.92 limits the effect of the cap.
        strain_time /= ((strain_time / hit_window) / 0.93).clamp(0.92, 1.0);

        // * derive speedBonus for calculation
        let speed_bonus = if strain_time < Self::MIN_SPEED_BONUS {
            let base = (Self::MIN_SPEED_BONUS - strain_time) / Self::SPEED_BALANCING_FACTOR;

            1.0 + 0.75 * base * base
        } else {
            1.0
        };

        let travel_dist = osu_prev_obj.map_or(0.0, |obj| obj.dists.travel_dist);
        let dist =
            Self::SINGLE_SPACING_THRESHOLD.min(travel_dist + osu_curr_obj.dists.min_jump_dist);

        (speed_bonus + speed_bonus * (dist / Self::SINGLE_SPACING_THRESHOLD).powf(3.5))
            * doubletapness
            / strain_time
    }
}

struct RhythmEvaluator;

impl RhythmEvaluator {
    // * 5 seconds of calculatingRhythmBonus max.
    const HISTORY_TIME_MAX: u32 = 5000;
    const RHYTHM_MULTIPLIER: f64 = 0.75;

    fn evaluate_diff_of(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        hit_window: f64,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let mut prev_island_size = 0;

        let mut rhythm_complexity_sum = 0.0;
        let mut island_size = 1;
        // * store the ratio of the current start of an island to buff for tighter rhythms
        let mut start_ratio = 0.0;

        let mut first_delta_switch = false;

        let historical_note_count = curr.idx.min(32);

        let mut rhythm_start = 0;

        while previous(diff_objects, curr.idx, rhythm_start)
            .filter(|prev| {
                rhythm_start + 2 < historical_note_count
                    && curr.start_time - prev.start_time < Self::HISTORY_TIME_MAX as f64
            })
            .is_some()
        {
            rhythm_start += 1;
        }

        for i in (1..=rhythm_start).rev() {
            let (curr_obj, prev_obj, last_obj) = if let Some(((curr, prev), last)) =
                previous(diff_objects, curr.idx, i - 1)
                    .zip(previous(diff_objects, curr.idx, i))
                    .zip(previous(diff_objects, curr.idx, i + 1))
            {
                (curr, prev, last)
            } else {
                break;
            };

            // * scales note 0 to 1 from history to now
            let mut curr_historical_decay = (Self::HISTORY_TIME_MAX as f64
                - (curr.start_time - curr_obj.start_time))
                / Self::HISTORY_TIME_MAX as f64;

            // * either we're limited by time or limited by object count.
            curr_historical_decay = curr_historical_decay
                .min((historical_note_count - i) as f64 / historical_note_count as f64);

            let curr_delta = curr_obj.strain_time;
            let prev_delta = prev_obj.strain_time;
            let last_delta = last_obj.strain_time;

            // * fancy function to calculate rhythmbonuses.
            let base = (PI / (prev_delta.min(curr_delta) / prev_delta.max(curr_delta))).sin();
            let curr_ratio = 1.0 + 6.0 * (base * base).min(0.5);

            let hit_window = !curr_obj.base.is_spinner() as u64 as f64 * hit_window;

            let mut window_penalty = ((((prev_delta - curr_delta).abs() - hit_window * 0.3)
                .max(0.0))
                / (hit_window * 0.3))
                .min(1.0);

            window_penalty = window_penalty.min(1.0);

            let mut effective_ratio = window_penalty * curr_ratio;

            if first_delta_switch {
                if !(prev_delta > 1.25 * curr_delta || prev_delta * 1.25 < curr_delta) {
                    if island_size < 7 {
                        // * island is still progressing, count size.
                        island_size += 1;
                    }
                } else {
                    // * bpm change is into slider, this is easy acc window
                    if curr_obj.base.is_slider() {
                        effective_ratio *= 0.125;
                    }

                    // * bpm change was from a slider, this is easier typically than circle -> circle
                    if prev_obj.base.is_slider() {
                        effective_ratio *= 0.25;
                    }

                    // * repeated island size (ex: triplet -> triplet)
                    if prev_island_size == island_size {
                        effective_ratio *= 0.25;
                    }

                    // * repeated island polartiy (2 -> 4, 3 -> 5)
                    if prev_island_size % 2 == island_size % 2 {
                        effective_ratio *= 0.5;
                    }

                    // * previous increase happened a note ago, 1/1->1/2-1/4, dont want to buff this.
                    if last_delta > prev_delta + 10.0 && prev_delta > curr_delta + 10.0 {
                        effective_ratio *= 0.125;
                    }

                    rhythm_complexity_sum += (effective_ratio * start_ratio).sqrt()
                        * curr_historical_decay
                        * ((4 + island_size) as f64).sqrt()
                        / 2.0
                        * ((4 + prev_island_size) as f64).sqrt()
                        / 2.0;

                    start_ratio = effective_ratio;

                    // * log the last island size.
                    prev_island_size = island_size;

                    // * we're slowing down, stop counting
                    if prev_delta * 1.25 < curr_delta {
                        // * if we're speeding up, this stays true and  we keep counting island size.
                        first_delta_switch = false;
                    }

                    island_size = 1;
                }
            } else if prev_delta > 1.25 * curr_delta {
                // * we want to be speeding up.
                // * Begin counting island until we change speed again.
                first_delta_switch = true;
                start_ratio = effective_ratio;
                island_size = 1;
            }
        }

        // * produces multiplier that can be applied to strain. range [1, infinity) (not really though)
        (4.0 + rhythm_complexity_sum * Self::RHYTHM_MULTIPLIER).sqrt() / 2.0
    }
}
