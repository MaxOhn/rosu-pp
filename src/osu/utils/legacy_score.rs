use crate::{
    model::hit_object::Spinner,
    osu::object::{OsuObject, OsuObjectKind},
};

pub struct NestedScorePerObject;

impl NestedScorePerObject {
    pub fn calculate(objects: &[OsuObject], passed_objects: usize) -> f64 {
        let mut inner = InnerNestedScorePerObject::default();

        for h in objects.iter().take(passed_objects) {
            inner.process_next(h);
        }

        inner.calculate()
    }
}

fn calculate_spinner_score(spinner: Spinner) -> f64 {
    const SPIN_SCORE: i32 = 100;
    const BONUS_SPIN_SCORE: i32 = 1000;

    // * The spinner object applies a lenience because gameplay mechanics differ from osu-stable.
    // * We'll redo the calculations to match osu-stable here...
    const MAXIMUM_ROTATIONS_PER_SECOND: f64 = 477.0 / 60.0;

    // * Normally, this value depends on the final overall difficulty. For simplicity, we'll only consider the worst case that maximises bonus score.
    // * As we're primarily concerned with computing the maximum theoretical final score,
    // * this will have the final effect of slightly underestimating bonus score achieved on stable when converting from score V1.
    const MINIMUM_ROTATIONS_PER_SECOND: f64 = 3.0;

    let seconds_duration = spinner.duration / 1000.0;

    // * The total amount of half spins possible for the entire spinner.
    let total_half_spins_possible = (seconds_duration * MAXIMUM_ROTATIONS_PER_SECOND * 2.0) as i32;
    // * The amount of half spins that are required to successfully complete the spinner (i.e. get a 300).
    let half_spins_required_for_completion =
        (seconds_duration * MINIMUM_ROTATIONS_PER_SECOND) as i32;
    // * To be able to receive bonus points, the spinner must be rotated another 1.5 times.
    let half_spins_required_before_bonus = half_spins_required_for_completion + 3;

    let mut score: i64 = 0;

    let full_spins = total_half_spins_possible / 2;

    // * Normal spin score
    score += i64::from(SPIN_SCORE * full_spins);

    let mut bonus_spins = (total_half_spins_possible - half_spins_required_before_bonus) / 2;

    // * Reduce amount of bonus spins because we want to represent the more average case, rather than the best one.
    bonus_spins = (bonus_spins - full_spins / 2).max(0);

    score += i64::from(BONUS_SPIN_SCORE * bonus_spins);

    score as f64
}

#[derive(Default)]
struct InnerNestedScorePerObject {
    n_sliders: usize,
    n_repeats: usize,
    amount_of_small_ticks: usize,
    spinner_score: f64,
    object_count: usize,
}

impl InnerNestedScorePerObject {
    fn process_next(&mut self, h: &OsuObject) {
        self.object_count += 1;

        match h.kind {
            OsuObjectKind::Circle => {}
            OsuObjectKind::Slider(ref slider) => {
                self.n_sliders += 1;
                self.n_repeats += slider.repeat_count();
                self.amount_of_small_ticks += slider.tick_count();
            }
            OsuObjectKind::Spinner(spinner) => {
                self.spinner_score += calculate_spinner_score(spinner);
            }
        }
    }

    fn calculate(&self) -> f64 {
        const BIG_TICK_SCORE: f64 = 30.0;
        const SMALL_TICK_SCORE: f64 = 10.0;

        // * 1 for head, 1 for tail
        let mut amount_of_big_ticks = self.n_sliders * 2;

        // * Add slider repeats
        amount_of_big_ticks += self.n_repeats;

        let slider_score = amount_of_big_ticks as f64 * BIG_TICK_SCORE
            + self.amount_of_small_ticks as f64 * SMALL_TICK_SCORE;

        (slider_score + self.spinner_score) / self.object_count as f64
    }
}

#[derive(Default)]
pub struct GradualNestedScorePerObject(InnerNestedScorePerObject);

impl GradualNestedScorePerObject {
    pub fn calculate_next(&mut self, h: &OsuObject) -> f64 {
        let Self(inner) = self;
        inner.process_next(h);

        inner.calculate()
    }
}
