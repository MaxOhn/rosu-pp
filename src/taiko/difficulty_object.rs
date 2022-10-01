use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use crate::parse::HitObject;

use super::{colours::TaikoDifficultyColour, rim::Rim, taiko_object::TaikoObject};

#[derive(Clone, Debug, Default)]
pub(crate) struct ObjectLists<'o> {
    pub(crate) all: Vec<Rc<RefCell<TaikoDifficultyObject<'o>>>>,
    pub(crate) centres: Vec<usize>,
    pub(crate) rims: Vec<usize>,
    pub(crate) notes: Vec<usize>,
}

impl<'o> ObjectLists<'o> {
    pub(crate) fn prev_mono(
        &self,
        curr: usize,
        backwards_idx: usize,
    ) -> Option<&'_ Rc<RefCell<TaikoDifficultyObject<'o>>>> {
        let curr = &self.all[curr];

        let prev = match curr.borrow().mono_idx {
            MonoIndex::Centre(idx) => idx
                .checked_sub(backwards_idx + 1)
                .and_then(|idx| self.centres.get(idx))?,
            MonoIndex::Rim(idx) => idx
                .checked_sub(backwards_idx + 1)
                .and_then(|idx| self.rims.get(idx))?,
            MonoIndex::None => return None,
        };

        self.all.get(*prev)
    }

    #[allow(unused)]
    pub(crate) fn next_mono(
        &self,
        curr: usize,
        forwards_idx: usize,
    ) -> Option<&'_ Rc<RefCell<TaikoDifficultyObject<'o>>>> {
        let curr = &self.all[curr];

        let next = match curr.borrow().mono_idx {
            MonoIndex::Centre(idx) => self.centres.get(idx + (forwards_idx + 1))?,
            MonoIndex::Rim(idx) => self.rims.get(idx + (forwards_idx + 1))?,
            MonoIndex::None => return None,
        };

        self.all.get(*next)
    }

    pub(crate) fn prev_note(
        &self,
        curr: usize,
        backwards_idx: usize,
    ) -> Option<&'_ Rc<RefCell<TaikoDifficultyObject<'o>>>> {
        let curr = &self.all[curr];
        let note_idx = curr.borrow().note_idx?;
        let idx = note_idx.checked_sub(backwards_idx + 1)?;
        let prev = self.notes.get(idx)?;

        self.all.get(*prev)
    }

    #[allow(unused)]
    pub(crate) fn next_note(
        &self,
        curr: usize,
        forwards_idx: usize,
    ) -> Option<&'_ Rc<RefCell<TaikoDifficultyObject<'o>>>> {
        let curr = &self.all[curr];
        let note_idx = curr.borrow().note_idx?;
        let idx = note_idx + (forwards_idx + 1);
        let prev = self.notes.get(idx)?;

        self.all.get(*prev)
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum MonoIndex {
    Centre(usize),
    Rim(usize),
    None,
}

#[derive(Clone, Debug)]
pub(crate) struct TaikoDifficultyObject<'o> {
    pub(crate) base: TaikoObject<'o>,
    pub(crate) prev_time: f64,
    pub(crate) colour: TaikoDifficultyColour<'o>,
    pub(crate) rhythm: &'static HitObjectRhythm,
    pub(crate) mono_idx: MonoIndex,
    pub(crate) note_idx: Option<usize>,
    pub(crate) idx: usize,
    pub(crate) delta: f64,
}

impl<'o> TaikoDifficultyObject<'o> {
    pub(crate) fn new(
        base: TaikoObject<'o>,
        last: TaikoObject<'o>,
        last_last: TaikoObject<'o>,
        clock_rate: f64,
        lists: &ObjectLists<'o>,
        idx: usize,
    ) -> Self {
        // * Create the Colour object, its properties should be filled in by TaikoDifficultyPreprocessor
        let colour = TaikoDifficultyColour::default();

        let delta = (base.h.start_time - last.h.start_time) / clock_rate;
        let rhythm = closest_rhythm(delta, last.h, last_last.h, clock_rate);

        let mono_idx = if !base.is_hit() {
            MonoIndex::None
        } else if base.sound.is_rim() {
            MonoIndex::Rim(lists.rims.len())
        } else {
            MonoIndex::Centre(lists.centres.len())
        };

        let note_idx = base.is_hit().then_some(lists.notes.len());

        Self {
            base,
            prev_time: last.h.start_time / clock_rate,
            colour,
            rhythm,
            mono_idx,
            note_idx,
            idx,
            delta,
        }
    }
}

#[rustfmt::skip]
pub(crate) static COMMON_RHYTHMS: [HitObjectRhythm; 9] = [
    HitObjectRhythm { id: 0, ratio: 1.0, difficulty: 0.0 },
    HitObjectRhythm { id: 1, ratio: 2.0 / 1.0, difficulty: 0.3 },
    HitObjectRhythm { id: 2, ratio: 1.0 / 2.0, difficulty: 0.5 },
    HitObjectRhythm { id: 3, ratio: 3.0 / 1.0, difficulty: 0.3 },
    HitObjectRhythm { id: 4, ratio: 1.0 / 3.0, difficulty: 0.35 },
    // * purposefully higher (requires hand switch in full alternating gameplay style)
    HitObjectRhythm { id: 5, ratio: 3.0 / 2.0, difficulty: 0.6 },
    HitObjectRhythm { id: 6, ratio: 2.0 / 3.0, difficulty: 0.4 },
    HitObjectRhythm { id: 7, ratio: 5.0 / 4.0, difficulty: 0.5 },
    HitObjectRhythm { id: 8, ratio: 4.0 / 5.0, difficulty: 0.7 },
];

#[derive(Copy, Clone, Debug)]
pub(crate) struct HitObjectRhythm {
    id: u8,
    pub(crate) ratio: f64,
    pub(crate) difficulty: f64,
}

impl HitObjectRhythm {
    pub(crate) fn static_ref() -> &'static Self {
        &COMMON_RHYTHMS[0]
    }
}

impl PartialEq for HitObjectRhythm {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for HitObjectRhythm {}

fn closest_rhythm(
    delta_time: f64,
    last: &HitObject,
    last_last: &HitObject,
    clock_rate: f64,
) -> &'static HitObjectRhythm {
    let prev_len = (last.start_time - last_last.start_time) / clock_rate;
    let ratio = delta_time / prev_len;

    COMMON_RHYTHMS
        .iter()
        .min_by(|r1, r2| {
            (r1.ratio - ratio)
                .abs()
                .partial_cmp(&(r2.ratio - ratio).abs())
                .unwrap_or(Ordering::Equal)
        })
        .unwrap()
}
