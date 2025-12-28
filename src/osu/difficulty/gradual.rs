use std::{cmp, mem};

use rosu_map::section::general::GameMode;

use crate::{
    Beatmap, Difficulty,
    any::difficulty::skills::StrainSkill,
    model::mode::ConvertError,
    osu::{
        convert::convert_objects,
        legacy_score_simulator::gradual::OsuGradualLegacyScoreSimulator,
        object::{OsuObject, OsuObjectKind},
    },
};

use self::osu_objects::OsuObjects;

use super::{
    DifficultyValues, OsuDifficultyAttributes, OsuDifficultySetup, object::OsuDifficultyObject,
    skills::OsuSkills,
};

/// Gradually calculate the difficulty attributes of an osu!standard map.
///
/// Note that this struct implements [`Iterator`].
/// On every call of [`Iterator::next`], the map's next hit object will
/// be processed and the [`OsuDifficultyAttributes`] will be updated and
/// returned.
///
/// If you want to calculate performance attributes, use
/// [`OsuGradualPerformance`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::osu::{Osu, OsuGradualDifficulty};
///
/// let map = Beatmap::from_path("./resources/2785319.osu").unwrap();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = OsuGradualDifficulty::new(difficulty, &map).unwrap();
///
///  // the difficulty of the map after the first hit object
/// let attrs1 = iter.next();
/// // ... after the second hit object
/// let attrs2 = iter.next();
///
/// // Remaining hit objects
/// for difficulty in iter {
///     // ...
/// }
/// ```
///
/// [`OsuGradualPerformance`]: crate::osu::OsuGradualPerformance
pub struct OsuGradualDifficulty {
    pub(crate) idx: usize,
    pub(crate) difficulty: Difficulty,
    attrs: OsuDifficultyAttributes,
    skills: OsuSkills,
    // Lifetimes actually depend on `osu_objects` so this type is
    // self-referential. This field must be treated with great caution, moving
    // `osu_objects` will immediately invalidate `diff_objects`.
    diff_objects: Box<[OsuDifficultyObject<'static>]>,
    osu_objects: OsuObjects,
    score_simulator: OsuGradualLegacyScoreSimulator,
    // Additional safety measure that this type can't be cloned as it would
    // invalidate `diff_objects`.
    _not_clonable: NotClonable,
}

struct NotClonable;

impl OsuGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!standard maps.
    pub fn new(difficulty: Difficulty, map: &Beatmap) -> Result<Self, ConvertError> {
        let mods = difficulty.get_mods();
        let map = map.convert_ref(GameMode::Osu, mods)?;

        let OsuDifficultySetup {
            scaling_factor,
            map_attrs,
            mut attrs,
            time_preempt,
        } = OsuDifficultySetup::new(&difficulty, &map);

        let osu_objects = convert_objects(
            &map,
            &scaling_factor,
            mods.reflection(),
            time_preempt,
            map.hit_objects.len(),
            &mut attrs,
        );

        attrs.n_circles = 0;
        attrs.n_sliders = 0;
        attrs.n_large_ticks = 0;
        attrs.n_spinners = 0;
        attrs.max_combo = 0;

        if let Some(h) = osu_objects.first() {
            Self::increment_combo(h, &mut attrs);
        }

        let mut osu_objects = OsuObjects::new(osu_objects);

        let diff_objects = DifficultyValues::create_difficulty_objects(
            &difficulty,
            &scaling_factor,
            osu_objects.iter_mut(),
        );

        let skills = OsuSkills::new(mods, &scaling_factor, &map_attrs, time_preempt);
        let diff_objects = extend_lifetime(diff_objects.into_boxed_slice());
        let score_simulator = OsuGradualLegacyScoreSimulator::new(&map, map_attrs);

        Ok(Self {
            idx: 0,
            difficulty,
            attrs,
            skills,
            diff_objects,
            osu_objects,
            score_simulator,
            _not_clonable: NotClonable,
        })
    }

    fn increment_combo(h: &OsuObject, attrs: &mut OsuDifficultyAttributes) {
        attrs.max_combo += 1;

        match &h.kind {
            OsuObjectKind::Circle => attrs.n_circles += 1,
            OsuObjectKind::Slider(slider) => {
                attrs.n_sliders += 1;
                attrs.n_large_ticks += slider.large_tick_count() as u32;
                attrs.max_combo += slider.nested_objects.len() as u32;
            }
            OsuObjectKind::Spinner { .. } => attrs.n_spinners += 1,
        }
    }
}

