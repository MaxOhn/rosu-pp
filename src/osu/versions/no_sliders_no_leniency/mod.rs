use super::super::{difficulty_range_ar, difficulty_range_od, DifficultyAttributes};

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
pub fn stars(map: &Beatmap, mods: impl Mods) -> DifficultyAttributes {
    let attributes = map.attributes().mods(mods);

    if map.hit_objects.len() < 2 {
        return DifficultyAttributes {
            ar: attributes.ar,
            od: attributes.od,
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

    let mut hit_objects = map.hit_objects.iter().map(|h| match &h.kind {
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

    let hit_window_od = difficulty_range_od(attributes.od) as i32 as f32 / attributes.clock_rate;
    let hit_window_ar = difficulty_range_ar(attributes.ar) as i32 as f32 / attributes.clock_rate;

    let od = (80.0 - hit_window_od) / 6.0;
    let ar = if hit_window_ar > 1200.0 {
        (1800.0 - hit_window_ar) / 120.0
    } else {
        (1200.0 - hit_window_ar) / 150.0 + 5.0
    };

    DifficultyAttributes {
        stars,
        ar,
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
    use super::super::super::PpCalculator;
    use super::stars;
    use crate::Beatmap;
    use std::fs::File;

    #[test]
    #[ignore]
    fn no_sliders_single_stars() {
        let file = match File::open("E:/Games/osu!/beatmaps/70090.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };
        // let file = match File::open("C:/Users/Max/Desktop/1241370.osu") {
        //     Ok(file) => file,
        //     Err(why) => panic!("Could not open file: {}", why),
        // };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let stars = stars(&map, 1024 + 8 + 64 + 16).stars;

        println!("Stars: {}", stars);
    }

    #[test]
    #[ignore]
    fn no_sliders_stars() {
        let margin = 0.5;

        #[rustfmt::skip]
        // TODO: More mods
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

            let stars = stars(&map, mods).stars;

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

    #[test]
    #[ignore]
    fn no_sliders_single_pp() {
        let file = match File::open("E:/Games/osu!/beatmaps/1241370.osu") {
            Ok(file) => file,
            Err(why) => panic!("Could not open file: {}", why),
        };
        // let file = match File::open("C:/Users/Max/Desktop/1851299_base.osu") {
        //     Ok(file) => file,
        //     Err(why) => panic!("Could not open file: {}", why),
        // };

        let map = match Beatmap::parse(file) {
            Ok(map) => map,
            Err(why) => panic!("Error while parsing map: {}", why),
        };

        let calculator = PpCalculator::new(&map)
            .misses(2)
            .accuracy(96.78)
            .combo(1876)
            // .n100(0)
            .mods(8 + 16);

        let result = calculator.calculate(stars);

        println!("Stars: {}", result.attributes.stars);
        println!("PP: {}", result.pp);
    }
}
