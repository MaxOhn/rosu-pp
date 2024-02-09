use std::{
    cell::{Ref, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use crate::taiko::difficulty::object::TaikoDifficultyObjects;

use super::{
    alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
    repeating_hit_patterns::RepeatingHitPatterns,
};

pub struct ColorDifficultyPreprocessor;

impl ColorDifficultyPreprocessor {
    pub fn process_and_assign(hit_objects: &TaikoDifficultyObjects) {
        let hit_patterns = Self::encode(hit_objects);

        for repeating_hit_pattern in hit_patterns {
            if let Some(obj) = repeating_hit_pattern.borrow().first_hit_object() {
                obj.borrow_mut().color.repeating_hit_patterns =
                    Some(Rc::clone(&repeating_hit_pattern));
            }

            let mono_patterns = Ref::map(repeating_hit_pattern.borrow(), |repeating| {
                repeating.alternating_mono_patterns.as_slice()
            });

            for (i, mono_pattern) in mono_patterns.iter().enumerate() {
                {
                    let mut mono_pattern = mono_pattern.borrow_mut();
                    mono_pattern.parent = Some(Rc::downgrade(&repeating_hit_pattern));
                    mono_pattern.idx = i;
                }

                if let Some(obj) = mono_pattern.borrow().first_hit_object() {
                    obj.borrow_mut().color.alternating_mono_pattern =
                        Some(Rc::downgrade(mono_pattern));
                }

                let mono_streaks = Ref::map(mono_pattern.borrow(), |alternating| {
                    alternating.mono_streaks.as_slice()
                });

                for (j, mono_streak) in mono_streaks.iter().enumerate() {
                    {
                        let mut borrowed = mono_streak.borrow_mut();
                        borrowed.parent = Some(Rc::downgrade(mono_pattern));
                        borrowed.idx = j;
                    }

                    if let Some(obj) = mono_streak.borrow().first_hit_object() {
                        obj.borrow_mut().color.mono_streak = Some(Rc::downgrade(mono_streak));
                    };
                }
            }
        }
    }

    fn encode(data: &TaikoDifficultyObjects) -> Vec<Rc<RefCell<RepeatingHitPatterns>>> {
        let mono_streaks = Self::encode_mono_streaks(data);
        let alternating_mono_patterns = Self::encode_alternating_mono_pattern(mono_streaks);

        Self::encode_repeating_hit_patterns(alternating_mono_patterns)
    }

    fn encode_mono_streaks(data: &TaikoDifficultyObjects) -> Vec<Rc<RefCell<MonoStreak>>> {
        let mut data_iter = data.objects.iter();

        let Some(taiko_object) = data_iter.next() else {
            return Vec::new();
        };

        let mut mono_streaks = vec![MonoStreak::new()];
        let mut curr_mono_streak = mono_streaks.last();

        if let Some(curr) = curr_mono_streak {
            curr.borrow_mut()
                .hit_objects
                .push(Rc::downgrade(taiko_object));
        }

        for taiko_object in data_iter {
            let condition = data
                .previous_note(&taiko_object.borrow(), 0)
                .filter(|prev| taiko_object.borrow().base_hit_type == prev.borrow().base_hit_type);

            if condition.is_none() {
                mono_streaks.push(MonoStreak::new());
                curr_mono_streak = mono_streaks.last();
            }

            if let Some(curr) = curr_mono_streak {
                curr.borrow_mut()
                    .hit_objects
                    .push(Rc::downgrade(taiko_object));
            }
        }

        mono_streaks
    }

    fn encode_alternating_mono_pattern(
        data: Vec<Rc<RefCell<MonoStreak>>>,
    ) -> VecDeque<Rc<RefCell<AlternatingMonoPattern>>> {
        let mut data = data.into_iter();

        let Some(mono) = data.next() else {
            return VecDeque::new();
        };

        let mut mono_patterns = VecDeque::new();
        mono_patterns.push_back(AlternatingMonoPattern::new());
        let mut curr_mono_pattern = mono_patterns.back();

        let mut prev_run_len = mono.borrow().run_len();

        if let Some(curr) = curr_mono_pattern {
            curr.borrow_mut().mono_streaks.push(mono);
        }

        for mono in data {
            let run_len = mono.borrow().run_len();

            if run_len != prev_run_len {
                mono_patterns.push_back(AlternatingMonoPattern::new());
                curr_mono_pattern = mono_patterns.back();
            }

            prev_run_len = run_len;

            if let Some(curr_mono_pattern) = curr_mono_pattern {
                curr_mono_pattern.borrow_mut().mono_streaks.push(mono);
            }
        }

        mono_patterns
    }

    fn encode_repeating_hit_patterns(
        mut data: VecDeque<Rc<RefCell<AlternatingMonoPattern>>>,
    ) -> Vec<Rc<RefCell<RepeatingHitPatterns>>> {
        let mut hit_patterns = Vec::new();
        let mut curr_hit_pattern = None;

        while !data.is_empty() {
            let old = curr_hit_pattern.as_ref().map(Rc::downgrade);
            let curr_hit_pattern = &*curr_hit_pattern.insert(RepeatingHitPatterns::new(old));

            let mut is_coupled = data.get(2).map_or(false, |other| {
                data[0].borrow().is_repetition_of(&other.borrow())
            });

            if is_coupled {
                while is_coupled {
                    curr_hit_pattern
                        .borrow_mut()
                        .alternating_mono_patterns
                        .push(data.pop_front().unwrap());

                    is_coupled = data.get(2).map_or(false, |other| {
                        data[0].borrow().is_repetition_of(&other.borrow())
                    });
                }

                for front in data.drain(..2) {
                    curr_hit_pattern
                        .borrow_mut()
                        .alternating_mono_patterns
                        .push(front);
                }
            } else {
                curr_hit_pattern
                    .borrow_mut()
                    .alternating_mono_patterns
                    .push(data.pop_front().unwrap());
            }

            hit_patterns.push(Rc::clone(curr_hit_pattern));
        }

        hit_patterns
            .iter_mut()
            .for_each(|pattern| pattern.borrow_mut().find_repetition_interval());

        hit_patterns
    }
}
