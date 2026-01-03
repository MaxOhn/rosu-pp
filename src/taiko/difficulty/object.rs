use std::{
    hash::{Hash, Hasher},
    slice::Iter,
};

use crate::{
    Beatmap,
    any::difficulty::object::{HasStartTime, IDifficultyObject, IDifficultyObjects},
    model::control_point::{EffectPoint, TimingPoint},
    taiko::{
        difficulty::utils::has_interval::HasInterval,
        object::{HitType, TaikoObject},
    },
    util::sync::RefCount,
};

use super::{color::color_data::ColorData, rhythm::rhythm_data::RhythmData};

#[derive(Debug)]
pub struct TaikoDifficultyObject {
    pub idx: usize,
    pub delta_time: f64,
    pub start_time: f64,
    pub base_hit_type: HitType,
    pub mono_idx: MonoIndex,
    pub note_idx: usize,
    pub rhythm_data: RhythmData,
    pub color_data: ColorData,
    pub effective_bpm: f64,
}

impl TaikoDifficultyObject {
    pub fn new(
        hit_object: &TaikoObject,
        last_object: &TaikoObject,
        clock_rate: f64,
        idx: usize,
        map: &Beatmap,
        global_slider_velocity: f64,
        objects: &mut TaikoDifficultyObjects,
    ) -> RefCount<Self> {
        let delta_time = (hit_object.start_time - last_object.start_time) / clock_rate;

        let prev_delta_time = idx
            .checked_sub(1)
            .map(|i| objects.objects[i].get().delta_time);

        let color_data = ColorData::default();
        let rhythm_data = RhythmData::new(delta_time, prev_delta_time);

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

        let start_time = hit_object.start_time / clock_rate;

        // * Using `hitObject.StartTime` causes floating point error differences
        let normalized_start_time = start_time * clock_rate;

        // * Retrieve the timing point at the note's start time
        let curr_control_point_bpm = map
            .timing_point_at(normalized_start_time)
            .map_or(TimingPoint::DEFAULT_BPM, TimingPoint::bpm);

        // * Calculate the slider velocity at the note's start time.
        let curr_slider_velocity = calculate_slider_velocity(
            map,
            normalized_start_time,
            clock_rate,
            global_slider_velocity,
        );

        let effective_bpm = curr_control_point_bpm * curr_slider_velocity;

        let this = RefCount::new(Self {
            idx,
            delta_time,
            start_time,
            base_hit_type: hit_object.hit_type,
            mono_idx,
            note_idx,
            rhythm_data,
            color_data,
            effective_bpm,
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

fn calculate_slider_velocity(
    map: &Beatmap,
    start_time: f64,
    clock_rate: f64,
    global_slider_velocity: f64,
) -> f64 {
    let active_effect_control_point_scroll_speed = map
        .effect_point_at(start_time)
        .map_or(EffectPoint::DEFAULT_SCROLL_SPEED, |effect_point| {
            effect_point.scroll_speed
        });

    global_slider_velocity * active_effect_control_point_scroll_speed * clock_rate
}

impl HasInterval for TaikoDifficultyObject {
    fn interval(&self) -> f64 {
        self.delta_time
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

    pub const fn is_empty(&self) -> bool {
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

impl IDifficultyObjects for TaikoDifficultyObjects {
    type DifficultyObject = RefCount<TaikoDifficultyObject>;

    fn get(&self, idx: usize) -> Option<&Self::DifficultyObject> {
        self.objects.get(idx)
    }
}

impl IDifficultyObject for TaikoDifficultyObject {
    type DifficultyObjects = TaikoDifficultyObjects;

    fn idx(&self) -> usize {
        self.idx
    }
}

impl HasStartTime for RefCount<TaikoDifficultyObject> {
    fn start_time(&self) -> f64 {
        self.get().start_time
    }
}

impl PartialEq for TaikoDifficultyObject {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Eq for TaikoDifficultyObject {}

impl Hash for TaikoDifficultyObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}
