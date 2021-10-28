use std::collections::VecDeque;

use crate::parse::Pos2;

use super::DifficultyObject;

const SINGLE_SPACING_TRESHOLD: f32 = 125.0;
const SPEED_ANGLE_BONUS_BEGIN: f32 = 5.0 * std::f32::consts::FRAC_PI_6;
const PI_OVER_4: f32 = std::f32::consts::FRAC_PI_4;
const PI_OVER_2: f32 = std::f32::consts::FRAC_PI_2;

const MIN_SPEED_BONUS: f32 = 75.0;
const MAX_SPEED_BONUS: f32 = 45.0;
const SPEED_BALANCING_FACTOR: f32 = 40.0;

const AIM_ANGLE_BONUS_BEGIN: f32 = std::f32::consts::FRAC_PI_3;
const TIMING_THRESHOLD: f32 = 107.0;

const AIM_REDUCED_SECTION_COUNT: usize = 10;
const FLASHLIGHT_REDUCED_SECTION_COUNT: usize = 10;
const SPEED_REDUCED_SECTION_COUNT: usize = 5;

const FLASHLIGHT_HISTORY_LENGTH: usize = 10;
const SPEED_HISTORY_LENGTH: usize = 32;

const AIM_DIFFICULTY_MULTIPLIER: f32 = 1.06;
const FLASHLIGHT_DIFFICULTY_MULTIPLIER: f32 = 1.06;
const SPEED_DIFFICULTY_MULTIPLIER: f32 = 1.04;

const SPEED_HISTORY_TIME_MAX: f32 = 4.0;
const SPEED_RHYTHM_MULTIPLIER: f32 = 1.0;

pub(crate) struct FlashlightHistoryEntry {
    is_spinner: bool,
    end_pos: Pos2,
    strain_time: f32,
}

pub(crate) struct SpeedHistoryEntry {
    is_slider: bool,
    start_time: f32,
    strain_time: f32,
}

pub(crate) enum SkillKind {
    Aim,
    Flashlight {
        history: VecDeque<FlashlightHistoryEntry>,
        scaling_factor: f32,
    },
    Speed {
        history: VecDeque<SpeedHistoryEntry>,
    },
}

impl SkillKind {
    pub(crate) fn flashlight(scaling_factor: f32) -> Self {
        Self::Flashlight {
            history: VecDeque::with_capacity(FLASHLIGHT_HISTORY_LENGTH),
            scaling_factor,
        }
    }

    pub(crate) fn speed() -> Self {
        Self::Speed {
            history: VecDeque::with_capacity(SPEED_HISTORY_LENGTH),
        }
    }

    pub(crate) fn pre_process(&mut self) {
        match self {
            Self::Aim => {}
            Self::Flashlight { history, .. } => history.truncate(FLASHLIGHT_HISTORY_LENGTH),
            Self::Speed { history } => history.truncate(SPEED_HISTORY_LENGTH),
        }
    }

    pub(crate) fn post_process(&mut self, current: &DifficultyObject) {
        match self {
            Self::Aim => {}
            Self::Flashlight { history, .. } => {
                let entry = FlashlightHistoryEntry {
                    is_spinner: current.base.is_spinner(),
                    end_pos: current.base.end_pos(),
                    strain_time: current.strain_time,
                };

                history.push_front(entry);
            }
            Self::Speed { history } => {
                let entry = SpeedHistoryEntry {
                    is_slider: current.base.is_slider(),
                    start_time: current.base.time,
                    strain_time: current.strain_time,
                };

                history.push_front(entry);
            }
        }
    }

