use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub(crate) use self::{
    alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
    preprocessor::ColourDifficultyPreprocessor, repeating_hit_patterns::RepeatingHitPatterns,
};

mod alternating_mono_pattern;
mod mono_streak;
mod preprocessor;
mod repeating_hit_patterns;

#[derive(Clone, Debug, Default)]
pub(crate) struct TaikoDifficultyColour {
    pub(crate) mono_streak: Option<Weak<RefCell<MonoStreak>>>,
    pub(crate) alternating_mono_pattern: Option<Weak<RefCell<AlternatingMonoPattern>>>,
    pub(crate) repeating_hit_patterns: Option<Rc<RefCell<RepeatingHitPatterns>>>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum HitKind {
    Centre,
    Rim,
}
