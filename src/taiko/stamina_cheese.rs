use super::{LimitedQueue, Rim};
use crate::{parse::HitObject, Beatmap};

const ROLL_MIN_REPETITIONS: usize = 12;
const TL_MIN_REPETITIONS: isize = 16;

pub(crate) trait StaminaCheeseDetector {
    fn find_cheese(&self) -> Vec<bool>;
    fn find_rolls(&self, pattern_len: usize, cheese: &mut [bool]);
    fn find_tl_tap(&self, parity: usize, is_rin: bool, cheese: &mut [bool]);
}

impl StaminaCheeseDetector for Beatmap {
    fn find_cheese(&self) -> Vec<bool> {
        let mut cheese = vec![false; self.hit_objects.len()];

        self.find_rolls(3, &mut cheese);
        self.find_rolls(4, &mut cheese);

        self.find_tl_tap(0, true, &mut cheese);
        self.find_tl_tap(1, true, &mut cheese);
        self.find_tl_tap(0, false, &mut cheese);
        self.find_tl_tap(1, false, &mut cheese);

        cheese
    }

    fn find_rolls(&self, pattern_len: usize, cheese: &mut [bool]) {
        let mut history = LimitedQueue::new(2 * pattern_len);

        let mut index_before_last_repeat = -1;
        let mut last_mark_end = 0;

        for (i, h) in self.hit_objects.iter().enumerate() {
            history.push(h);

            if !history.full() {
                continue;
            }

            let contains = contains_pattern_repeat(&history, pattern_len);

            if !contains {
                index_before_last_repeat = (i + 1 - history.len()) as isize;

                continue;
            }

            let repeated_len = (i as isize - index_before_last_repeat) as usize;

            if repeated_len < ROLL_MIN_REPETITIONS {
                continue;
            }

            mark_as_cheese(last_mark_end.max(i + 1 - repeated_len), i, cheese);

            last_mark_end = i;
        }
    }

    fn find_tl_tap(&self, parity: usize, is_rin: bool, cheese: &mut [bool]) {
        let mut tl_len = -2;
        let mut last_mark_end = 0;

        for (i, h) in self.hit_objects.iter().enumerate().skip(parity).step_by(2) {
            if h.is_rim() == is_rin {
                tl_len += 2;
            } else {
                tl_len = -2;
            }

            if tl_len < TL_MIN_REPETITIONS {
                continue;
            }

            mark_as_cheese(
                (i as isize + 1 - tl_len).max(last_mark_end as isize) as usize,
                i,
                cheese,
            );

            last_mark_end = i;
        }
    }
}

#[inline]
fn mark_as_cheese(start: usize, end: usize, cheese: &mut [bool]) {
    cheese
        .iter_mut()
        .take(end + 1)
        .skip(start)
        .for_each(|b| *b = true);
}

#[inline]
fn contains_pattern_repeat(history: &LimitedQueue<&HitObject>, pattern_len: usize) -> bool {
    for (&curr, &to_compare) in history.iter().zip(history.iter().skip(pattern_len)) {
        if curr.is_rim() != to_compare.is_rim() {
            return false;
        }
    }

    true
}
