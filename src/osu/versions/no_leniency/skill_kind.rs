use std::{collections::VecDeque, f32::consts::PI, iter};

use crate::{math_util, parse::Pos2};

use super::DifficultyObject;

const SINGLE_SPACING_TRESHOLD: f32 = 125.0;

const MIN_SPEED_BONUS: f32 = 75.0;
const SPEED_BALANCING_FACTOR: f32 = 40.0;

const TIMING_THRESHOLD: f32 = 107.0;

const AIM_SKILL_MULTIPLIER: f32 = 26.25;
const AIM_STRAIN_DECAY_BASE: f32 = 0.15;
const AIM_DECAY_WEIGHT: f32 = 0.9;
const AIM_DIFFICULTY_MULTIPLIER: f32 = 1.06;
const AIM_REDUCED_SECTION_COUNT: usize = 10;

const AIM_ANGLE_BONUS_BEGIN: f32 = std::f32::consts::FRAC_PI_3;

const SPEED_SKILL_MULTIPLIER: f32 = 1375.0;
const SPEED_STRAIN_DECAY_BASE: f32 = 0.3;
const SPEED_DECAY_WEIGHT: f32 = 0.9;
const SPEED_DIFFICULTY_MULTIPLIER: f32 = 1.04;
const SPEED_REDUCED_SECTION_COUNT: usize = 5;

const SPEED_HISTORY_LENGTH: usize = 32;
const SPEED_HISTORY_TIME_MAX: f32 = 5000.0;
const SPEED_RHYTHM_MULTIPLIER: f32 = 0.75;

const FLASHLIGHT_SKILL_MULTIPLIER: f32 = 0.15;
const FLASHLIGHT_STRAIN_DECAY_BASE: f32 = 0.15;
const FLASHLIGHT_DECAY_WEIGHT: f32 = 1.0;
const FLASHLIGHT_DIFFICULTY_MULTIPLIER: f32 = 1.06;
const FLASHLIGHT_REDUCED_SECTION_COUNT: usize = 10;

const FLASHLIGHT_HISTORY_LENGTH: usize = 10;

pub(crate) struct FlashlightHistoryEntry {
    end_pos: Pos2,
    is_spinner: bool,
    jump_dist: f32,
    strain_time: f32,
}

impl From<&DifficultyObject<'_>> for FlashlightHistoryEntry {
    fn from(h: &DifficultyObject) -> Self {
        Self {
            end_pos: h.base.end_pos(),
            is_spinner: h.base.is_spinner(),
            jump_dist: h.jump_dist,
            strain_time: h.strain_time,
        }
    }
}

pub(crate) struct SpeedHistoryEntry {
    is_slider: bool,
    start_time: f32,
    strain_time: f32,
}

impl From<&DifficultyObject<'_>> for SpeedHistoryEntry {
    fn from(h: &DifficultyObject) -> Self {
        Self {
            is_slider: h.base.is_slider(),
            start_time: h.base.time,
            strain_time: h.strain_time,
        }
    }
}

pub(crate) enum SkillKind {
    Aim,
    Flashlight {
        history: VecDeque<FlashlightHistoryEntry>,
        scaling_factor: f32,
    },
    Speed {
        curr_rhythm: f32,
        history: VecDeque<SpeedHistoryEntry>,
        hit_window: f32,
    },
}

impl SkillKind {
    pub(crate) fn flashlight(scaling_factor: f32) -> Self {
        Self::Flashlight {
            history: VecDeque::with_capacity(FLASHLIGHT_HISTORY_LENGTH),
            scaling_factor,
        }
    }

    pub(crate) fn speed(hit_window: f32) -> Self {
        Self::Speed {
            curr_rhythm: 1.0,
            history: VecDeque::with_capacity(SPEED_HISTORY_LENGTH),
            hit_window,
        }
    }

    pub(crate) fn pre_process(&mut self) {
        match self {
            Self::Aim => {}
            Self::Flashlight { history, .. } => history.truncate(FLASHLIGHT_HISTORY_LENGTH),
            Self::Speed { history, .. } => history.truncate(SPEED_HISTORY_LENGTH),
        }
    }

    pub(crate) fn post_process(&mut self, current: &DifficultyObject) {
        match self {
            Self::Aim => {}
            Self::Flashlight { history, .. } => history.push_front(current.into()),
            Self::Speed { history, .. } => history.push_front(current.into()),
        }
    }

