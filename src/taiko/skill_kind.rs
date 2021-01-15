use super::{DifficultyObject, HitObjectRhythm, LimitedQueue, Rim};

const RHYTHM_STRAIN_DECAY: f32 = 0.96;
const MOST_RECENT_PATTERNS_TO_COMPARE: usize = 2;

const MONO_HISTORY_MAX_LEN: usize = 5;
const RHYTHM_HISTORY_MAX_LEN: usize = 8;
const STAMINA_HISTORY_MAX_LEN: usize = 2;

pub(crate) enum SkillKind {
    Color {
        mono_history: LimitedQueue<usize>,
        prev_is_rim: Option<bool>,
        current_mono_len: usize,
    },
    Rhythm {
        rhythm_history: LimitedQueue<(usize, HitObjectRhythm)>, // (idx, rhythm)
        notes_since_rhythm_change: usize,
        current_strain: f32,
    },
    Stamina {
        note_pair_duration_history: LimitedQueue<f32>,
        hand: u8,
        off_hand_object_duration: f32,
    },
}

impl SkillKind {
    #[inline]
    pub(crate) fn color() -> Self {
        Self::Color {
            mono_history: LimitedQueue::new(MONO_HISTORY_MAX_LEN),
            prev_is_rim: None,
            current_mono_len: 0,
        }
    }

    #[inline]
    pub(crate) fn rhythm() -> Self {
        Self::Rhythm {
            rhythm_history: LimitedQueue::new(RHYTHM_HISTORY_MAX_LEN),
            notes_since_rhythm_change: 0,
            current_strain: 0.0,
        }
    }

    #[inline]
    pub(crate) fn stamina(right_hand: bool) -> Self {
        Self::Stamina {
            note_pair_duration_history: LimitedQueue::new(STAMINA_HISTORY_MAX_LEN),
            hand: right_hand as u8,
            off_hand_object_duration: f32::MAX,
        }
    }

