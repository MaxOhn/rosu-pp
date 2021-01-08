mod curve;
mod difficulty_object;
mod math_util;
mod osu_object;
mod skill;
mod skill_kind;

use difficulty_object::DifficultyObject;
use osu_object::OsuObject;
use skill::Skill;
use skill_kind::SkillKind;

use parse::{Beatmap, Mods};

const SECTION_LEN: f32 = 400.0;
const DIFFICULTY_MULTIPLIER: f32 = 0.0675;

/// Star calculation for osu!standard maps
pub fn stars(map: &Beatmap, mods: impl Mods) -> f32 {
    if map.hit_objects.len() < 2 {
        return 0.0;
    }

    let attributes = map.attributes().mods(mods);
    let section_len = SECTION_LEN * attributes.clock_rate;

    let mut hit_objects = map
        .hit_objects
        .iter()
        .map(|h| OsuObject::new(h, map, &attributes));

    let mut skills = vec![Skill::new(SkillKind::Aim), Skill::new(SkillKind::Speed)];

    let mut current_section_end =
        (map.hit_objects[0].start_time / section_len).ceil() * section_len;

    let mut prev_prev = None;
    let mut prev = hit_objects.next().unwrap();
    let mut prev_diff = None;

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

        while h.base.time() > current_section_end {
            for skill in skills.iter_mut() {
                skill.save_current_peak();
                skill.start_new_section_from(current_section_end);
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

    let aim_rating = skills[0].difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
    let speed_rating = skills[1].difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

    aim_rating + speed_rating + (aim_rating - speed_rating).abs() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_single() {
        // let file = match File::open("E:/Games/osu!/beatmaps/1851299.osu") {
        //     Ok(file) => file,
        //     Err(why) => panic!("Could not open file: {}", why),
        // };
        let file = match File::open("C:/Users/Max/Desktop/1851299.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let stars = stars(&map, 0);

        println!("Stars: {}", stars);
    }

    #[test]
    #[ignore]
    fn test_osu() {
        let margin = 0.005;

        #[rustfmt::skip]
        let data = vec![
            (1851299, 1 << 8, 4.23514130038547),  // HT
            (1851299, 0, 5.356786475158158),      // NM
            (1851299, 1 << 6, 7.450616908751305), // DT
            (1851299, 1 << 4, 5.6834681957637665),// HR
            (1851299, 1 << 1, 4.937817303399699), // EZ

            (70090, 1 << 8, 2.2929922580201803),  // HT
            (70090, 0, 2.8322940761833983),       // NM
            (70090, 1 << 6, 3.8338563325375485),  // DT
            (70090, 1 << 4, 3.0617492228478174),  // HR
            (70090, 1 << 1, 2.698823231324141),   // EZ

            (1241370, 1 << 8, 5.662809600985943), // HT
            (1241370, 0, 7.0367002127481975),     // NM
            (1241370, 1 << 6, 11.144720506574934),// DT
            (1241370, 1 << 4, 7.641688110458715), // HR
            (1241370, 1 << 1, 6.316288616688052), // EZ
            
            // Slider fiesta
            // (1657535, 1 << 8, 4.1727975286379895),// HT
            // (1657535, 0, 5.16048239944917),       // NM
            // (1657535, 1 << 6, 7.125936779100417), // DT
            // (1657535, 1 << 4, 5.545877027713307), // HR
            // (1657535, 1 << 1, 4.66015083361088),  // EZ
        ];

        for (map_id, mods, expected_stars) in data {
            let file = match File::open(format!("./test/{}.osu", map_id)) {
                Ok(file) => file,
                Err(why) => panic!("Could not open file {}.osu: {}", map_id, why),
            };

            let map = match Beatmap::parse(file) {
                Ok(map) => map,
                Err(why) => panic!("Error while parsing map {}: {}", map_id, why),
            };

            let stars = stars(&map, mods);

            assert!(
                (stars - expected_stars).abs() < margin,
                "Stars: {} | Expected: {} => {} margin [map {} | mods {}]",
                stars,
                expected_stars,
                (stars - expected_stars).abs(),
                map_id,
                mods
            );
        }
    }
}
