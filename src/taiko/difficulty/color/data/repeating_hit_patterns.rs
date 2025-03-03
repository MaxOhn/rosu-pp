use std::cmp;

use crate::{
    taiko::difficulty::object::TaikoDifficultyObject,
    util::sync::{RefCount, Weak},
};

use super::alternating_mono_pattern::AlternatingMonoPattern;

const MAX_REPETITION_INTERVAL: usize = 16;

#[derive(Debug)]
pub struct RepeatingHitPatterns {
    pub alternating_mono_patterns: Vec<RefCount<AlternatingMonoPattern>>,
    pub prev: Option<Weak<Self>>,
    pub repetition_interval: usize,
}

impl RepeatingHitPatterns {
    pub fn new(prev: Option<Weak<Self>>) -> RefCount<Self> {
        RefCount::new(Self {
            alternating_mono_patterns: Vec::new(),
            prev,
            repetition_interval: 0,
        })
    }

    pub fn find_repetition_interval(&mut self) {
        let Some(mut other) = self.prev.as_ref().and_then(Weak::upgrade) else {
            return self.repetition_interval = MAX_REPETITION_INTERVAL + 1;
        };

        let mut interval = 1;

        while interval < MAX_REPETITION_INTERVAL {
            if self.is_repetition_of(&other.get()) {
                self.repetition_interval = cmp::min(interval, MAX_REPETITION_INTERVAL);

                return;
            }

            let Some(next) = other.get().prev.as_ref().and_then(Weak::upgrade) else {
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
            .all(|(self_pat, other_pat)| self_pat.get().has_identical_mono_len(&other_pat.get()))
    }

    pub fn first_hit_object(&self) -> Option<RefCount<TaikoDifficultyObject>> {
        self.alternating_mono_patterns
            .first()
            .and_then(|mono| mono.get().first_hit_object())
    }
}
