use crate::{
    taiko::difficulty::object::{TaikoDifficultyObject, TaikoDifficultyObjects},
    util::sync::{RefCount, Weak},
};

use super::data::{
    alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
    repeating_hit_patterns::RepeatingHitPatterns,
};

#[derive(Debug, Default)]
pub struct ColorData {
    pub mono_streak: Option<Weak<MonoStreak>>,
    pub alternating_mono_pattern: Option<Weak<AlternatingMonoPattern>>,
    pub repeating_hit_patterns: Option<RefCount<RepeatingHitPatterns>>,
}

impl ColorData {
    pub fn previous_color_change<'a>(
        &self,
        hit_objects: &'a TaikoDifficultyObjects,
    ) -> Option<&'a RefCount<TaikoDifficultyObject>> {
        self.mono_streak
            .as_ref()
            .and_then(Weak::upgrade)
            .and_then(|mono| mono.get().first_hit_object())
            .and_then(|h| hit_objects.previous_note(&h.get(), 0))
    }

    pub fn next_color_change<'a>(
        &self,
        hit_objects: &'a TaikoDifficultyObjects,
    ) -> Option<&'a RefCount<TaikoDifficultyObject>> {
        self.mono_streak
            .as_ref()
            .and_then(Weak::upgrade)
            .and_then(|mono| mono.get().last_hit_object())
            .and_then(|h| hit_objects.next_note(&h.get(), 0))
    }
}
