use crate::HitObject;

use std::cmp::Ordering;

static COMMON_RHYTHMS: [HitObjectRhythm; 9] = [
    HitObjectRhythm {
        ratio: 1.0 / 1.0,
        difficulty: 0.0,
    },
    HitObjectRhythm {
        ratio: 2.0 / 1.0,
        difficulty: 0.3,
    },
    HitObjectRhythm {
        ratio: 1.0 / 2.0,
        difficulty: 0.5,
    },
    HitObjectRhythm {
        ratio: 3.0 / 1.0,
        difficulty: 0.3,
    },
    HitObjectRhythm {
        ratio: 1.0 / 3.0,
        difficulty: 0.35,
    },
    HitObjectRhythm {
        ratio: 3.0 / 2.0,
        difficulty: 0.6,
    },
    HitObjectRhythm {
        ratio: 2.0 / 3.0,
        difficulty: 0.4,
    },
    HitObjectRhythm {
        ratio: 5.0 / 4.0,
        difficulty: 0.5,
    },
    HitObjectRhythm {
        ratio: 4.0 / 5.0,
        difficulty: 0.7,
    },
];

#[derive(Copy, Clone, Debug)]
pub(crate) struct HitObjectRhythm {
    ratio: f32,
    pub(crate) difficulty: f32,
}

impl PartialEq for HitObjectRhythm {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.ratio - other.ratio).abs() < f32::EPSILON
            && (self.difficulty - other.difficulty).abs() < f32::EPSILON
    }
}

impl Eq for HitObjectRhythm {}

#[inline]
pub(crate) fn closest_rhythm(
    delta_time: f32,
    last: &HitObject,
    last_last: &HitObject,
    clock_rate: f32,
) -> &'static HitObjectRhythm {
    let prev_len = (last.start_time - last_last.start_time) / clock_rate;
    let ratio = delta_time / prev_len;

    COMMON_RHYTHMS
        .iter()
        .min_by(|r1, r2| {
            (r1.ratio - ratio)
                .abs()
                .partial_cmp(&(r2.ratio - ratio).abs())
                .unwrap_or(Ordering::Equal)
        })
        .unwrap()
}