    pub(crate) fn strain_value_of(&self, curr: &DifficultyObject) -> f32 {
        match self {
            Self::Aim => {
                if curr.base.is_spinner() {
                    return 0.0;
                }

                let mut aim_strain = 0.0;

                if let Some((prev_jump_dist, prev_strain_time)) = curr.prev {
                    if let Some(angle) = curr.angle.filter(|a| *a > AIM_ANGLE_BONUS_BEGIN) {
                        let scale = 90.0;

                        let angle_bonus = (((angle - AIM_ANGLE_BONUS_BEGIN).sin()).powi(2)
                            * (prev_jump_dist - scale).max(0.0)
                            * (curr.jump_dist - scale).max(0.0))
                        .sqrt();

                        aim_strain = 1.4 * apply_diminishing_exp(angle_bonus.max(0.0))
                            / (TIMING_THRESHOLD).max(prev_strain_time)
                    }
                }

                let jump_dist_exp = apply_diminishing_exp(curr.jump_dist);
                let travel_dist_exp = apply_diminishing_exp(curr.travel_dist);

                let dist_exp =
                    jump_dist_exp + travel_dist_exp + (travel_dist_exp * jump_dist_exp).sqrt();

                (aim_strain + dist_exp / (curr.strain_time).max(TIMING_THRESHOLD))
                    .max(dist_exp / curr.strain_time)
            }
            Self::Flashlight {
                history,
                scaling_factor,
            } => {
                if curr.base.is_spinner() {
                    return 0.0;
                }

                let mut small_dist_nerf = 1.0;
                let mut result = 0.0;
                let mut cumulative_strain_time = 0.0;
                let mut history_iter = history.iter();

                if let Some(prev) = history_iter.next() {
                    // Handle first entry distinctly for slight optimization
                    if !prev.is_spinner {
                        let jump_dist = (curr.base.pos - prev.end_pos).length();
                        cumulative_strain_time += prev.strain_time;

                        // We want to nerf objects that can be easily seen within the Flashlight circle radius
                        small_dist_nerf = (jump_dist / 75.0).min(1.0);

                        // We also want to nerf stacks so that only the first object of the stack is accounted for
                        // -- since jump distance is 0 on stacked notes in this version, approximate value as 0.2
                        let stack_nerf =
                            ((prev.jump_dist / scaling_factor) / 25.0).min(1.0).max(0.2);

                        result += stack_nerf * scaling_factor * jump_dist / cumulative_strain_time;
                    }

                    let factors = iter::successors(Some(0.8), |s| Some(s * 0.8));

                    for (factor, prev) in factors.zip(history_iter) {
                        if !prev.is_spinner {
                            let jump_dist = (curr.base.pos - prev.end_pos).length();
                            cumulative_strain_time += prev.strain_time;
                            let stack_nerf =
                                ((prev.jump_dist / scaling_factor) / 25.0).min(1.0).max(0.2);

                            result += factor * stack_nerf * scaling_factor * jump_dist
                                / cumulative_strain_time;
                        }
                    }
                }

                result *= small_dist_nerf;

                result * result
            }
            Self::Speed {
                history,
                hit_window,
                ..
            } => {
                if curr.base.is_spinner() {
                    return 0.0;
                }

                let mut strain_time = curr.strain_time;
                let hit_window_full = hit_window * 2.0;
                let speed_window_ratio = strain_time / hit_window_full;
                let prev = history.front();

                // Aim to nerf cheesy rhythms (very fast consecutive doubles with large delta times between)
                if let Some(prev) =
                    prev.filter(|p| strain_time < hit_window_full && p.strain_time > strain_time)
                {
                    strain_time =
                        math_util::lerp(prev.strain_time, strain_time, speed_window_ratio);
                }

                // Cap delta time to the OD 300 hit window
                // 0.93 is derived from making sure 260bpm OD8 streams aren't nerfed harshly,
                // whilst 0.92 limits the effect of the cap
                strain_time /= (strain_time / hit_window_full / 0.93).clamp(0.92, 1.0);

                // Derive speed bonus for calculation
                let mut speed_bonus = 1.0;

                if strain_time < MIN_SPEED_BONUS {
                    let base = (MIN_SPEED_BONUS - strain_time) / SPEED_BALANCING_FACTOR;
                    speed_bonus = 1.0 + 0.75 * base * base;
                }

                let dist = SINGLE_SPACING_TRESHOLD.min(curr.travel_dist + curr.jump_dist);

                (speed_bonus + speed_bonus * (dist / SINGLE_SPACING_TRESHOLD).powf(3.5))
                    / strain_time
            }
        }
    }

    #[inline]
    pub(crate) fn difficulty_values(&self) -> (usize, f32) {
        match self {
            Self::Aim => (AIM_REDUCED_SECTION_COUNT, AIM_DIFFICULTY_MULTIPLIER),
            Self::Flashlight { .. } => (
                FLASHLIGHT_REDUCED_SECTION_COUNT,
                FLASHLIGHT_DIFFICULTY_MULTIPLIER,
            ),
            Self::Speed { .. } => (SPEED_REDUCED_SECTION_COUNT, SPEED_DIFFICULTY_MULTIPLIER),
        }
    }

    #[inline]
    pub(crate) fn skill_multiplier(&self) -> f32 {
        match self {
            SkillKind::Aim => AIM_SKILL_MULTIPLIER,
            SkillKind::Flashlight { .. } => FLASHLIGHT_SKILL_MULTIPLIER,
            SkillKind::Speed { .. } => SPEED_SKILL_MULTIPLIER,
        }
    }

