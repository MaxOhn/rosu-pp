use std::slice::Iter;

use crate::{
    any::difficulty::object::IDifficultyObject,
    taiko::object::{HitType, TaikoObject},
    util::sync::RefCount,
};

use super::{color::TaikoDifficultyColor, rhythm::HitObjectRhythm};

#[derive(Debug)]
pub struct TaikoDifficultyObject {
    pub idx: usize,
    pub delta_time: f64,
    pub start_time: f64,
    pub base_hit_type: HitType,
    pub mono_idx: MonoIndex,
    pub note_idx: usize,
    pub rhythm: &'static HitObjectRhythm,
    pub color: TaikoDifficultyColor,
}

impl TaikoDifficultyObject {
    pub fn new(
        hit_object: &TaikoObject,
        last_object: &TaikoObject,
        last_last_object: &TaikoObject,
        clock_rate: f64,
        idx: usize,
        objects: &mut TaikoDifficultyObjects,
    ) -> RefCount<Self> {
        let delta_time = (hit_object.start_time - last_object.start_time) / clock_rate;
        let rhythm = closest_rhythm(delta_time, last_object, last_last_object, clock_rate);
        let color = TaikoDifficultyColor::default();
        let mut note_idx = 0;

        let mono_idx = match hit_object.hit_type {
            HitType::Center => {
                note_idx = objects.note_objects.len();

                MonoIndex::Center(objects.center_hit_objects.len())
            }
            HitType::Rim => {
                note_idx = objects.note_objects.len();

                MonoIndex::Rim(objects.rim_hit_objects.len())
            }
            HitType::NonHit => MonoIndex::None,
        };

        let this = RefCount::new(Self {
            idx,
            delta_time,
            start_time: hit_object.start_time / clock_rate,
            base_hit_type: hit_object.hit_type,
            mono_idx,
            note_idx,
            rhythm,
            color,
        });

        match hit_object.hit_type {
            HitType::Center => {
                objects.note_objects.push(RefCount::clone(&this));
                objects.center_hit_objects.push(RefCount::clone(&this));
            }
            HitType::Rim => {
                objects.note_objects.push(RefCount::clone(&this));
                objects.rim_hit_objects.push(RefCount::clone(&this));
            }
            HitType::NonHit => {}
        }

        this
    }
}

#[derive(Debug)]
pub enum MonoIndex {
    Center(usize),
    Rim(usize),
    None,
}

pub struct TaikoDifficultyObjects {
    pub objects: Vec<RefCount<TaikoDifficultyObject>>,
    pub center_hit_objects: Vec<RefCount<TaikoDifficultyObject>>,
    pub rim_hit_objects: Vec<RefCount<TaikoDifficultyObject>>,
    pub note_objects: Vec<RefCount<TaikoDifficultyObject>>,
}

impl TaikoDifficultyObjects {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            objects: Vec::with_capacity(capacity),
            // mean=301.7 | median=215
            center_hit_objects: Vec::with_capacity(256),
            // mean=309.21 | median=229
            rim_hit_objects: Vec::with_capacity(256),
            // mean=610.91 | median=466
            note_objects: Vec::with_capacity(256),
        }
    }

    pub fn push(&mut self, hit_object: RefCount<TaikoDifficultyObject>) {
        self.objects.push(hit_object);
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, RefCount<TaikoDifficultyObject>> {
        self.objects.iter()
    }

    pub fn previous_mono(
        &self,
        curr: &TaikoDifficultyObject,
        mut backwards_idx: usize,
    ) -> Option<&RefCount<TaikoDifficultyObject>> {
        backwards_idx += 1;

        match curr.mono_idx {
            MonoIndex::Center(idx) => idx
                .checked_sub(backwards_idx)
                .and_then(|idx| self.center_hit_objects.get(idx)),
            MonoIndex::Rim(idx) => idx
                .checked_sub(backwards_idx)
                .and_then(|idx| self.rim_hit_objects.get(idx)),
            MonoIndex::None => None,
        }
    }

    pub fn previous_note<'a>(
        &'a self,
        curr: &TaikoDifficultyObject,
        backwards_idx: usize,
    ) -> Option<&'a RefCount<TaikoDifficultyObject>> {
        curr.note_idx
            .checked_sub(backwards_idx + 1)
            .and_then(|idx| self.note_objects.get(idx))
    }

    pub fn next_note<'a>(
        &'a self,
        curr: &TaikoDifficultyObject,
        forwards_idx: usize,
    ) -> Option<&'a RefCount<TaikoDifficultyObject>> {
        self.note_objects.get(curr.note_idx + (forwards_idx + 1))
    }
}

#[rustfmt::skip]
pub static COMMON_RHYTHMS: [HitObjectRhythm; 9] = [
    HitObjectRhythm { id: 0, ratio: 1.0, difficulty: 0.0 },
    HitObjectRhythm { id: 1, ratio: 2.0 / 1.0, difficulty: 0.3 },
    HitObjectRhythm { id: 2, ratio: 1.0 / 2.0, difficulty: 0.5 },
    HitObjectRhythm { id: 3, ratio: 3.0 / 1.0, difficulty: 0.3 },
    HitObjectRhythm { id: 4, ratio: 1.0 / 3.0, difficulty: 0.35 },
    HitObjectRhythm { id: 5, ratio: 3.0 / 2.0, difficulty: 0.6 },
    HitObjectRhythm { id: 6, ratio: 2.0 / 3.0, difficulty: 0.4 },
    HitObjectRhythm { id: 7, ratio: 5.0 / 4.0, difficulty: 0.5 },
    HitObjectRhythm { id: 8, ratio: 4.0 / 5.0, difficulty: 0.7 },
];

fn closest_rhythm(
    delta_time: f64,
    last_object: &TaikoObject,
    last_last_object: &TaikoObject,
    clock_rate: f64,
) -> &'static HitObjectRhythm {
    let prev_len = (last_object.start_time - last_last_object.start_time) / clock_rate;
    let ratio = delta_time / prev_len;

    COMMON_RHYTHMS
        .iter()
        .min_by(|r1, r2| {
            (r1.ratio - ratio)
                .abs()
                .total_cmp(&(r2.ratio - ratio).abs())
        })
        .unwrap()
}

impl IDifficultyObject for TaikoDifficultyObject {
    fn idx(&self) -> usize {
        self.idx
    }
}

impl PartialEq for TaikoDifficultyObject {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}
