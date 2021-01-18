#![allow(unused)]

mod difficulty_object;
mod osu_object;
mod skill;
mod skill_kind;

use difficulty_object::DifficultyObject;
use osu_object::OsuObject;
use skill::Skill;
use skill_kind::SkillKind;

use super::super::DifficultyAttributes;
use crate::{Beatmap, Mods};

const SECTION_LEN: f32 = 400.0;
const DIFFICULTY_MULTIPLIER: f32 = 0.0675;

/// Star calculation for osu!standard maps
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

    let mut hit_objects = map
        .hit_objects
        .iter()
        .take(take)
        .filter_map(|h| OsuObject::new(h, map, &attributes));

    let mut skills = vec![Skill::new(SkillKind::Aim), Skill::new(SkillKind::Speed)];

    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_diff = None;

    let mut _i = 0;

    for curr in hit_objects {
        let h = DifficultyObject::new(
            curr.clone(),
            prev.clone(),
            prev_diff,
            prev_prev,
            attributes.clock_rate,
        );

        // println!(
        //     "strain_time={} | travel_dist={} | jump_dist={} | angle={:?}",
        //     h.strain_time, h.travel_dist, h.jump_dist, h.angle
        // );

        // println!("[{}] time={}", _i, curr.time());

        while h.base.time() > current_section_end {
            for skill in skills.iter_mut() {
                skill.save_current_peak();
                skill.start_new_section_from(current_section_end);

                _i += 1;
            }

            current_section_end += section_len;
        }

        for skill in skills.iter_mut() {
            skill.process(&h);
        }

        prev_prev = Some(prev);
        prev = curr;
        prev_diff = Some(h);
    }

    for skill in skills.iter_mut() {
        skill.save_current_peak();
    }

    // println!("Aim:");
    // for (i, strain) in skills[0].strain_peaks.iter().enumerate() {
    //     println!("{}: {}", i, strain);
    // }

    // println!("Speed:");
    // for (i, strain) in skills[1].strain_peaks.iter().enumerate() {
    //     println!("{}: {}", i, strain);
    // }

    // println!("Aim: {:?}", skills[0].strain_peaks);
    // println!("Speed: {:?}", skills[1].strain_peaks);

    let aim_rating = skills[0].difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    // println!("After:\n{:?}", skills[0].strain_peaks);

    let speed_rating = skills[1].difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    // println!("After:\n{:?}", skills[1].strain_peaks);

    let stars = aim_rating + speed_rating + (aim_rating - speed_rating).abs() / 2.0;

    DifficultyAttributes {
        stars,
        ar: attributes.ar,
        od,
        speed_strain: speed_rating,
        aim_strain: aim_rating,
        max_combo: 0,  // TODO
        n_circles: 0,  // TODO
        n_spinners: 0, // TODO
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
    fn all_included_single() {
        let file = match File::open("./maps/70090.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let result = OsuPP::new(&map).mods(64).calculate(stars);

        println!("Stars: {}", result.stars());
        println!("PP: {}", result.pp());
    }
}
