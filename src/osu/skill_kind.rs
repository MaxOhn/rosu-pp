use std::{
    collections::VecDeque,
    f64::consts::{FRAC_PI_2, PI},
    fmt, iter,
};

use crate::parse::Pos2;

use super::{lerp, DifficultyObject};

const SINGLE_SPACING_TRESHOLD: f64 = 125.0;

const SPEED_BALANCING_FACTOR: f64 = 40.0;

const AIM_SKILL_MULTIPLIER: f64 = 23.25;
const AIM_STRAIN_DECAY_BASE: f64 = 0.15;
const AIM_DECAY_WEIGHT: f64 = 0.9;
const AIM_DIFFICULTY_MULTIPLIER: f64 = 1.06;
const AIM_REDUCED_SECTION_COUNT: usize = 10;

const AIM_HISTORY_LENGTH: usize = 2;
const AIM_WIDE_ANGLE_MULTIPLIER: f64 = 1.5;
const AIM_ACUTE_ANGLE_MULTIPLIER: f64 = 2.0;
const AIM_SLIDER_MULTIPLIER: f64 = 1.5;
const AIM_VELOCITY_CHANGE_MULTIPLIER: f64 = 0.75;

const SPEED_SKILL_MULTIPLIER: f64 = 1375.0;
const SPEED_STRAIN_DECAY_BASE: f64 = 0.3;
const SPEED_DECAY_WEIGHT: f64 = 0.9;
const SPEED_DIFFICULTY_MULTIPLIER: f64 = 1.04;
const SPEED_REDUCED_SECTION_COUNT: usize = 5;

const SPEED_HISTORY_LENGTH: usize = 32;
const SPEED_RHYTHM_MULTIPLIER: f64 = 0.75;
const SPEED_HISTORY_TIME_MAX: f64 = 5000.0; // * 5 seconds of calculate_speed_rhythm_bonus max
const MIN_SPEED_BONUS: f64 = 75.0; // * ~200BPM

const FLASHLIGHT_SKILL_MULTIPLIER: f64 = 0.15;
const FLASHLIGHT_STRAIN_DECAY_BASE: f64 = 0.15;
const FLASHLIGHT_DECAY_WEIGHT: f64 = 1.0;
const FLASHLIGHT_DIFFICULTY_MULTIPLIER: f64 = 1.06;
const FLASHLIGHT_REDUCED_SECTION_COUNT: usize = 10;

const FLASHLIGHT_HISTORY_LENGTH: usize = 10;

#[derive(Clone)]
pub(crate) struct AimHistoryEntry {
    angle: Option<f64>,
    is_slider: bool,
    is_spinner: bool,
    strain_time: f64,
    jump_dist: f64,
    movement_dist: f64,
    movement_time: f64,
    travel_dist: f64,
    travel_time: f64,
}

impl From<&DifficultyObject<'_>> for AimHistoryEntry {
    fn from(h: &DifficultyObject<'_>) -> Self {
        Self {
            angle: h.angle,
            is_slider: h.base.is_slider(),
            is_spinner: h.base.is_spinner(),
            strain_time: h.strain_time,
            jump_dist: h.jump_dist,
            movement_dist: h.movement_dist,
            movement_time: h.movement_time,
            travel_dist: h.travel_dist,
            travel_time: h.travel_time,
        }
    }
}

#[derive(Clone)]
pub(crate) struct FlashlightHistoryEntry {
    end_pos: Pos2,
    is_spinner: bool,
    jump_dist: f64,
    strain_time: f64,
}

