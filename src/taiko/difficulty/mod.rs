use crate::{
    any::difficulty::ModeDifficulty,
    taiko::{
        difficulty::{
            color::preprocessor::ColorDifficultyPreprocessor,
            object::{TaikoDifficultyObject, TaikoDifficultyObjects},
            skills::peaks::PeaksSkill,
        },
        object::TaikoObject,
    },
};

use self::skills::peaks::Peaks;

use super::{attributes::TaikoDifficultyAttributes, convert::TaikoBeatmap};

mod color;
mod object;
mod rhythm;
mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 1.35;

pub fn difficulty(
    difficulty: &ModeDifficulty,
    converted: &TaikoBeatmap<'_>,
) -> TaikoDifficultyAttributes {
    let clock_rate = difficulty.get_clock_rate();

    let hit_window = converted
        .attributes()
        .mods(difficulty.get_mods())
        .clock_rate(clock_rate)
        .hit_windows()
        .od;

    let DifficultyValues { peaks, max_combo } = DifficultyValues::calculate(difficulty, converted);

    let color_rating = peaks.color_difficulty_value() * DIFFICULTY_MULTIPLIER;
    let rhythm_rating = peaks.rhythm_difficulty_value() * DIFFICULTY_MULTIPLIER;
    let stamina_rating = peaks.stamina_difficulty_value() * DIFFICULTY_MULTIPLIER;
    let combined_rating = peaks.difficulty_value() * DIFFICULTY_MULTIPLIER;

    let mut star_rating = rescale(combined_rating * 1.4);

    // * TODO: This is temporary measure as we don't detect abuse of multiple-input
    // * playstyles of converts within the current system.
    if converted.is_convert {
        star_rating *= 0.925;

        // * For maps with low colour variance and high stamina requirement,
        // * multiple inputs are more likely to be abused.
        if color_rating < 2.0 && stamina_rating > 8.0 {
            star_rating *= 0.8;
        }
    }

    TaikoDifficultyAttributes {
        stamina: stamina_rating,
        rhythm: rhythm_rating,
        color: color_rating,
        peak: combined_rating,
        hit_window,
        stars: star_rating,
        max_combo,
        is_convert: converted.is_convert,
    }
}

fn rescale(stars: f64) -> f64 {
    if stars < 0.0 {
        stars
    } else {
        10.43 * (stars / 8.0 + 1.0).ln()
    }
}

pub struct DifficultyValues {
    pub peaks: Peaks,
    pub max_combo: u32,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &ModeDifficulty, converted: &TaikoBeatmap<'_>) -> Self {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut n_diff_objects = 0;
        let mut max_combo = 0;

        let mut hit_objects_iter = converted
            .map
            .hit_objects
            .iter()
            .zip(converted.map.hit_sounds.iter())
            .map(|(h, s)| TaikoObject::new(h, *s))
            .inspect(|h| {
                if max_combo < take {
                    n_diff_objects += 1;
                    max_combo += usize::from(h.is_hit());
                }
            });

        let Some((mut last_last, mut last)) = hit_objects_iter.next().zip(hit_objects_iter.next())
        else {
            return Self {
                peaks: Peaks::new(),
                max_combo: max_combo as u32,
            };
        };

        let mut diff_objects =
            TaikoDifficultyObjects::with_capacity(converted.map.hit_objects.len() - 2);

        for (i, curr) in hit_objects_iter.enumerate() {
            let diff_object = TaikoDifficultyObject::new(
                &curr,
                &last,
                &last_last,
                clock_rate,
                i,
                &mut diff_objects,
            );

            diff_objects.push(diff_object);

            last_last = last;
            last = curr;
        }

        ColorDifficultyPreprocessor::process_and_assign(&diff_objects);

        // The first two hit objects have no difficulty object
        n_diff_objects -= 2;

        let mut peaks = Peaks::new();

        {
            let mut peaks = PeaksSkill::new(&mut peaks, &diff_objects);

            for hit_object in diff_objects.iter().take(n_diff_objects) {
                peaks.process(&hit_object.borrow());
            }
        }

        Self {
            peaks,
            max_combo: max_combo as u32,
        }
    }
}