    #[inline]
    pub(crate) fn strain_decay_base(&self) -> f32 {
        match self {
            SkillKind::Aim => AIM_STRAIN_DECAY_BASE,
            SkillKind::Flashlight { .. } => FLASHLIGHT_STRAIN_DECAY_BASE,
            SkillKind::Speed { .. } => SPEED_STRAIN_DECAY_BASE,
        }
    }

    #[inline]
    pub(crate) fn decay_weight(&self) -> f32 {
        match self {
            SkillKind::Aim => AIM_DECAY_WEIGHT,
            SkillKind::Flashlight { .. } => FLASHLIGHT_DECAY_WEIGHT,
            SkillKind::Speed { .. } => SPEED_DECAY_WEIGHT,
        }
    }

    #[inline]
    pub(crate) fn strain_decay(&self, ms: f32) -> f32 {
        self.strain_decay_base().powf(ms / 1000.0)
    }
}

pub(crate) fn calculate_speed_rhythm_bonus(
    current: &DifficultyObject,
    history: &VecDeque<SpeedHistoryEntry>,
    hit_window: f32,
) -> f32 {
    if current.base.is_spinner() {
        return 0.0;
    }

    let mut prev_island_size = 0;
    let mut rhythm_complexity_sum = 0.0;
    let mut island_size = 1;
    let mut first_delta_switch = false;
    let adjusted_hit_window = hit_window * 0.6;
    let history_len = history.len() as f32;

    // Store the ratio of the current start of an island to buff for tighter rhythms
    let mut start_ratio = 0.0;

    let currs = history.iter();
    let prevs = history.iter().skip(1);
    let lasts = history.iter().skip(2);

    for (((prev, curr), last), i) in prevs.zip(currs).zip(lasts).rev().zip(2..) {
        let mut curr_historical_decay =
            (SPEED_HISTORY_TIME_MAX - (current.base.time - curr.start_time)).max(0.0)
                / SPEED_HISTORY_TIME_MAX;

        if curr_historical_decay.abs() > f32::EPSILON {
            // Either we're limited by time or limited by object count
            curr_historical_decay = curr_historical_decay.min(i as f32 / history_len);

            let curr_delta = curr.strain_time;
            let prev_delta = prev.strain_time;
            let last_delta = last.strain_time;

            // Fancy function to calculate rhythm bonuses
            let base = (PI / (prev_delta.min(curr_delta) / prev_delta.max(curr_delta))).sin();
            let curr_ratio = 1.0 + 6.0 * (base * base).min(0.5);

            let lower_penalty = ((prev_delta - curr_delta).abs() - adjusted_hit_window).max(0.0);
            let window_penalty = (lower_penalty / adjusted_hit_window).min(1.0);

            let mut effective_ratio = window_penalty * curr_ratio;

            if first_delta_switch {
                if !(prev_delta > 1.25 * curr_delta || prev_delta * 1.25 < curr_delta) {
                    if island_size < 7 {
                        island_size += 1;
                    }
                } else {
                    if curr.is_slider {
                        // bpm change is into slider, this is easy acc window
                        effective_ratio *= 0.125;
                    }

                    if prev.is_slider {
                        // bpm change was from a slider, this is easier typically than circle -> circle
                        effective_ratio *= 0.25;
                    }

                    if prev_island_size == island_size {
                        // repeated island size (ex: triplet -> triplet)
                        effective_ratio *= 0.25;
                    }

                    if prev_island_size % 2 == island_size % 2 {
                        // repeated island polarity (2 -> 4, 3 -> 5)
                        effective_ratio *= 0.5;
                    }

                    if last_delta > prev_delta + 10.0 && prev_delta > curr_delta + 10.0 {
                        // previous increase happened a note ago, 1/1 -> 1/2-1/4, don't want to buff this
                        effective_ratio *= 0.125;
                    }

                    rhythm_complexity_sum += (effective_ratio * start_ratio).sqrt()
                        * curr_historical_decay
                        * ((4 + island_size) as f32).sqrt()
                        * ((4 + prev_island_size) as f32).sqrt()
                        / 4.0;

                    start_ratio = effective_ratio;
                    prev_island_size = island_size;
                    island_size = 1;

                    // we're slowing down, stop counting
                    if prev_delta * 1.25 < curr_delta {
                        // if we're speeding up, this stays true and we keep counting island size
                        first_delta_switch = false;
                    }
                }
            } else if prev_delta > 1.25 * curr_delta {
                // we want to be speeding up
                // begin counting island until we change speed again
                first_delta_switch = true;
                start_ratio = effective_ratio;
                island_size = 1;
            }
        }
    }

    // produces multiplier that can be applied to strain. range [1, infinity) (not really though)
    (4.0 + rhythm_complexity_sum * SPEED_RHYTHM_MULTIPLIER).sqrt() / 2.0
}

#[inline]
fn apply_diminishing_exp(val: f32) -> f32 {
    val.powf(0.99)
}
