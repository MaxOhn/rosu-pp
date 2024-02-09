use std::{
    cell::RefCell,
    cmp,
    rc::{Rc, Weak},
};

use crate::taiko::difficulty::object::TaikoDifficultyObject;

use super::alternating_mono_pattern::AlternatingMonoPattern;

const MAX_REPETITION_INTERVAL: usize = 16;

#[derive(Debug)]
pub struct RepeatingHitPatterns {
    pub alternating_mono_patterns: Vec<Rc<RefCell<AlternatingMonoPattern>>>,
    pub prev: Option<Weak<RefCell<Self>>>,
    pub repetition_interval: usize,
}

impl RepeatingHitPatterns {
    pub fn new(prev: Option<Weak<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            alternating_mono_patterns: Vec::new(),
            prev,
            repetition_interval: 0,
        }))
    }

    pub fn find_repetition_interval(&mut self) {
        let Some(mut other) = self.prev.as_ref().and_then(Weak::upgrade) else {
            return self.repetition_interval = MAX_REPETITION_INTERVAL + 1;
        };

        let mut interval = 1;

        while interval < MAX_REPETITION_INTERVAL {
            if self.is_repetition_of(&other.borrow()) {
                self.repetition_interval = cmp::min(interval, MAX_REPETITION_INTERVAL);

                return;
            }

            let Some(next) = other.borrow().prev.as_ref().and_then(Weak::upgrade) else {
                break;
            };

            other = next;
            interval += 1;
        }

        self.repetition_interval = MAX_REPETITION_INTERVAL + 1;
    }

    fn is_repetition_of(&self, other: &Self) -> bool {
        if self.alternating_mono_patterns.len() != other.alternating_mono_patterns.len() {
            return false;
        }

        self.alternating_mono_patterns
            .iter()
            .zip(other.alternating_mono_patterns.iter())
            .take(2)
            .all(|(self_pat, other_pat)| {
                self_pat
                    .borrow()
                    .has_identical_mono_len(&other_pat.borrow())
            })
    }

    pub fn first_hit_object(&self) -> Option<Rc<RefCell<TaikoDifficultyObject>>> {
        self.alternating_mono_patterns
            .first()
            .and_then(|mono| mono.borrow().first_hit_object())
    }
}
