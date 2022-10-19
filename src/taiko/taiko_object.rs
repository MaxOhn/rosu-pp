use std::slice::Iter;

use crate::{parse::HitObject, Beatmap};

use super::rim::Rim;

#[derive(Copy, Clone, Debug)]
pub(crate) struct TaikoObject {
    pub(crate) is_hit: bool,
    pub(crate) is_rim: bool,
}

impl TaikoObject {
    pub(crate) fn new(h: &HitObject, sound: u8) -> Self {
        Self {
            is_hit: h.is_circle(),
            is_rim: sound.is_rim(),
        }
    }
}

pub(crate) trait IntoTaikoObjectIter {
    fn taiko_objects(&self) -> TaikoObjectIter<'_>;
}

#[derive(Clone, Debug)]
pub(crate) struct TaikoObjectIter<'m> {
    hit_objects: Iter<'m, HitObject>,
    sounds: Iter<'m, u8>,
}

impl IntoTaikoObjectIter for Beatmap {
    #[inline]
    fn taiko_objects(&self) -> TaikoObjectIter<'_> {
        TaikoObjectIter {
            hit_objects: self.hit_objects.iter(),
            sounds: self.sounds.iter(),
        }
    }
}

impl<'m> Iterator for TaikoObjectIter<'m> {
    type Item = (TaikoObject, f64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let h = self.hit_objects.next()?;
        let sound = self.sounds.next()?;

        Some((TaikoObject::new(h, *sound), h.start_time))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hit_objects.size_hint()
    }
}

impl ExactSizeIterator for TaikoObjectIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.hit_objects.len()
    }
}
