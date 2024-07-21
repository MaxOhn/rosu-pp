use crate::util::sync::{RefCount, Weak};

use self::{
    alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
    repeating_hit_patterns::RepeatingHitPatterns,
};

pub mod alternating_mono_pattern;
pub mod mono_streak;
pub mod preprocessor;
pub mod repeating_hit_patterns;

#[derive(Debug, Default)]
pub struct TaikoDifficultyColor {
    pub mono_streak: Option<Weak<MonoStreak>>,
    pub alternating_mono_pattern: Option<Weak<AlternatingMonoPattern>>,
    pub repeating_hit_patterns: Option<RefCount<RepeatingHitPatterns>>,
}
