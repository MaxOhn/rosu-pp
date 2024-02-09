use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

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
    pub mono_streak: Option<Weak<RefCell<MonoStreak>>>,
    pub alternating_mono_pattern: Option<Weak<RefCell<AlternatingMonoPattern>>>,
    pub repeating_hit_patterns: Option<Rc<RefCell<RepeatingHitPatterns>>>,
}
