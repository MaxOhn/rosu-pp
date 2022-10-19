use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use crate::taiko::difficulty_object::ObjectLists;

use super::{
    alternating_mono_pattern::AlternatingMonoPattern, mono_streak::MonoStreak,
    repeating_hit_patterns::RepeatingHitPatterns,
};

pub(crate) struct ColourDifficultyPreprocessor;

impl ColourDifficultyPreprocessor {
    pub(crate) fn process_and_assign(lists: &mut ObjectLists) {
        // * Assign indexing and encoding data to all relevant objects. Only the first note of each encoding type is
        // * assigned with the relevant encodings.
        for repeating_hit_pattern in Self::encode(lists) {
            if let Some(obj) = repeating_hit_pattern
                .borrow()
                .first_hit_object()
                .as_ref()
                .and_then(Weak::upgrade)
            {
                obj.borrow_mut().colour.repeating_hit_patterns =
                    Some(Rc::clone(&repeating_hit_pattern));
            }

            // * The outermost loop is kept a ForEach loop since it doesn't need index information, and we want to
            // * keep i and j for AlternatingMonoPattern's and MonoStreak's index respectively, to keep it in line with
            // * documentation.
            for i in 0..repeating_hit_pattern
                .borrow()
                .alternating_mono_patterns
                .len()
            {
                let borrowed_repeating_hit_pattern = repeating_hit_pattern.borrow();
                let mono_pattern = &borrowed_repeating_hit_pattern.alternating_mono_patterns[i];

                {
                    let mut borrowed = mono_pattern.borrow_mut();
                    borrowed.parent = Some(Rc::downgrade(&repeating_hit_pattern));
                    borrowed.idx = i;
                }

                if let Some(obj) = mono_pattern
                    .borrow()
                    .first_hit_object()
                    .as_ref()
                    .and_then(Weak::upgrade)
                {
                    obj.borrow_mut().colour.alternating_mono_pattern =
                        Some(Rc::downgrade(mono_pattern));
                }

                for j in 0..mono_pattern.borrow().mono_streaks.len() {
                    let borrowed_mono_pattern = mono_pattern.borrow();
                    let mono_streak = &borrowed_mono_pattern.mono_streaks[j];

                    {
                        let mut borrowed = mono_streak.borrow_mut();
                        borrowed.parent = Some(Rc::downgrade(mono_pattern));
                        borrowed.idx = j;
                    }

                    if let Some(obj) = mono_streak
                        .borrow()
                        .first_hit_object()
                        .as_ref()
                        .and_then(Weak::upgrade)
                    {
                        obj.borrow_mut().colour.mono_streak = Some(Rc::downgrade(mono_streak));
                    };
                }
            }
        }
    }

    fn encode(data: &mut ObjectLists) -> Vec<Rc<RefCell<RepeatingHitPatterns>>> {
        let mono_streaks = Self::encode_mono_streak(data);
        let alternating_mono_patterns = Self::encode_alternating_mono_pattern(mono_streaks);

        Self::encode_repeating_hit_pattern(alternating_mono_patterns)
    }

    fn encode_mono_streak(data: &mut ObjectLists) -> Vec<Rc<RefCell<MonoStreak>>> {
        let mut mono_streaks = vec![MonoStreak::new()];
        let mut curr_mono_streak = mono_streaks.last_mut();
        let mut data_iter = data.all.iter();

        if let (Some(curr), Some(taiko_obj)) = (&curr_mono_streak, data_iter.next()) {
            curr.borrow_mut().hit_objects.push(Rc::downgrade(taiko_obj));
        }

        for taiko_obj in data_iter {
            // * This ignores all non-note objects, which may or may not be the desired behaviour
            let prev = data.prev_note(taiko_obj.borrow().idx, 0);

            // * If this is the first object in the list or the colour changed, create a new mono streak
            let condition = prev.filter(|prev| {
                !(taiko_obj.borrow().base.is_hit
                    && prev.borrow().base.is_hit
                    && (taiko_obj.borrow().base.is_rim != prev.borrow().base.is_rim))
            });

            if condition.is_none() {
                mono_streaks.push(MonoStreak::new());
                curr_mono_streak = mono_streaks.last_mut();
            }

            // * Add the current object to the encoded payload.
            if let Some(ref curr) = curr_mono_streak {
                curr.borrow_mut().hit_objects.push(Rc::downgrade(taiko_obj));
            }
        }

        mono_streaks
    }

    fn encode_alternating_mono_pattern(
        data: Vec<Rc<RefCell<MonoStreak>>>,
    ) -> VecDeque<Rc<RefCell<AlternatingMonoPattern>>> {
        let mut mono_patterns = VecDeque::new();
        mono_patterns.push_back(AlternatingMonoPattern::new());
        let mut curr_mono_pattern = mono_patterns.back_mut();

        if let (Some(curr), Some(mono)) = (&curr_mono_pattern, data.first()) {
            curr.borrow_mut().mono_streaks.push(Rc::clone(mono));
        }

        for (prev, curr) in data.iter().zip(data.iter().skip(1)) {
            // * Start a new AlternatingMonoPattern if the previous MonoStreak has a different mono length,
            // * or if this is the first MonoStreak in the list.
            if curr.borrow().run_len() != prev.borrow().run_len() {
                mono_patterns.push_back(AlternatingMonoPattern::new());
                curr_mono_pattern = mono_patterns.back_mut();
            }

            // * Add the current MonoStreak to the encoded payload.
            if let Some(ref curr_mono_pattern) = curr_mono_pattern {
                curr_mono_pattern
                    .borrow_mut()
                    .mono_streaks
                    .push(Rc::clone(curr));
            }
        }

        mono_patterns
    }

    fn encode_repeating_hit_pattern(
        mut data: VecDeque<Rc<RefCell<AlternatingMonoPattern>>>,
    ) -> Vec<Rc<RefCell<RepeatingHitPatterns>>> {
        let mut hit_patterns = Vec::new();
        let mut curr_hit_pattern: Option<Rc<std::cell::RefCell<_>>> = None;

        while !data.is_empty() {
            let old = curr_hit_pattern.as_ref().map(Rc::downgrade);
            let curr_hit_pattern = curr_hit_pattern.insert(RepeatingHitPatterns::new(old));

            let mut is_coupled = data.get(2).map_or(false, |other| {
                data[0].borrow().is_repetition_of(&other.borrow())
            });

            if is_coupled {
                // * If so, add the current AlternatingMonoPattern to the encoded payload and start repeatedly checking if the
                // * subsequent AlternatingMonoPatterns should be grouped by increasing i and doing the appropriate isCoupled check.
                while is_coupled {
                    curr_hit_pattern
                        .borrow_mut()
                        .alternating_mono_patterns
                        .push(data.pop_front().unwrap());

                    is_coupled = data.get(2).map_or(false, |other| {
                        data[0].borrow().is_repetition_of(&other.borrow())
                    });
                }

                // * Skip over viewed data and add the rest to the payload
                for front in data.drain(..2) {
                    curr_hit_pattern
                        .borrow_mut()
                        .alternating_mono_patterns
                        .push(front);
                }
            } else {
                // * If not, add the current AlternatingMonoPattern to the encoded payload and continue.
                curr_hit_pattern
                    .borrow_mut()
                    .alternating_mono_patterns
                    .push(data.pop_front().unwrap());
            }

            hit_patterns.push(Rc::clone(&*curr_hit_pattern));
        }

        hit_patterns
            .iter_mut()
            .for_each(|pattern| pattern.borrow_mut().find_repetition_interval());

        hit_patterns
    }
}
