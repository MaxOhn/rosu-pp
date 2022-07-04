use super::Rim;
use crate::{limited_queue::LimitedQueue, Beatmap};

const ROLL_MIN_REPETITIONS: usize = 12;
const TL_MIN_REPETITIONS: isize = 16;

pub(crate) trait StaminaCheeseDetector {
    fn find_cheese(&self) -> Vec<bool>;
    fn find_rolls<const PATTERN_LEN: usize, const DOUBLE_PATTERN_LEN: usize>(
        &self,
        cheese: &mut [bool],
    );
    fn find_tl_tap<const PARITY: usize, const IS_RIN: bool>(&self, cheese: &mut [bool]);
}

impl StaminaCheeseDetector for Beatmap {
    fn find_cheese(&self) -> Vec<bool> {
        let mut cheese = vec![false; self.hit_objects.len()];

        self.find_rolls::<3, 6>(&mut cheese);
        self.find_rolls::<4, 8>(&mut cheese);

        self.find_tl_tap::<0, true>(&mut cheese);
        self.find_tl_tap::<1, true>(&mut cheese);
        self.find_tl_tap::<0, false>(&mut cheese);
        self.find_tl_tap::<1, false>(&mut cheese);

        cheese
    }

    fn find_rolls<const PATTERN_LEN: usize, const DOUBLE_PATTERN_LEN: usize>(
        &self,
        cheese: &mut [bool],
    ) {
        let mut history: LimitedQueue<u8, DOUBLE_PATTERN_LEN> = LimitedQueue::new();

        let mut index_before_last_repeat = -1;
        let mut last_mark_end = 0;

        for (i, &h) in self.sounds.iter().enumerate() {
            history.push(h);

            if !history.full() {
                continue;
            }

            let contains = contains_pattern_repeat::<PATTERN_LEN, DOUBLE_PATTERN_LEN>(&history);

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

    fn find_tl_tap<const PARITY: usize, const IS_RIN: bool>(&self, cheese: &mut [bool]) {
        let mut tl_len = -2;
        let mut last_mark_end = 0;

        for (i, &sound) in self.sounds.iter().enumerate().skip(PARITY).step_by(2) {
            if sound.is_rim() == IS_RIN {
                tl_len += 2;
            } else {
                tl_len = -2;
            }

            if tl_len < TL_MIN_REPETITIONS {
                continue;
            }

            let start = (i as isize + 1 - tl_len).max(last_mark_end as isize);
            mark_as_cheese(start as usize, i, cheese);

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
fn contains_pattern_repeat<const PATTERN_LEN: usize, const DOUBLE_PATTERN_LEN: usize>(
    history: &LimitedQueue<u8, DOUBLE_PATTERN_LEN>,
) -> bool {
    for (&curr, &to_compare) in history.iter().zip(history.iter().skip(PATTERN_LEN)) {
        if curr.is_rim() != to_compare.is_rim() {
            return false;
        }
    }

    true
}
