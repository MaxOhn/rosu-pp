use std::{cmp, mem};

use crate::{
    any::difficulty::skills::Skill,
    osu::{
        convert::convert_objects,
        object::{OsuObject, OsuObjectKind},
        OsuBeatmap,
    },
    util::mods::Mods,
    Difficulty,
};

use self::osu_objects::OsuObjects;

use super::{
    object::OsuDifficultyObject, skills::OsuSkills, DifficultyValues, OsuDifficultyAttributes,
    OsuDifficultySetup,
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
/// let converted = Beatmap::from_path("./resources/2785319.osu")
///     .unwrap()
///     .unchecked_into_converted::<Osu>();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut iter = OsuGradualDifficulty::new(difficulty, &converted);
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
    // Additional safety measure that this type can't be cloned as it would
    // invalidate `diff_objects`.
    _not_clonable: NotClonable,
}

struct NotClonable;

impl OsuGradualDifficulty {
    /// Create a new difficulty attributes iterator for osu!standard maps.
    pub fn new(difficulty: Difficulty, converted: &OsuBeatmap<'_>) -> Self {
        let mods = difficulty.get_mods();

        let OsuDifficultySetup {
            scaling_factor,
            map_attrs,
            mut attrs,
            time_preempt,
        } = OsuDifficultySetup::new(&difficulty, converted);

        let osu_objects = convert_objects(
            converted,
            &scaling_factor,
            mods.hr(),
            time_preempt,
            converted.hit_objects.len(),
            &mut attrs,
        );

        attrs.n_circles = 0;
        attrs.n_sliders = 0;
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

        Self {
            idx: 0,
            difficulty,
            attrs,
            skills,
            diff_objects,
            osu_objects,
            _not_clonable: NotClonable,
        }
    }

    fn increment_combo(h: &OsuObject, attrs: &mut OsuDifficultyAttributes) {
        attrs.max_combo += 1;

        match &h.kind {
            OsuObjectKind::Circle => attrs.n_circles += 1,
            OsuObjectKind::Slider(slider) => {
                attrs.n_sliders += 1;
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
        // The first difficulty object belongs to the second note since each
        // difficulty object requires the current and the last note. Hence, if
        // we're still on the first object, we don't have a difficulty object
        // yet and just skip processing.
        if self.idx > 0 {
            let curr = self.diff_objects.get(self.idx - 1)?;

            Skill::new(&mut self.skills.aim, &self.diff_objects).process(curr);
            Skill::new(&mut self.skills.aim_no_sliders, &self.diff_objects).process(curr);
            Skill::new(&mut self.skills.speed, &self.diff_objects).process(curr);
            Skill::new(&mut self.skills.flashlight, &self.diff_objects).process(curr);

            Self::increment_combo(curr.base, &mut self.attrs);
        } else if self.osu_objects.is_empty() {
            return None;
        }

        self.idx += 1;

        let mut attrs = self.attrs.clone();

        let aim_difficulty_value = self.skills.aim.as_difficulty_value();
        let aim_no_sliders_difficulty_value = self.skills.aim_no_sliders.as_difficulty_value();
        let speed_relevant_note_count = self.skills.speed.relevant_note_count();
        let speed_difficulty_value = self.skills.speed.as_difficulty_value();
        let flashlight_difficulty_value = self.skills.flashlight.as_difficulty_value();

        DifficultyValues::eval(
            &mut attrs,
            self.difficulty.get_mods(),
            aim_difficulty_value,
            aim_no_sliders_difficulty_value,
            speed_difficulty_value,
            speed_relevant_note_count,
            flashlight_difficulty_value,
        );

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
            take -= 1;
            self.idx += 1;
        }

        let mut aim = Skill::new(&mut self.skills.aim, &self.diff_objects);
        let mut aim_no_sliders = Skill::new(&mut self.skills.aim_no_sliders, &self.diff_objects);
        let mut speed = Skill::new(&mut self.skills.speed, &self.diff_objects);
        let mut flashlight = Skill::new(&mut self.skills.flashlight, &self.diff_objects);

        for curr in skip_iter.take(take) {
            aim.process(curr);
            aim_no_sliders.process(curr);
            speed.process(curr);
            flashlight.process(curr);

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
    use crate::{osu::Osu, Beatmap};

    use super::*;

    #[test]
    fn empty() {
        let converted = Beatmap::from_bytes(&[])
            .unwrap()
            .unchecked_into_converted::<Osu>();

        let mut gradual = OsuGradualDifficulty::new(Difficulty::new(), &converted);

        assert!(gradual.next().is_none());
    }

    #[test]
    fn next_and_nth() {
        let converted = Beatmap::from_path("./resources/2785319.osu")
            .unwrap()
            .unchecked_into_converted::<Osu>();

        let difficulty = Difficulty::new();

        let mut gradual = OsuGradualDifficulty::new(difficulty.clone(), &converted);
        let mut gradual_2nd = OsuGradualDifficulty::new(difficulty.clone(), &converted);
        let mut gradual_3rd = OsuGradualDifficulty::new(difficulty.clone(), &converted);

        let hit_objects_len = converted.hit_objects.len();

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
                .with_mode()
                .calculate(&converted);

            assert_eq!(next_gradual, expected);
        }
    }
}