    pub(crate) fn strain_value_of(&self, current: &DifficultyObject) -> f32 {
        match self {
            Self::Aim => {
                if current.base.is_spinner() {
                    return 0.0;
                }

                let mut result = 0.0;

                if let Some((prev_jump_dist, prev_strain_time)) = current.prev {
                    if let Some(angle) = current.angle.filter(|a| *a > AIM_ANGLE_BONUS_BEGIN) {
                        let scale = 90.0;

                        let angle_bonus = (((angle - AIM_ANGLE_BONUS_BEGIN).sin()).powi(2)
                            * (prev_jump_dist - scale).max(0.0)
                            * (current.jump_dist - scale).max(0.0))
                        .sqrt();

                        result = 1.4 * apply_diminishing_exp(angle_bonus.max(0.0))
                            / (TIMING_THRESHOLD).max(prev_strain_time)
                    }
                }

                let jump_dist_exp = apply_diminishing_exp(current.jump_dist);
                let travel_dist_exp = apply_diminishing_exp(current.travel_dist);

                let dist_exp =
                    jump_dist_exp + travel_dist_exp + (travel_dist_exp * jump_dist_exp).sqrt();

                (result + dist_exp / (current.strain_time).max(TIMING_THRESHOLD))
                    .max(dist_exp / current.strain_time)
            }
            Self::Flashlight {
                history,
                scaling_factor,
            } => {
                if current.base.is_spinner() {
                    return 0.0;
                }

                let mut small_dist_nerf = 1.0;

                let mut result = 0.0;
                let mut cumulative_strain_time = 0.0;
                let mut history = history.iter();

                if let Some(entry) = history.next() {
                    if !entry.is_spinner {
                        let jump_dist = (current.base.pos - entry.end_pos).length();
                        cumulative_strain_time += entry.strain_time;

                        // We want to nerf objects that can be easily seen within the Flashlight circle radius
                        if jump_dist < 50.0 {
                            small_dist_nerf = jump_dist / 50.0;
                        }

                        result += scaling_factor * jump_dist / cumulative_strain_time;
                    }

                    for (i, entry) in (1..).zip(history) {
                        if !entry.is_spinner {
                            let jump_dist = (current.base.pos - entry.end_pos).length();
                            cumulative_strain_time += entry.strain_time;

                            result += 0.8_f32.powi(i) * scaling_factor * jump_dist
                                / cumulative_strain_time;
                        }
                    }
                }

                result *= small_dist_nerf;

                result * result
            }
            Self::Speed { .. } => {
                if current.base.is_spinner() {
                    return 0.0;
                }

                let dist = SINGLE_SPACING_TRESHOLD.min(current.travel_dist + current.jump_dist);
                let delta_time = MAX_SPEED_BONUS.max(current.delta);

                let mut speed_bonus = 1.0;

                if delta_time < MIN_SPEED_BONUS {
                    let exp_base = (MIN_SPEED_BONUS - delta_time) / SPEED_BALANCING_FACTOR;
                    speed_bonus = 1.0 + exp_base * exp_base;
                }

                let mut angle_bonus = 1.0;

                if let Some(angle) = current.angle.filter(|a| *a < SPEED_ANGLE_BONUS_BEGIN) {
                    let exp_base = (1.5 * (SPEED_ANGLE_BONUS_BEGIN - angle)).sin();
                    angle_bonus = 1.0 + exp_base * exp_base / 3.57;

                    if angle < PI_OVER_2 {
                        angle_bonus = 1.28;

                        if dist < 90.0 && angle < PI_OVER_4 {
                            angle_bonus += (1.0 - angle_bonus) * ((90.0 - dist) / 10.0).min(1.0);
                        } else if dist < 90.0 {
                            angle_bonus += (1.0 - angle_bonus)
                                * ((90.0 - dist) / 10.0).min(1.0)
                                * ((PI_OVER_2 - angle) / PI_OVER_4).sin();
                        }
                    }
                }

                (1.0 + (speed_bonus - 1.0) * 0.75)
                    * angle_bonus
                    * (0.95 + speed_bonus * (dist / SINGLE_SPACING_TRESHOLD).powf(3.5))
                    / current.strain_time
            }
        }
    }

    pub(crate) fn total_current_strain(
        &self,
        current_strain: f32,
        current: &DifficultyObject,
    ) -> f32 {
        match self {
            SkillKind::Aim | SkillKind::Flashlight { .. } => current_strain,
            SkillKind::Speed { history } => {
                current_strain * calculate_speed_rhythm_bonus(history, current.base.time)
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
}

fn calculate_speed_rhythm_bonus(history: &VecDeque<SpeedHistoryEntry>, start_time: f32) -> f32 {
    let mut previous_island_size = usize::MAX;
    let mut island_times = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let mut island_size = 0;
    let mut first_delta_switch = false;

    for (prev, curr) in history.iter().skip(1).zip(history) {
        let prev_delta = prev.strain_time;
        let curr_delta = curr.strain_time;

        let mut effective_ratio = prev_delta.min(curr_delta) / prev_delta.max(curr_delta);

        let curr_historical_decay = (SPEED_HISTORY_TIME_MAX - (start_time - curr.start_time))
            .max(0.0)
            / SPEED_HISTORY_TIME_MAX;

        if first_delta_switch {
            if is_ratio_equal(1.0, prev_delta, curr_delta) {
                island_size += 1;
            } else {
                if island_size > 6 {
                    island_size = 6;
                }

                if curr.is_slider {
                    effective_ratio /= 2.0;
                }

                if prev.is_slider {
                    effective_ratio *= 0.75;
                }

                if previous_island_size == island_size {
                    effective_ratio /= 0.5;
                }

                island_times[island_size] += effective_ratio * curr_historical_decay;
                previous_island_size = island_size;

                if prev_delta * 1.25 < curr_delta {
                    first_delta_switch = false;
                }

                island_size = 6;
            }
        } else if prev_delta > 1.25 * curr_delta {
            first_delta_switch = true;
            island_size = 0;
        }
    }

    let rhythm_complexity_sum: f32 = island_times.iter().sum();

    ((4.0 + rhythm_complexity_sum * SPEED_RHYTHM_MULTIPLIER).sqrt() / 2.0).min(1.5)
}

#[inline]
fn apply_diminishing_exp(val: f32) -> f32 {
    val.powf(0.99)
}

#[inline]
fn is_ratio_equal(ratio: f32, a: f32, b: f32) -> bool {
    a + 15.0 > ratio * b && a - 15.0 < ratio * b
}
