use std::{error, fmt};

use rosu_map::{section::general::GameMode, util::Pos};

use crate::{
    model::hit_object::{HitObject, HitObjectKind},
    util::hint::unlikely,
};

use super::Beatmap;

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
    /// Notes are too dense time-wise.
    Density,
    /// The map seems too long.
    Length,
    /// Too many objects.
    ObjectCount,
    /// General red flag.
    RedFlag,
    /// Too many sliders' positions were suspicious.
    SliderPositions,
    /// Too many sliders had a very high amount of repeats.
    SliderRepeats,
}

impl TooSuspicious {
    pub(crate) fn new(map: &Beatmap) -> Option<Self> {
        #[inline]
        const fn too_long(hit_objects: &[HitObject]) -> bool {
            const DAY_MS: u32 = 60 * 60 * 24 * 1000;

            if unlikely(hit_objects.len() < 2) {
                return false;
            }

            let [first, .., last] = hit_objects else {
                unreachable!()
            };

            (last.start_time - first.start_time) > DAY_MS as f64
        }

        #[inline]
        fn too_many_objects(map: &Beatmap) -> bool {
            const THRESHOLD: usize = 500_000;
            /// Taiko calculation is especially expensive for high object counts
            const THRESHOLD_TAIKO: usize = 20_000;

            match map.mode {
                GameMode::Taiko => map.hit_objects.len() > THRESHOLD_TAIKO,
                _ => map.hit_objects.len() > THRESHOLD,
            }
        }

        #[inline]
        fn too_dense(i: usize, curr: &HitObject, map: &Beatmap) -> bool {
            const fn too_dense<const PER_1S: usize, const PER_10S: usize>(
                i: usize,
                curr: &HitObject,
                hit_objects: &[HitObject],
            ) -> bool {
                (hit_objects.len() > i + PER_1S
                    && hit_objects[i + PER_1S].start_time - curr.start_time < 1000.0)
                    || (hit_objects.len() > i + PER_10S
                        && hit_objects[i + PER_10S].start_time - curr.start_time < 10_000.0)
            }

            match map.mode {
                GameMode::Mania => {
                    // In mania it's more common to have a high note density
                    const THRESHOLD_1S: usize = 200; // 200 4K notes per 1s = 3000BPM
                    const THRESHOLD_10S: usize = 500; // 500 4K notes per 10s = 750BPM

                    too_dense::<THRESHOLD_1S, THRESHOLD_10S>(i, curr, &map.hit_objects)
                }
                _ => {
                    const THRESHOLD_1S: usize = 100; // 100 notes per 1s = 6000BPM
                    const THRESHOLD_10S: usize = 250; // 250 notes per 10s = 1500BPM

                    too_dense::<THRESHOLD_1S, THRESHOLD_10S>(i, curr, &map.hit_objects)
                }
            }
        }

        #[inline]
        const fn check_pos(pos: Pos) -> bool {
            /// osu!'s max value is `131_072` and the playfield is `512x384`
            const THRESHOLD: f32 = 10_000.0;

            f32::abs(pos.x) > THRESHOLD || f32::abs(pos.y) > THRESHOLD
        }

        #[inline]
        const fn check_repeats(repeats: usize) -> bool {
            /// osu!'s max value is `9000`
            const THRESHOLD: usize = 1000;

            repeats > THRESHOLD
        }

        if unlikely(too_many_objects(map)) {
            return Some(Self::ObjectCount);
        } else if unlikely(too_long(&map.hit_objects)) {
            return Some(Self::Length);
        }

        let mut pos_beyond_threshold = 0;
        let mut repeats_beyond_threshold = 0;

        for (i, h) in map.hit_objects.iter().enumerate() {
            if unlikely(too_dense(i, h, map)) {
                return Some(Self::Density);
            }

            if let HitObjectKind::Slider(ref slider) = h.kind {
                if unlikely(check_repeats(slider.repeats)) {
                    if unlikely(
                        check_pos(h.pos) && matches!(map.mode, GameMode::Osu | GameMode::Catch),
                    ) {
                        return Some(Self::RedFlag);
                    }

                    repeats_beyond_threshold += 1;
                } else if unlikely(check_pos(h.pos)) {
                    pos_beyond_threshold += 1;
                }
            }
        }

        if matches!(map.mode, GameMode::Taiko | GameMode::Mania) {
            // Taiko and Mania calculations aren't as susceptible to malicious
            // slider values
            None
        } else if unlikely(pos_beyond_threshold > 256) {
            Some(Self::SliderPositions)
        } else if unlikely(repeats_beyond_threshold > 256) {
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
            "the map seems too suspicious for further calculation (reason={self:?})",
        )
    }
}