impl From<&DifficultyObject<'_>> for FlashlightHistoryEntry {
    fn from(h: &DifficultyObject<'_>) -> Self {
        Self {
            end_pos: h.base.end_pos(),
            is_spinner: h.base.is_spinner(),
            jump_dist: h.jump_dist,
            strain_time: h.strain_time,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SpeedHistoryEntry {
    is_slider: bool,
    start_time: f64,
    strain_time: f64,
}

impl From<&DifficultyObject<'_>> for SpeedHistoryEntry {
    fn from(h: &DifficultyObject<'_>) -> Self {
        Self {
            is_slider: h.base.is_slider(),
            start_time: h.base.time / h.clock_rate,
            strain_time: h.strain_time,
        }
    }
}

#[derive(Clone)]
pub(crate) enum SkillKind {
    Aim {
        history: VecDeque<AimHistoryEntry>,
        with_sliders: bool,
    },
    Flashlight {
        history: VecDeque<FlashlightHistoryEntry>,
        scaling_factor: f64,
    },
    Speed {
        curr_rhythm: f64,
        history: VecDeque<SpeedHistoryEntry>,
        hit_window: f64,
    },
}

impl SkillKind {
    pub(crate) fn aim(with_sliders: bool) -> Self {
        Self::Aim {
            history: VecDeque::with_capacity(AIM_HISTORY_LENGTH + 1),
            with_sliders,
        }
    }

    pub(crate) fn flashlight(scaling_factor: f64) -> Self {
        Self::Flashlight {
            history: VecDeque::with_capacity(FLASHLIGHT_HISTORY_LENGTH + 1),
            scaling_factor,
        }
    }

    pub(crate) fn speed(hit_window: f64) -> Self {
        Self::Speed {
            curr_rhythm: 1.0,
            history: VecDeque::with_capacity(SPEED_HISTORY_LENGTH + 1),
            hit_window,
        }
    }

    pub(crate) fn pre_process(&mut self) {
        match self {
            Self::Aim { history, .. } => history.truncate(AIM_HISTORY_LENGTH),
            Self::Flashlight { history, .. } => history.truncate(FLASHLIGHT_HISTORY_LENGTH),
            Self::Speed { history, .. } => history.truncate(SPEED_HISTORY_LENGTH),
        }
    }

    pub(crate) fn post_process(&mut self, current: &DifficultyObject<'_>) {
        match self {
            Self::Aim { history, .. } => history.push_front(current.into()),
            Self::Flashlight { history, .. } => history.push_front(current.into()),
            Self::Speed { history, .. } => history.push_front(current.into()),
        }
    }

    pub(crate) fn strain_value_of(&self, curr: &DifficultyObject<'_>) -> f64 {
        match self {
            Self::Aim {
                history,
                with_sliders,
            } => {
                if curr.base.is_spinner() || history.len() < 2 || history[0].is_spinner {
                    return 0.0;
                }

                let prev = &history[0];
                let prev_prev = &history[1];

                // * Calculate the velocity to the current hitobject,
                // * which starts with a base distance / time assuming the last object is a hitcircle.
                let mut curr_velocity = curr.jump_dist / curr.strain_time;

                // * But if the last object is a slider, then we extend the
                // * travel velocity through the slider into the current object.
                if prev.is_slider && *with_sliders {
                    // * calculate the movement velocity from slider end to current object
                    let movement_velocity = curr.movement_dist / curr.movement_time;

                    // * calculate the slider velocity from slider head to slider end.
                    let travel_velocity = curr.travel_dist / curr.travel_time;

                    // * take the larger total combined velocity.
                    curr_velocity = curr_velocity.max(movement_velocity + travel_velocity);
                }

                // * As above, do the same for the previous hitobject.
                let mut prev_velocity = prev.jump_dist / prev.strain_time;

                if prev_prev.is_slider && *with_sliders {
                    let movement_velocity = prev.movement_dist / prev.movement_time;
                    let travel_velocity = prev.travel_dist / prev.travel_time;
                    prev_velocity = prev_velocity.max(movement_velocity + travel_velocity);
                }

                let mut wide_angle_bonus = 0.0;
                let mut acute_angle_bonus = 0.0;
                let mut slider_bonus = 0.0;
                let mut velocity_change_bonus = 0.0;

                // * Start strain with regular velocity
                let mut aim_strain = curr_velocity;

                // * If rhythms are the same.
                if curr.strain_time.max(prev.strain_time)
                    < 1.25 * curr.strain_time.min(prev.strain_time)
                {
                    if let (Some(curr_angle), Some(prev_angle), Some(prev_prev_angle)) =
                        (curr.angle, prev.angle, prev_prev.angle)
                    {
                        // * Rewarding angles, take the smaller velocity as base.
                        let angle_bonus = curr_velocity.min(prev_velocity);

                        wide_angle_bonus = calculate_wide_angle_bonus(curr_angle);

                        // * Only bufff delta_time exceeding 300 bpm 1/2.
                        if curr.strain_time <= 100.0 {
                            let curr_bonus = calculate_acute_angle_bonus(curr_angle);

                            // * Multiply by previous angle, we don't want to buff unless this is a wiggle type pattern.
                            let prev_bonus = calculate_acute_angle_bonus(prev_angle);

                            // * The maximum velocity we buff is equal to 125 / strainTime
                            let angle_bonus = angle_bonus.min(125.0 / curr.strain_time);

                            // * scale buff from 150 bpm 1/4 to 200 bpm 1/4
                            let base1 =
                                (FRAC_PI_2 * ((100.0 - curr.strain_time) / 25.0).min(1.0)).sin();

                            // * Buff distance exceeding 50 (radius) up to 100 (diameter).
                            let base2 = (FRAC_PI_2 * (curr.jump_dist.clamp(50.0, 100.0) - 50.0)
                                / 50.0)
                                .sin();

                            acute_angle_bonus = curr_bonus
                                * prev_bonus
                                * angle_bonus
                                * base1
                                * base1
                                * base2
                                * base2
                        }

                        // * Penalize wide angles if they're repeated,
                        // * reducing the penalty as the lastAngle gets more acute.
                        let base = calculate_wide_angle_bonus(prev_angle);
                        wide_angle_bonus *=
                            angle_bonus * (1.0 - wide_angle_bonus.min(base * base * base));

                        // * Penalize acute angles if they're repeated,
                        // * reducing the penalty as the lastLastAngle gets more obtuse.
                        let base = calculate_acute_angle_bonus(prev_prev_angle);
                        acute_angle_bonus *=
                            0.5 + 0.5 * (1.0 - acute_angle_bonus.min(base * base * base));
                    }
                }

                if prev_velocity.max(curr_velocity).abs() > f64::EPSILON {
                    // * We want to use the average velocity over the whole object when
                    // * awarding differences, not the individual jump and slider path velocities.
                    prev_velocity = (prev.jump_dist + prev.travel_dist) / prev.strain_time;
                    curr_velocity = (curr.jump_dist + curr.travel_dist) / curr.strain_time;

                    let velocity_diff = (prev_velocity - curr_velocity).abs();

                    // * Scale with ratio of difference compared to 0.5 * max dist.
                    let base = (FRAC_PI_2 * velocity_diff / prev_velocity.max(curr_velocity)).sin();
                    let dist_ratio = base * base;

                    // * Reward for % distance up to 125 / strainTime
                    // * for overlaps where velocity is still changing.
                    let overlap_velocity_buff =
                        velocity_diff.min(125.0 / curr.strain_time.min(prev.strain_time));

                    // * Reward for % distance slowed down compared to previous,
                    // * paying attention to not award overlap
                    let base =
                        (FRAC_PI_2 * (curr.jump_dist.min(prev.jump_dist) / 100.0).min(1.0)).sin();
                    let non_overlap_velocity_buff = velocity_diff * base * base;

                    // * Choose the largest bonus, multiplied by ratio.
                    velocity_change_bonus =
                        overlap_velocity_buff.max(non_overlap_velocity_buff) * dist_ratio;

                    // * Penalize for rhythm changes.
                    let base = curr.strain_time.min(prev.strain_time)
                        / curr.strain_time.max(prev.strain_time);
                    velocity_change_bonus *= base * base;
                }

                if curr.travel_time.abs() > f64::EPSILON {
                    // * Reward sliders based on velocity
                    slider_bonus = curr.travel_dist / curr.travel_time;
                }

                // * Add in acute angle bonus or wide angle bonus + velocity change bonus,
                // * whichever is larger
                aim_strain += (acute_angle_bonus * AIM_ACUTE_ANGLE_MULTIPLIER).max(
                    wide_angle_bonus * AIM_WIDE_ANGLE_MULTIPLIER
                        + velocity_change_bonus * AIM_VELOCITY_CHANGE_MULTIPLIER,
                );

                // * Add in additional slider velocity bonus.
                if *with_sliders {
                    aim_strain += slider_bonus * AIM_SLIDER_MULTIPLIER;
                }

                aim_strain
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
                let mut history = history.iter();

                if let Some(prev) = history.next() {
                    // Handle first entry distinctly for slight optimization
                    if !prev.is_spinner {
                        let jump_dist = (curr.base.pos - prev.end_pos).length() as f64;
                        cumulative_strain_time += prev.strain_time;

                        // * We want to nerf objects that can be easily seen within the Flashlight circle radius
                        small_dist_nerf = (jump_dist / 75.0).min(1.0);

                        // * We also want to nerf stacks so that only the first object of the stack is accounted for
                        let stack_nerf = ((prev.jump_dist / scaling_factor) / 25.0).min(1.0);

                        result += stack_nerf * scaling_factor * jump_dist / cumulative_strain_time;
                    }

                    let factors = iter::successors(Some(0.8), |s| Some(s * 0.8));

                    for (factor, prev) in factors.zip(history) {
                        if !prev.is_spinner {
                            let jump_dist = (curr.base.pos - prev.end_pos).length() as f64;
                            cumulative_strain_time += prev.strain_time;

                            // * We also want to nerf stacks so that only the first object of the stack is accounted for
                            let stack_nerf = ((prev.jump_dist / scaling_factor) / 25.0).min(1.0);

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

                // * Aim to nerf cheesy rhythms (very fast consecutive doubles with large delta times between)
                if let Some(prev) =
                    prev.filter(|p| strain_time < hit_window_full && p.strain_time > strain_time)
                {
                    strain_time = lerp(prev.strain_time, strain_time, speed_window_ratio);
                }

                // * Cap delta time to the OD 300 hit window
                // * 0.93 is derived from making sure 260bpm OD8 streams aren't nerfed harshly,
                // * whilst 0.92 limits the effect of the cap
                strain_time /= (strain_time / hit_window_full / 0.93).clamp(0.92, 1.0);

                // * Derive speed bonus for calculation
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
    pub(crate) fn difficulty_values(&self) -> (usize, f64) {
        match self {
            Self::Aim { .. } => (AIM_REDUCED_SECTION_COUNT, AIM_DIFFICULTY_MULTIPLIER),
            Self::Flashlight { .. } => (
                FLASHLIGHT_REDUCED_SECTION_COUNT,
                FLASHLIGHT_DIFFICULTY_MULTIPLIER,
            ),
            Self::Speed { .. } => (SPEED_REDUCED_SECTION_COUNT, SPEED_DIFFICULTY_MULTIPLIER),
        }
    }

    #[inline]
    pub(crate) fn skill_multiplier(&self) -> f64 {
        match self {
            SkillKind::Aim { .. } => AIM_SKILL_MULTIPLIER,
            SkillKind::Flashlight { .. } => FLASHLIGHT_SKILL_MULTIPLIER,
            SkillKind::Speed { .. } => SPEED_SKILL_MULTIPLIER,
        }
    }

    #[inline]
    pub(crate) fn strain_decay_base(&self) -> f64 {
        match self {
            SkillKind::Aim { .. } => AIM_STRAIN_DECAY_BASE,
            SkillKind::Flashlight { .. } => FLASHLIGHT_STRAIN_DECAY_BASE,
            SkillKind::Speed { .. } => SPEED_STRAIN_DECAY_BASE,
        }
    }

    #[inline]
    pub(crate) fn decay_weight(&self) -> f64 {
        match self {
            SkillKind::Aim { .. } => AIM_DECAY_WEIGHT,
            SkillKind::Flashlight { .. } => FLASHLIGHT_DECAY_WEIGHT,
            SkillKind::Speed { .. } => SPEED_DECAY_WEIGHT,
        }
    }

    #[inline]
    pub(crate) fn strain_decay(&self, ms: f64) -> f64 {
        self.strain_decay_base().powf(ms / 1000.0)
    }
}

pub(crate) fn calculate_speed_rhythm_bonus(
    current: &DifficultyObject<'_>,
    history: &VecDeque<SpeedHistoryEntry>,
    hit_window: f64,
) -> f64 {
    if current.base.is_spinner() {
        return 0.0;
    }

    let mut prev_island_size = 0;
    let mut rhythm_complexity_sum = 0.0;
    let mut island_size = 1;
    let mut first_delta_switch = false;
    let adjusted_hit_window = hit_window * 0.6;
    let history_len = history.len() as f64;

    // * Store the ratio of the current start of an island to buff for tighter rhythms
    let mut start_ratio = 0.0;

    let currs = history.iter();
    let prevs = history.iter().skip(1);
    let lasts = history.iter().skip(2);

    for (((prev, curr), last), i) in prevs.zip(currs).zip(lasts).rev().zip(2..) {
        let mut curr_historical_decay = (SPEED_HISTORY_TIME_MAX
            - (current.base.time / current.clock_rate - curr.start_time))
            .max(0.0)
            / SPEED_HISTORY_TIME_MAX;

        if curr_historical_decay.abs() > f64::EPSILON {
            // * Either we're limited by time or limited by object count
            curr_historical_decay = curr_historical_decay.min(i as f64 / history_len);

            let curr_delta = curr.strain_time;
            let prev_delta = prev.strain_time;
            let last_delta = last.strain_time;

            // * Fancy function to calculate rhythm bonuses
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
                        // * bpm change is into slider, this is easy acc window
                        effective_ratio *= 0.125;
                    }

                    if prev.is_slider {
                        // * bpm change was from a slider, this is easier typically than circle -> circle
                        effective_ratio *= 0.25;
                    }

                    if prev_island_size == island_size {
                        // * repeated island size (ex: triplet -> triplet)
                        effective_ratio *= 0.25;
                    }

                    if prev_island_size % 2 == island_size % 2 {
                        // * repeated island polarity (2 -> 4, 3 -> 5)
                        effective_ratio *= 0.5;
                    }

                    if last_delta > prev_delta + 10.0 && prev_delta > curr_delta + 10.0 {
                        // * previous increase happened a note ago, 1/1 -> 1/2-1/4, don't want to buff this
                        effective_ratio *= 0.125;
                    }

                    rhythm_complexity_sum += (effective_ratio * start_ratio).sqrt()
                        * curr_historical_decay
                        * ((4 + island_size) as f64).sqrt()
                        * ((4 + prev_island_size) as f64).sqrt()
                        / 4.0;

                    start_ratio = effective_ratio;
                    prev_island_size = island_size;
                    island_size = 1;

                    // * we're slowing down, stop counting
                    if prev_delta * 1.25 < curr_delta {
                        // * if we're speeding up, this stays true and we keep counting island size
                        first_delta_switch = false;
                    }
                }
            } else if prev_delta > 1.25 * curr_delta {
                // * we want to be speeding up
                // * begin counting island until we change speed again
                first_delta_switch = true;
                start_ratio = effective_ratio;
                island_size = 1;
            }
        }
    }

    // * produces multiplier that can be applied to strain. range [1, infinity) (not really though)
    (4.0 + rhythm_complexity_sum * SPEED_RHYTHM_MULTIPLIER).sqrt() / 2.0
}

fn calculate_wide_angle_bonus(angle: f64) -> f64 {
    let base = (3.0 / 4.0 * ((PI / 6.0).max(angle).min(5.0 / 6.0 * PI) - PI / 6.0)).sin();

    base * base
}

fn calculate_acute_angle_bonus(angle: f64) -> f64 {
    1.0 - calculate_wide_angle_bonus(angle)
}

impl fmt::Debug for SkillKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aim { .. } => f.debug_struct("Aim").finish(),
            Self::Flashlight { .. } => f.debug_struct("Flashlight").finish(),
            Self::Speed { .. } => f.debug_struct("Speed").finish(),
        }
    }
}
