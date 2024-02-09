use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::taiko::difficulty::object::TaikoDifficultyObject;

use super::{mono_streak::MonoStreak, repeating_hit_patterns::RepeatingHitPatterns};

#[derive(Debug)]
pub struct AlternatingMonoPattern {
    pub mono_streaks: Vec<Rc<RefCell<MonoStreak>>>,
    pub parent: Option<Weak<RefCell<RepeatingHitPatterns>>>,
    pub idx: usize,
}

impl AlternatingMonoPattern {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            mono_streaks: Vec::new(),
            parent: None,
            idx: 0,
        }))
    }

    pub fn is_repetition_of(&self, other: &Self) -> bool {
        self.has_identical_mono_len(other)
            && self.mono_streaks.len() == other.mono_streaks.len()
            && self.mono_streaks[0].borrow().hit_type() == other.mono_streaks[0].borrow().hit_type()
    }

    pub fn has_identical_mono_len(&self, other: &Self) -> bool {
        self.mono_streaks[0].borrow().run_len() == other.mono_streaks[0].borrow().run_len()
    }

    pub fn first_hit_object(&self) -> Option<Rc<RefCell<TaikoDifficultyObject>>> {
        self.mono_streaks
            .first()
            .and_then(|mono| mono.borrow().first_hit_object())
    }
}