    pub(crate) fn strain_value_of(&mut self, current: &DifficultyObject, cheese: &[bool]) -> f32 {
        match self {
            Self::Color {
                mono_history,
                prev_is_rim,
                current_mono_len,
            } => {
                let prev_is_circle = current.prev.is_circle();
                let base_is_circle = current.base.is_circle();
                let curr_is_rim = current.base.is_rim();

                if !(current.delta < 1000.0 && prev_is_circle && base_is_circle) {
                    mono_history.clear();
                    *current_mono_len = base_is_circle as usize;

                    *prev_is_rim = if base_is_circle {
                        Some(curr_is_rim)
                    } else {
                        None
                    };

                    return 0.0;
                }

                let mut strain = 0.0;

                if prev_is_rim
                    .filter(|&is_rim| is_rim != curr_is_rim)
                    .is_some()
                {
                    strain = if mono_history.len() < 2
                        || (*mono_history.last().unwrap() + *current_mono_len) % 2 == 0
                    {
                        0.0
                    } else {
                        1.0
                    };

                    let mut reps_penalty = 1.0;

                    mono_history.push(*current_mono_len);

                    let iter = (0..mono_history
                        .len()
                        .saturating_sub(MOST_RECENT_PATTERNS_TO_COMPARE))
                        .rev();

                    for start in iter {
                        let different_pattern = (0..MOST_RECENT_PATTERNS_TO_COMPARE).any(|i| {
                            mono_history[start + i]
                                != mono_history
                                    [mono_history.len() + i - MOST_RECENT_PATTERNS_TO_COMPARE]
                        });

                        if different_pattern {
                            continue;
                        }

                        let mut notes_since = 0;

                        for i in start..mono_history.len() {
                            notes_since += mono_history[i];
                        }

                        reps_penalty *= repetition_penalty(notes_since);

                        break;
                    }

                    strain *= reps_penalty;

                    *current_mono_len = 1;
                } else {
                    *current_mono_len += 1;
                }

                *prev_is_rim = Some(curr_is_rim);

                strain
            }
            Self::Rhythm {
                rhythm_history,
                notes_since_rhythm_change,
                current_strain,
            } => {
                let base_is_circle = current.base.is_circle();

                if !base_is_circle {
                    *current_strain = 0.0;
                    *notes_since_rhythm_change = 0;

                    return 0.0;
                }

                *current_strain *= RHYTHM_STRAIN_DECAY;
                *notes_since_rhythm_change += 1;

                if current.rhythm.difficulty.abs() < f32::EPSILON {
                    return 0.0;
                }

                let mut strain = current.rhythm.difficulty;

                rhythm_history.push((current.idx, *current.rhythm));

                let mut reps_penalty = 1.0;

                for most_recent_patterns_to_compare in 2..=RHYTHM_HISTORY_MAX_LEN / 2 {
                    let iter = (0..rhythm_history
                        .len()
                        .saturating_sub(most_recent_patterns_to_compare))
                        .rev();

                    for start in iter {
                        let different_pattern = (0..most_recent_patterns_to_compare).any(|i| {
                            rhythm_history[start + i].1
                                != rhythm_history
                                    [rhythm_history.len() + i - most_recent_patterns_to_compare]
                                    .1
                        });

                        if different_pattern {
                            continue;
                        }

                        reps_penalty *= repetition_penalty(current.idx - rhythm_history[start].0);

                        break;
                    }
                }

                let speed_penalty = if current.delta < 80.0 {
                    1.0
                } else if current.delta < 210.0 {
                    (1.4 - 0.005 * current.delta).max(0.0)
                } else {
                    *current_strain = 0.0;
                    *notes_since_rhythm_change = 0;

                    0.0
                };

                strain *= reps_penalty;
                strain *= pattern_len_penalty(*notes_since_rhythm_change);
                strain *= speed_penalty;

                *notes_since_rhythm_change = 0;
                *current_strain += strain;

                *current_strain
            }
            Self::Stamina {
                hand,
                note_pair_duration_history,
                off_hand_object_duration,
            } => {
                let base_is_circle = current.base.is_circle();

                if !base_is_circle {
                    return 0.0;
                }

                if current.idx % 2 == *hand as usize {
                    if current.idx == 1 {
                        return 1.0;
                    }

                    let mut strain = 1.0;
                    note_pair_duration_history.push(current.delta + *off_hand_object_duration);
                    let shortest_recent_note = *note_pair_duration_history.min().unwrap();
                    strain += speed_bonus(shortest_recent_note);

                    if cheese[current.idx] {
                        let p = cheese_penalty(current.delta + *off_hand_object_duration);
                        strain *= p;
                    }

                    return strain;
                }

                *off_hand_object_duration = current.delta;

                0.0
            }
        }
    }
}

#[inline]
fn pattern_len_penalty(pattern_len: usize) -> f32 {
    let short_pattern_penalty = (0.15 * pattern_len as f32).min(1.0);
    let long_pattern_penalty = (2.5 - 0.15 * pattern_len as f32).max(0.0).min(1.0);

    short_pattern_penalty.min(long_pattern_penalty)
}

#[inline]
fn cheese_penalty(note_pair_duration: f32) -> f32 {
    if note_pair_duration > 125.0 {
        1.0
    } else if note_pair_duration < 100.0 {
        0.6
    } else {
        0.6 + (note_pair_duration - 100.0) * 0.016
    }
}

#[inline]
fn speed_bonus(note_pair_duration: f32) -> f32 {
    if note_pair_duration > 200.0 {
        return 0.0;
    }

    let mut bonus = 200.0 - note_pair_duration;
    bonus *= bonus;

    bonus / 100_000.0
}

#[inline]
fn repetition_penalty(notes_since: usize) -> f32 {
    (0.032 * notes_since as f32).min(1.0)
}
