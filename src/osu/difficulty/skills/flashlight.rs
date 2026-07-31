use crate::{
    GameMods,
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills_new::{strain_decay_base, strain_skill::NewStrainSkill},
    },
    osu::difficulty::{evaluators::FlashlightEvaluator, object::OsuDifficultyObject},
    util::difficulty::reverse_lerp,
};

define_new_skill! {
    pub struct Flashlight: NewStrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64,
        total_objects: i32,
        overall_difficulty: f64,
        mods: GameMods,
        evaluator: FlashlightEvaluator,
    }

    pub fn new(mods: &GameMods, total_objects: i32, overall_difficulty: f64, radius: f64, time_preempt: f64, time_fade_in: f64) -> Self {
        let scaling_factor = 52.0 / radius;

        Self {
            current_strain: 0.0,
            total_objects: total_objects,
            mods: mods.clone(),
            overall_difficulty: overall_difficulty,
            evaluator: FlashlightEvaluator::new(scaling_factor, time_preempt, time_fade_in),
        }
    }
}

impl Flashlight {
    const SKILL_MULTIPLIER: f64 = 0.058;

    fn strain_decay(ms: f64) -> f64 {
        strain_decay_base(ms, 0.15)
    }

    fn calculate_initial_strain<'a>(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'a>,
        objects: &[OsuDifficultyObject<'a>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * Self::strain_decay(time - prev_start_time)
    }

    fn strain_value_at<'a>(
        &mut self,
        curr: &OsuDifficultyObject<'a>,
        objects: &[OsuDifficultyObject<'a>],
    ) -> f64 {
        if !self.mods.fl() {
            return 0.0;
        }

        self.current_strain *= Self::strain_decay(curr.delta_time);
        self.current_strain +=
            self.calculate_adjusted_difficulty(curr, objects) * Self::SKILL_MULTIPLIER;

        self.current_strain
    }

    fn calculate_adjusted_difficulty(
        &self,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut difficulty = self.evaluator.evaluate_diff_of(curr, objects, &self.mods);

        if self.mods.td() {
            difficulty = difficulty.powf(0.9);
        }

        if let Some(attraction_strength) = self.mods.attraction_strength() {
            difficulty *= 1.0 - attraction_strength;
        }

        if let Some(start_scale) = self.mods.deflate_start_scale() {
            difficulty *= reverse_lerp(start_scale, 11.0, 1.0).clamp(0.1, 1.0);
        }

        if self.mods.rx() {
            difficulty *= 0.7;
        }

        if self.mods.ap() {
            difficulty *= 0.4;
        }

        difficulty *= 0.985 + self.overall_difficulty.max(0.0).powf(2.0) / 4000.0;

        difficulty
    }

    // NOTE:
    // Flashlight is (currently) the only skill needing to override `StrainSkill.difficulty_value(current_strain_peaks) -> f64`
    // and requires `self.total_objects`. Since the static method isn't ever used as far as I can tell, it can remain default for now.
    // As long as `into_difficulty_value` and `cloned_difficulty_value` are correct it should not affect elsewhere.

    #[expect(dead_code, reason = "overwrites macro impl")]
    fn into_difficulty_value(self) -> f64 {
        flashlight_difficulty_value(
            Self::get_current_strain_peaks(
                self.skill_strain_peaks,
                self.skill_current_section_peak,
            ),
            self.total_objects,
        )
    }

    pub fn cloned_difficulty_value(&self) -> f64 {
        flashlight_difficulty_value(
            Self::get_current_strain_peaks(
                self.skill_strain_peaks.clone(),
                self.skill_current_section_peak,
            ),
            self.total_objects,
        )
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        25.0 * difficulty.powf(2.0)
    }
}

fn flashlight_difficulty_value(current_strain_peaks: Vec<f64>, total_objects: i32) -> f64 {
    let sum: f64 = current_strain_peaks.into_iter().sum();

    sum * 0.7
        + 0.1 * (f64::from(total_objects) / 200.0).min(1.0)
        + (if total_objects > 200 {
            0.2 * f64::from(std::cmp::min(total_objects - 200, 1) / 200)
        } else {
            0.0
        })
}
