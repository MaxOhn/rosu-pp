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
pub mod gradual;
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

    let mut attrs = TaikoDifficultyAttributes {
        hit_window,
        max_combo,
        is_convert: converted.is_convert,
        ..Default::default()
    };

    let color_rating = peaks.color_difficulty_value();
    let rhythm_rating = peaks.rhythm_difficulty_value();
    let stamina_rating = peaks.stamina_difficulty_value();
    let combined_rating = peaks.difficulty_value();

    DifficultyValues::eval(
        &mut attrs,
        color_rating,
        rhythm_rating,
        stamina_rating,
        combined_rating,
    );

    attrs
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

        let diff_objects = Self::create_difficulty_objects(
            converted,
            take as u32,
            clock_rate,
            &mut max_combo,
            &mut n_diff_objects,
        );

        // The first two hit objects have no difficulty object
        n_diff_objects = n_diff_objects.saturating_sub(2);

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

    pub fn eval(
        attrs: &mut TaikoDifficultyAttributes,
        color_difficulty_value: f64,
        rhythm_difficulty_value: f64,
        stamina_difficulty_value: f64,
        peaks_difficulty_value: f64,
    ) {
        let color_rating = color_difficulty_value * DIFFICULTY_MULTIPLIER;
        let rhythm_rating = rhythm_difficulty_value * DIFFICULTY_MULTIPLIER;
        let stamina_rating = stamina_difficulty_value * DIFFICULTY_MULTIPLIER;
        let combined_rating = peaks_difficulty_value * DIFFICULTY_MULTIPLIER;

        let mut star_rating = rescale(combined_rating * 1.4);

        // * TODO: This is temporary measure as we don't detect abuse of multiple-input
        // * playstyles of converts within the current system.
        if attrs.is_convert {
            star_rating *= 0.925;

            // * For maps with low colour variance and high stamina requirement,
            // * multiple inputs are more likely to be abused.
            if color_rating < 2.0 && stamina_rating > 8.0 {
                star_rating *= 0.8;
            }
        }

        attrs.stamina = stamina_rating;
        attrs.rhythm = rhythm_rating;
        attrs.color = color_rating;
        attrs.peak = combined_rating;
        attrs.stars = star_rating;
    }

    pub fn create_difficulty_objects(
        converted: &TaikoBeatmap<'_>,
        take: u32,
        clock_rate: f64,
        max_combo: &mut u32,
        n_diff_objects: &mut usize,
    ) -> TaikoDifficultyObjects {
        let mut hit_objects_iter = converted
            .map
            .hit_objects
            .iter()
            .zip(converted.map.hit_sounds.iter())
            .map(|(h, s)| TaikoObject::new(h, *s))
            .inspect(|h| {
                if *max_combo < take {
                    *n_diff_objects += 1;
                    *max_combo += u32::from(h.is_hit());
                }
            });

        let Some((mut last_last, mut last)) = hit_objects_iter.next().zip(hit_objects_iter.next())
        else {
            return TaikoDifficultyObjects::with_capacity(0);
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

        diff_objects
    }
}
