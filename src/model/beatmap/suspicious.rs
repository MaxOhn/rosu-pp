use std::{error, fmt};

use rosu_map::util::Pos;

use crate::{
    model::hit_object::{HitObject, HitObjectKind},
    util::hint::unlikely,
};

/// Resulting error type of [`Beatmap::check_suspicion`].
///
/// If you feel like a [`Beatmap`] is incorrectly flagged as suspicious or if
/// a map should be flagged but isn't, please open an issue so the heuristic
/// can be improved.
///
/// [`Beatmap::check_suspicion`]: crate::model::beatmap::Beatmap::check_suspicion
/// [`Beatmap`]: crate::model::beatmap::Beatmap
#[derive(Debug)]
#[non_exhaustive]
pub enum TooSuspicious {
    /// The map seems too long.
    Length,
    /// General red flag.
    RedFlag,
    /// Too many sliders' positions were suspicious.
    SliderPositions,
    /// Too many sliders had a very amount of repeats.
    SliderRepeats,
}

impl TooSuspicious {
    pub(crate) fn new(hit_objects: &[HitObject]) -> Option<Self> {
        #[inline]
        const fn too_long(hit_objects: &[HitObject]) -> bool {
            const DAY_IN_MS: u32 = 60 * 60 * 24 * 1000;

            if unlikely(hit_objects.len() < 2) {
                return false;
            }

            let [first, .., last] = hit_objects else {
                unreachable!()
            };

            unlikely((last.start_time - first.start_time) > DAY_IN_MS as f64)
        }

        if unlikely(too_long(hit_objects)) {
            return Some(Self::Length);
        }

        let tracker = ValueTracker::new(hit_objects);

        if unlikely(tracker.red_flag) {
            Some(Self::RedFlag)
        } else if unlikely(tracker.pos_beyond_threshold > 256) {
            Some(Self::SliderPositions)
        } else if unlikely(tracker.repeats_beyond_threshold > 256) {
            Some(Self::SliderRepeats)
        } else {
            None
        }
    }
}

impl error::Error for TooSuspicious {}

impl fmt::Display for TooSuspicious {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the hit objects seem too suspicious for further calculation \
            (reason={self:?})",
        )
    }
}

pub(crate) struct ValueTracker {
    pos_beyond_threshold: usize,
    repeats_beyond_threshold: usize,
    red_flag: bool,
}

impl ValueTracker {
    /// osu!'s max value is `131_072` and the playfield is `512x384`
    const POS_THRESHOLD: f32 = 10_000.0;

    /// osu!'s max value is `9000`
    const REPEATS_THRESHOLD: usize = 1000;

    fn new(hit_objects: &[HitObject]) -> Self {
        let mut this = Self {
            pos_beyond_threshold: 0,
            repeats_beyond_threshold: 0,
            red_flag: false,
        };

        for h in hit_objects {
            this.process(h);
        }

        this
    }

    const fn process(&mut self, h: &HitObject) {
        #[inline]
        const fn check_pos(pos: Pos) -> bool {
            unlikely(
                f32::abs(pos.x) > ValueTracker::POS_THRESHOLD
                    || f32::abs(pos.y) > ValueTracker::POS_THRESHOLD,
            )
        }

        #[inline]
        const fn check_repeats(repeats: usize) -> bool {
            unlikely(repeats > ValueTracker::REPEATS_THRESHOLD)
        }

        if let HitObjectKind::Slider(ref slider) = h.kind {
            if check_repeats(slider.repeats) {
                self.repeats_beyond_threshold += 1;

                if check_pos(h.pos) {
                    self.red_flag = true;
                }
            } else if check_pos(h.pos) {
                self.pos_beyond_threshold += 1;
            }
        }
    }
}
