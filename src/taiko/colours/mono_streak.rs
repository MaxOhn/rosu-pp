use std::{
    cell::RefCell,
    fmt::{Debug, Formatter, Result as FmtResult},
    rc::{Rc, Weak},
};

use crate::taiko::difficulty_object::{MonoIndex, TaikoDifficultyObject};

use super::{alternating_mono_pattern::AlternatingMonoPattern, HitKind};

pub(crate) struct MonoStreak {
    pub(crate) hit_objects: Vec<Weak<RefCell<TaikoDifficultyObject>>>,
    pub(crate) parent: Option<Weak<RefCell<AlternatingMonoPattern>>>,
    pub(crate) idx: usize,
}

impl Debug for MonoStreak {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "(idx={}, obj_len={}, has_parent={})",
            self.idx,
            self.hit_objects.len(),
            self.parent.is_some()
        )
    }
}

impl MonoStreak {
    pub(crate) fn new() -> Rc<RefCell<Self>> {
        let this = Self {
            hit_objects: Vec::new(),
            parent: None,
            idx: 0,
        };

        Rc::new(RefCell::new(this))
    }

    pub(crate) fn first_hit_object(&self) -> Option<Weak<RefCell<TaikoDifficultyObject>>> {
        self.hit_objects.first().map(Weak::clone)
    }

    pub(crate) fn hit_kind(&self) -> Option<HitKind> {
        self.hit_objects
            .first()
            .and_then(Weak::upgrade)
            .and_then(|obj| match obj.borrow().mono_idx {
                MonoIndex::Centre(_) => Some(HitKind::Centre),
                MonoIndex::Rim(_) => Some(HitKind::Rim),
                MonoIndex::None => None,
            })
    }

    pub(crate) fn run_len(&self) -> usize {
        self.hit_objects.len()
    }
}
