use crate::parse::HitObject;

use std::cmp::Ordering;

static COMMON_RHYTHMS: [HitObjectRhythm; 9] = [
    HitObjectRhythm {
        id: 0,
        ratio: 1.0,
        difficulty: 0.0,
    },
    HitObjectRhythm {
        id: 1,
        ratio: 2.0 / 1.0,
        difficulty: 0.3,
    },
    HitObjectRhythm {
        id: 2,
        ratio: 1.0 / 2.0,
        difficulty: 0.5,
    },
    HitObjectRhythm {
        id: 3,
        ratio: 3.0 / 1.0,
        difficulty: 0.3,
    },
    HitObjectRhythm {
        id: 4,
        ratio: 1.0 / 3.0,
        difficulty: 0.35,
    },
    HitObjectRhythm {
        id: 5,
        ratio: 3.0 / 2.0,
        difficulty: 0.6,
    },
    HitObjectRhythm {
        id: 6,
        ratio: 2.0 / 3.0,
        difficulty: 0.4,
    },
    HitObjectRhythm {
        id: 7,
        ratio: 5.0 / 4.0,
        difficulty: 0.5,
    },
    HitObjectRhythm {
        id: 8,
        ratio: 4.0 / 5.0,
        difficulty: 0.7,
    },
];

#[derive(Copy, Clone, Debug)]
pub(crate) struct HitObjectRhythm {
    id: u8,
    ratio: f32,
    pub(crate) difficulty: f32,
}

impl PartialEq for HitObjectRhythm {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
