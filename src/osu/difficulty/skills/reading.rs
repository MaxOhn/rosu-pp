use crate::{
    GameMods,
    any::difficulty::skills_new::{count_top_weighted_object_difficulties, strain_decay_base},
    osu::difficulty::{evaluators::ReadingEvaluator, object::OsuDifficultyObject},
    util::difficulty::lerp,
};

define_new_skill! {
    pub struct Reading: HarmonicSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64 = 0.0,
        reduced_note_count: usize = 0,
        reduced_duration: Option<f64> = None,
        mods: GameMods,
        evaluator: ReadingEvaluator,
        overall_difficulty: f64,
    }

    pub fn new(mods: &GameMods, preempt: f64, time_preempt: f64, time_fade_in: f64, overall_difficulty: f64) -> Self {
        Self {
            current_strain: 0.0,
            reduced_note_count: 0,
            reduced_duration: None,
            mods: mods.clone(),
            evaluator: ReadingEvaluator::new(preempt, time_preempt, time_fade_in),
            overall_difficulty: overall_difficulty,
        }
    }
}

impl Reading {
    pub const SKILL_MULTIPLIER: f64 = 2.5;
    pub const REDUCED_DIFFICULTY_DURATION: i32 = 60 * 1000;
    pub const REDUCED_DIFFICULTY_BASE_LINE: f64 = 0.0;

    fn strain_decay(ms: f64) -> f64 {
        strain_decay_base(ms, 0.8)
    }

    fn object_difficulty_of<'a>(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        let decay = Self::strain_decay(curr.delta_time);

        self.current_strain *= decay;
        self.current_strain += self.calculate_adjusted_difficulty(curr, objects)
            * (1.0 - decay)
            * Self::SKILL_MULTIPLIER;

        // * This currently operates under the assumption that `ObjectDifficultyOf` is called once per object, and in order.
        // * Under that assumption, we can trust that `current.StartTime` refers to the start time of the first object in the case that `reducedDuration` is yet to be set.
        let reduced_duration = self
            .reduced_duration
            .get_or_insert(curr.start_time + f64::from(Self::REDUCED_DIFFICULTY_DURATION));

        // * This relies on the same assumption, as calling in order means that we can safely increase the note count until we reach the first object after the reduced duration.
        if curr.start_time <= *reduced_duration {
            self.reduced_note_count += 1;
        }

        self.current_strain
    }

    fn calculate_adjusted_difficulty<'a>(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        let mut difficulty = self
            .evaluator
            .evaluate_diff_of(curr, objects, self.mods.hd_full_fade());

        if self.mods.td() {
            difficulty = difficulty.powf(0.89);
        }

        if let Some(magnetised_strength) = self.mods.attraction_strength() {
            difficulty *= 1.0 - magnetised_strength;
        }

        if self.mods.rx() {
            difficulty *= 0.4;
        }

        if self.mods.ap() {
            difficulty *= 0.1;
        }

        difficulty *= 0.825 + self.overall_difficulty.max(0.0).powf(2.2) / 1125.0;

        difficulty
    }

    fn get_transformed_difficulties(&self, mut difficulties: Vec<f64>) -> Vec<f64> {
        difficulties.retain(|v| *v > 0.0);

        let count = std::cmp::min(difficulties.len(), self.reduced_note_count);
        for (i, difficulty) in difficulties.iter_mut().take(count).enumerate() {
            let scale = lerp(
                1.0,
                10.0,
                (i as f64 / self.reduced_note_count as f64).clamp(0.0, 1.0),
            )
            .log10();

            *difficulty *= lerp(Self::REDUCED_DIFFICULTY_BASE_LINE, 1.0, scale);
        }

        difficulties
    }

    pub fn count_top_weighted_object_difficulties(
        &self,
        difficulty_value: f64,
        object_weight_sum: f64,
    ) -> f64 {
        count_top_weighted_object_difficulties(
            difficulty_value,
            &self.skill_object_difficulties,
            object_weight_sum,
            1.15,
            5.0,
            Some(1.1),
        )
    }
}
