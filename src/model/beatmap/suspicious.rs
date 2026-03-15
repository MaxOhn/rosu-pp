use std::{cmp, error, fmt, ops::ControlFlow};

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
        const fn too_many_objects(map: &Beatmap) -> bool {
            const THRESHOLD: usize = 500_000;
            /// Taiko calculation is especially expensive for high object counts
            // Note that the ranked map /b/3824509 has 28866 objects.
            const THRESHOLD_TAIKO: usize = 30_000;

            match map.mode {
                GameMode::Taiko => map.hit_objects.len() > THRESHOLD_TAIKO,
                _ => map.hit_objects.len() > THRESHOLD,
            }
        }

        if unlikely(too_many_objects(map)) {
            return Some(Self::ObjectCount);
        } else if unlikely(too_long(&map.hit_objects)) {
            return Some(Self::Length);
        }

        match map.mode {
            GameMode::Osu => Self::check_osu(map),
            GameMode::Taiko => Self::check_taiko(map),
            GameMode::Catch => Self::check_catch(map),
            GameMode::Mania => Self::check_mania(map),
        }
    }

    fn check_osu(map: &Beatmap) -> Option<Self> {
        let mut state = SliderState::new();
        let per_1s = THRESHOLD_1S;
        let per_10s = THRESHOLD_10S;

        // Checking both note density and sliders
        for (i, h) in map.hit_objects.iter().enumerate() {
            if unlikely(Self::too_dense(&map.hit_objects, i, per_1s, per_10s)) {
                return Some(Self::Density);
            } else if unlikely(Self::suspicious_slider(h, &mut state).is_break()) {
                return Some(Self::RedFlag);
            }
        }

        state.eval()
    }

    fn check_taiko(map: &Beatmap) -> Option<Self> {
        let per_1s = THRESHOLD_1S * 2;
        let per_10s = THRESHOLD_10S * 2;

        // Only checking note density
        for i in 0..map.hit_objects.len() {
            if unlikely(Self::too_dense(&map.hit_objects, i, per_1s, per_10s)) {
                return Some(Self::Density);
            }
        }

        None
    }

    fn check_catch(map: &Beatmap) -> Option<Self> {
        let mut state = SliderState::new();

        // Only checking sliders
        for h in map.hit_objects.iter() {
            if unlikely(Self::suspicious_slider(h, &mut state).is_break()) {
                return Some(Self::RedFlag);
            }
        }

        state.eval()
    }

    fn check_mania(map: &Beatmap) -> Option<Self> {
        let keys_per_hand = cmp::max(1, map.cs as usize / 2);
        let per_1s = THRESHOLD_1S * keys_per_hand;
        let per_10s = THRESHOLD_10S * keys_per_hand;

        // Only checking note density
        for i in 0..map.hit_objects.len() {
            if unlikely(Self::too_dense(&map.hit_objects, i, per_1s, per_10s)) {
                return Some(Self::Density);
            }
        }

        None
    }

    #[inline]
    fn too_dense(hit_objects: &[HitObject], i: usize, per_1s: usize, per_10s: usize) -> bool {
        (hit_objects.len() > i + per_1s
            && hit_objects[i + per_1s].start_time - hit_objects[i].start_time < 1000.0)
            || (hit_objects.len() > i + per_10s
                && hit_objects[i + per_10s].start_time - hit_objects[i].start_time < 10_000.0)
    }

    #[inline]
    const fn suspicious_slider(h: &HitObject, state: &mut SliderState) -> ControlFlow<()> {
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

        if let HitObjectKind::Slider(ref slider) = h.kind {
            if unlikely(check_repeats(slider.repeats)) {
                if unlikely(check_pos(h.pos)) {
                    return ControlFlow::Break(());
                }

                state.repeats_beyond_threshold += 1;
            } else if unlikely(check_pos(h.pos)) {
                state.pos_beyond_threshold += 1;
            }
        }

        ControlFlow::Continue(())
    }
}

/// 200 notes per 1s = 12000 BPM
const THRESHOLD_1S: usize = 200;

/// 500 notes per 10s = 3000 BPM
const THRESHOLD_10S: usize = 500;

struct SliderState {
    repeats_beyond_threshold: usize,
    pos_beyond_threshold: usize,
}

impl SliderState {
    const fn new() -> Self {
        Self {
            repeats_beyond_threshold: 0,
            pos_beyond_threshold: 0,
        }
    }

    const fn eval(self) -> Option<TooSuspicious> {
        const CUTOFF: usize = 128;

        if unlikely(self.pos_beyond_threshold > CUTOFF) {
            Some(TooSuspicious::SliderPositions)
        } else if unlikely(self.repeats_beyond_threshold > CUTOFF) {
            Some(TooSuspicious::SliderRepeats)
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

/*
    Noteworthy loved maps:
    [1175457, 1277504, 1594580, 1904970, 2140631, 2440314, 2573161, 2571051,
    2573164, 2619200, 2923535, 3824509]
*/