fn extend_lifetime(
    diff_objects: Box<[OsuDifficultyObject<'_>]>,
) -> Box<[OsuDifficultyObject<'static>]> {
    // SAFETY: Owned values of the references will be contained in the same
    // struct (same lifetime). Also, the only mutable access wraps them in
    // `Pin` to ensure that they won't move.
    unsafe { mem::transmute(diff_objects) }
}

impl Iterator for OsuGradualDifficulty {
    type Item = OsuDifficultyAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(h) = self.osu_objects.get(self.idx) {
            let score_attrs = self.score_simulator.simulate_next(h);
            self.attrs.maximum_legacy_combo_score = score_attrs.combo_score as f64;
        }

        // The first difficulty object belongs to the second note since each
        // difficulty object requires the current and the last note. Hence, if
        // we're still on the first object, we don't have a difficulty object
        // yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;

            self.skills.aim.process(curr, &self.diff_objects);
            self.skills.aim_no_sliders.process(curr, &self.diff_objects);
            self.skills.speed.process(curr, &self.diff_objects);
            self.skills.flashlight.process(curr, &self.diff_objects);

            Self::increment_combo(curr.base, &mut self.attrs);
        } else if self.osu_objects.is_empty() {
            return None;
        }

        self.idx += 1;

        let mut attrs = self.attrs.clone();

        DifficultyValues::eval(&mut attrs, self.difficulty.get_mods(), &self.skills);

        Some(attrs)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let skip_iter = self.diff_objects.iter().skip(self.idx.saturating_sub(1));

        let mut take = cmp::min(n, self.len().saturating_sub(1));

        // The first note has no difficulty object
        if self.idx == 0 && take > 0 {
            if let Some(h) = self.osu_objects.get(self.idx) {
                let score_attrs = self.score_simulator.simulate_next(h);
                self.attrs.maximum_legacy_combo_score = score_attrs.combo_score as f64;
            }

            take -= 1;
            self.idx += 1;
        }

        for curr in skip_iter.take(take) {
            if let Some(h) = self.osu_objects.get(self.idx) {
                let score_attrs = self.score_simulator.simulate_next(h);
                self.attrs.maximum_legacy_combo_score = score_attrs.combo_score as f64;
            }

            self.skills.process(curr, &self.diff_objects);
            Self::increment_combo(curr.base, &mut self.attrs);
            self.idx += 1;
        }

        self.next()
    }
}

impl ExactSizeIterator for OsuGradualDifficulty {
    fn len(&self) -> usize {
        self.diff_objects.len() + 1 - self.idx
    }
}

mod osu_objects {
    use std::pin::Pin;

    use crate::osu::object::OsuObject;

    /// Wrapper to ensure that the data will not be moved
    pub(super) struct OsuObjects {
        objects: Box<[OsuObject]>,
    }

    impl OsuObjects {
        pub(super) const fn new(objects: Box<[OsuObject]>) -> Self {
            Self { objects }
        }

        pub(super) fn get(&self, idx: usize) -> Option<&OsuObject> {
            self.objects.get(idx)
        }

        pub(super) const fn is_empty(&self) -> bool {
            self.objects.is_empty()
        }

        pub(super) fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = Pin<&mut OsuObject>> {
            self.objects.iter_mut().map(Pin::new)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Beatmap, osu::Osu};

    use super::*;

    #[test]
    fn empty() {
        let map = Beatmap::from_bytes(&[]).unwrap();
        let mut gradual = OsuGradualDifficulty::new(Difficulty::new(), &map).unwrap();
        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let map = Beatmap::from_path("./resources/2785319.osu").unwrap();

        let difficulty = Difficulty::new();

        let mut gradual = OsuGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_2nd = OsuGradualDifficulty::new(difficulty.clone(), &map).unwrap();
        let mut gradual_3rd = OsuGradualDifficulty::new(difficulty.clone(), &map).unwrap();

        let hit_objects_len = map.hit_objects.len();

        for i in 1.. {
            let Some(next_gradual) = gradual.next() else {
                assert_eq!(i, hit_objects_len + 1);
                assert!(gradual_2nd.last().is_some() || hit_objects_len % 2 == 0);
                assert!(gradual_3rd.last().is_some() || hit_objects_len % 3 == 0);
                break;
            };

            if i % 2 == 0 {
                let next_gradual_2nd = gradual_2nd.nth(1).unwrap();
                assert_eq!(next_gradual, next_gradual_2nd);
            }

            if i % 3 == 0 {
                let next_gradual_3rd = gradual_3rd.nth(2).unwrap();
                assert_eq!(next_gradual, next_gradual_3rd);
            }

            let expected = difficulty
                .clone()
                .passed_objects(i as u32)
                .calculate_for_mode::<Osu>(&map)
                .unwrap();

            assert_eq!(next_gradual, expected);
        }
    }
}
