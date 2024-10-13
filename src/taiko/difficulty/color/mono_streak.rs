use crate::{
    taiko::{difficulty::object::TaikoDifficultyObject, object::HitType},
    util::sync::{RefCount, Weak},
};

use super::alternating_mono_pattern::AlternatingMonoPattern;

#[derive(Debug)]
pub struct MonoStreak {
    pub hit_objects: Vec<Weak<TaikoDifficultyObject>>,
    pub parent: Option<Weak<AlternatingMonoPattern>>,
    pub idx: usize,
}

impl MonoStreak {
    pub fn new() -> RefCount<Self> {
        RefCount::new(Self {
            hit_objects: Vec::new(),
            parent: None,
            idx: 0,
        })
    }

    pub fn run_len(&self) -> usize {
        self.hit_objects.len()
    }

    pub fn hit_type(&self) -> Option<HitType> {
        self.hit_objects
            .first()
            .and_then(Weak::upgrade)
            .map(|h| h.get().base_hit_type)
    }

    pub fn first_hit_object(&self) -> Option<RefCount<TaikoDifficultyObject>> {
        self.hit_objects.first().and_then(Weak::upgrade)
    }

    pub fn last_hit_object(&self) -> Option<RefCount<TaikoDifficultyObject>> {
        self.hit_objects.last().and_then(Weak::upgrade)
    }
}
