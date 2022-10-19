use std::{
    cell::RefCell,
    fmt::{Debug, Formatter, Result as FmtResult},
    rc::{Rc, Weak},
};

use crate::taiko::difficulty_object::TaikoDifficultyObject;

use super::alternating_mono_pattern::AlternatingMonoPattern;

pub(crate) struct RepeatingHitPatterns {
    pub(crate) alternating_mono_patterns: Vec<Rc<RefCell<AlternatingMonoPattern>>>,
    pub(crate) prev: Option<Weak<RefCell<Self>>>,
    pub(crate) repetition_interval: usize,
}

impl Debug for RepeatingHitPatterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "(interval={}, alt_len={}, has_prev={})",
            self.repetition_interval,
            self.alternating_mono_patterns.len(),
            self.prev.is_some()
        )
    }
}

impl RepeatingHitPatterns {
    const MAX_REPETITION_INTERVAL: usize = 16;

    pub(crate) fn new(prev: Option<Weak<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        let this = Self {
            alternating_mono_patterns: Vec::new(),
            prev,
            repetition_interval: 0,
        };

        Rc::new(RefCell::new(this))
    }

    pub(crate) fn first_hit_object(&self) -> Option<Weak<RefCell<TaikoDifficultyObject>>> {
        self.alternating_mono_patterns
            .first()
            .and_then(|pattern| pattern.borrow().first_hit_object())
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

    pub(crate) fn find_repetition_interval(&mut self) {
        let mut other = match self.prev.as_ref().and_then(Weak::upgrade) {
            Some(prev) => prev,
            None => return self.repetition_interval = Self::MAX_REPETITION_INTERVAL + 1,
        };

        let mut interval = 1;

        while interval < Self::MAX_REPETITION_INTERVAL {
            if self.is_repetition_of(&other.borrow()) {
                return self.repetition_interval = interval.min(Self::MAX_REPETITION_INTERVAL);
            }

            let next = match other.borrow().prev.as_ref().and_then(Weak::upgrade) {
                Some(prev) => prev,
                None => break,
            };

            // gotta love NLL...
            other = next;

            interval += 1;
        }

        self.repetition_interval = Self::MAX_REPETITION_INTERVAL + 1;
    }
}
