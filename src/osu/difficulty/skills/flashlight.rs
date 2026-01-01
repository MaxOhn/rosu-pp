use crate::{
    GameMods,
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::strain_decay,
    },
    osu::difficulty::{evaluators::FlashlightEvaluator, object::OsuDifficultyObject},
    util::strains_vec::StrainsVec,
};

define_skill! {
    pub struct Flashlight: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64,
        has_hidden_mod: bool,
        evaluator: FlashlightEvaluator,
    }

    pub fn new(mods: &GameMods, radius: f64, time_preempt: f64, time_fade_in: f64) -> Self {
        {
            let scaling_factor = 52.0 / radius;
        }

        Self {
            current_strain: 0.0,
            has_hidden_mod: mods.hd(),
            evaluator: FlashlightEvaluator::new(scaling_factor, time_preempt, time_fade_in),
        }
    }
}

impl Flashlight {
    const SKILL_MULTIPLIER: f64 = 0.05512;
    const STRAIN_DECAY_BASE: f64 = 0.15;

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.delta_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += self
            .evaluator
            .evaluate_diff_of(curr, objects, self.has_hidden_mod)
            * Self::SKILL_MULTIPLIER;

        self.current_strain
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "function definition needs to stay in-sync with `StrainSkill::difficulty_value`"
    )]
    fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
        current_strain_peaks.sum()
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        25.0 * f64::powf(difficulty, 2.0)
    }
}
