use crate::{
    taiko::difficulty::object::TaikoDifficultyObject,
    util::sync::{RefCount, Weak},
};

use super::{mono_streak::MonoStreak, repeating_hit_patterns::RepeatingHitPatterns};

#[derive(Debug)]
pub struct AlternatingMonoPattern {
    pub mono_streaks: Vec<RefCount<MonoStreak>>,
    pub parent: Option<Weak<RepeatingHitPatterns>>,
    pub idx: usize,
}

impl AlternatingMonoPattern {
    pub fn new() -> RefCount<Self> {
        RefCount::new(Self {
            mono_streaks: Vec::new(),
            parent: None,
            idx: 0,
        })
    }

    pub fn is_repetition_of(&self, other: &Self) -> bool {
        self.has_identical_mono_len(other)
            && self.mono_streaks.len() == other.mono_streaks.len()
            && self.mono_streaks[0].get().hit_type() == other.mono_streaks[0].get().hit_type()
    }

    pub fn has_identical_mono_len(&self, other: &Self) -> bool {
        self.mono_streaks[0].get().run_len() == other.mono_streaks[0].get().run_len()
    }

    pub fn first_hit_object(&self) -> Option<RefCount<TaikoDifficultyObject>> {
        self.mono_streaks
            .first()
            .and_then(|mono| mono.get().first_hit_object())
    }
}
