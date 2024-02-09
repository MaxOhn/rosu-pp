use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::taiko::{difficulty::object::TaikoDifficultyObject, object::HitType};

use super::alternating_mono_pattern::AlternatingMonoPattern;

#[derive(Debug)]
pub struct MonoStreak {
    pub hit_objects: Vec<Weak<RefCell<TaikoDifficultyObject>>>,
    pub parent: Option<Weak<RefCell<AlternatingMonoPattern>>>,
    pub idx: usize,
}

impl MonoStreak {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            hit_objects: Vec::new(),
            parent: None,
            idx: 0,
        }))
    }

    pub fn run_len(&self) -> usize {
        self.hit_objects.len()
    }

    pub fn hit_type(&self) -> Option<HitType> {
        self.hit_objects
            .first()
            .and_then(Weak::upgrade)
            .map(|h| h.borrow().base_hit_type)
    }

    pub fn first_hit_object(&self) -> Option<Rc<RefCell<TaikoDifficultyObject>>> {
        self.hit_objects.first().and_then(Weak::upgrade)
    }
}
