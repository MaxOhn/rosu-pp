//! In addtion to not considering the positional offset caused by stack leniency, slider paths are also ignored.
//! This means the travel distance of notes is completely omitted which may cause further inaccuracies.
//! Since the slider paths don't have to be computed though, it should generally be faster than `no_leniency`.

use super::super::DifficultyAttributes;

mod control_point_iter;
mod difficulty_object;
mod skill;
mod skill_kind;
mod slider_state;

use difficulty_object::DifficultyObject;
use skill::Skill;
use skill_kind::SkillKind;
use slider_state::SliderState;

use crate::{Beatmap, HitObject, HitObjectKind, Mods};

use std::borrow::Cow;

const OBJECT_RADIUS: f32 = 64.0;
const SECTION_LEN: f32 = 400.0;
const DIFFICULTY_MULTIPLIER: f32 = 0.0675;
const NORMALIZED_RADIUS: f32 = 52.0;

/// Star calculation for osu!standard maps.
///
/// Sliders are considered as regular hitcircles and stack leniency is ignored.
/// Still very good results but the least precise version in general.
/// However, this is the most efficient one.
///
/// In case of a partial play, e.g. a fail, one can specify the amount of passed objects.
pub fn stars(
    map: &Beatmap,
    mods: impl Mods,
    passed_objects: Option<usize>,
) -> DifficultyAttributes {
    let take = passed_objects.unwrap_or_else(|| map.hit_objects.len());

    let attributes = map.attributes().mods(mods);
    let hitwindow = super::difficulty_range(attributes.od).floor() / attributes.clock_rate;
    let od = (80.0 - hitwindow) / 6.0;

    if take < 2 {
        return DifficultyAttributes {
            ar: attributes.ar,
            od,
            ..Default::default()
        };
    }

    let section_len = SECTION_LEN * attributes.clock_rate;
    let radius = OBJECT_RADIUS * (1.0 - 0.7 * (attributes.cs - 5.0) / 5.0) / 2.0;
    let mut scaling_factor = NORMALIZED_RADIUS / radius;

    if radius < 30.0 {
        let small_circle_bonus = (30.0 - radius).min(5.0) / 50.0;
        scaling_factor *= 1.0 + small_circle_bonus;
    }

    let mut max_combo = 0;
    let mut n_circles = 0;
    let mut n_spinners = 0;
    let mut state = SliderState::new(&map);

    let mut hit_objects = map.hit_objects.iter().take(take).map(|h| match &h.kind {
        HitObjectKind::Circle => {
            max_combo += 1;
            n_circles += 1;

            Cow::Borrowed(h)
        }
        HitObjectKind::Slider {
            pixel_len, repeats, ..
        } => {
            max_combo += state.count_ticks(h.start_time, *pixel_len, *repeats, &map);

            Cow::Owned(HitObject {
                pos: h.pos,
                start_time: h.start_time,
                kind: HitObjectKind::Circle,
                sound: h.sound,
            })
        }
        HitObjectKind::Spinner { .. } => {
            max_combo += 1;
            n_spinners += 1;

            Cow::Borrowed(h)
        }
        HitObjectKind::Hold { .. } => panic!("found Hold object in osu!standard map"),
    });

    let mut aim = Skill::new(SkillKind::Aim);
    let mut speed = Skill::new(SkillKind::Speed);

    // First object has no predecessor and thus no strain, handle distinctly
    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_vals = None;

    // Handle second object separately to remove later if-branching
    let curr = hit_objects.next().unwrap();
    let h = DifficultyObject::new(
        curr.as_ref(),
        prev.as_ref(),
        prev_vals,
        prev_prev,
        attributes.clock_rate,
        scaling_factor,
    );

    while h.base.start_time > current_section_end {
        current_section_end += section_len;
    }

    aim.process(&h);
    speed.process(&h);

    prev_prev = Some(prev);
    prev_vals = Some((h.jump_dist, h.strain_time));
    prev = curr;

    // Handle all other objects
    for curr in hit_objects {
        let h = DifficultyObject::new(
            curr.as_ref(),
            prev.as_ref(),
            prev_vals,
            prev_prev,
            attributes.clock_rate,
            scaling_factor,
        );

        while h.base.start_time > current_section_end {
            aim.save_current_peak();
            aim.start_new_section_from(current_section_end);
            speed.save_current_peak();
            speed.start_new_section_from(current_section_end);

            current_section_end += section_len;
        }

        aim.process(&h);
        speed.process(&h);

        prev_prev = Some(prev);
        prev_vals = Some((h.jump_dist, h.strain_time));
        prev = curr;
    }

    aim.save_current_peak();
    speed.save_current_peak();

    let aim_strain = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    let speed_strain = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    let stars = aim_strain + speed_strain + (aim_strain - speed_strain).abs() / 2.0;

    DifficultyAttributes {
        stars,
        ar: attributes.ar,
        od,
        speed_strain,
        aim_strain,
        max_combo,
        n_circles,
        n_spinners,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::OsuPP;
    use super::stars;
    use crate::Beatmap;
    use std::fs::File;

    #[test]
    #[ignore]
    fn no_sliders_single() {
        let file = match File::open("./maps/1851299.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let result = OsuPP::new(&map).mods(2).calculate(stars);

        println!("Stars: {}", result.stars());
        println!("PP: {}", result.pp());
    }
}
