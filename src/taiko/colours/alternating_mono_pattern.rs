use std::{
    cell::RefCell,
    fmt::{Debug, Formatter, Result as FmtResult},
    rc::{Rc, Weak},
};

use crate::taiko::difficulty_object::TaikoDifficultyObject;

use super::{mono_streak::MonoStreak, repeating_hit_patterns::RepeatingHitPatterns};

pub(crate) struct AlternatingMonoPattern {
    pub(crate) mono_streaks: Vec<Rc<RefCell<MonoStreak>>>,
    pub(crate) parent: Option<Weak<RefCell<RepeatingHitPatterns>>>,
    pub(crate) idx: usize,
}

impl Debug for AlternatingMonoPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "(idx={}, mono_len={}, has_parent={})",
            self.idx,
            self.mono_streaks.len(),
            self.parent.is_some()
        )
    }
}

impl AlternatingMonoPattern {
    pub(crate) fn new() -> Rc<RefCell<Self>> {
        let this = Self {
            mono_streaks: Vec::new(),
            parent: None,
            idx: 0,
        };

        Rc::new(RefCell::new(this))
    }

    pub(crate) fn first_hit_object(&self) -> Option<Weak<RefCell<TaikoDifficultyObject>>> {
        self.mono_streaks
            .first()
            .and_then(|streak| streak.borrow().first_hit_object())
    }

    pub(crate) fn is_repetition_of(&self, other: &Self) -> bool {
        self.has_identical_mono_len(other)
            && other.mono_streaks.len() == self.mono_streaks.len()
            && other
                .mono_streaks
                .first()
                .map(|streak| streak.borrow().hit_kind())
                == self
                    .mono_streaks
                    .first()
                    .map(|streak| streak.borrow().hit_kind())
    }

    pub(crate) fn has_identical_mono_len(&self, other: &Self) -> bool {
        let other_len = other
            .mono_streaks
            .first()
            .map(|streak| streak.borrow().run_len());

        let self_len = self
            .mono_streaks
            .first()
            .map(|streak| streak.borrow().run_len());

        other_len == self_len
    }
}
