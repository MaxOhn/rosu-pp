use crate::{
    GameMods,
    mania::{ManiaDifficultyAttributes, ManiaPerformanceAttributes, ManiaScoreState},
};

pub(super) struct ManiaPerformanceCalculator<'mods> {
    attrs: ManiaDifficultyAttributes,
    mods: &'mods GameMods,
    state: ManiaScoreState,
}

impl<'a> ManiaPerformanceCalculator<'a> {
    pub const fn new(
        attrs: ManiaDifficultyAttributes,
        mods: &'a GameMods,
        state: ManiaScoreState,
    ) -> Self {
        Self { attrs, mods, state }
    }
}

impl ManiaPerformanceCalculator<'_> {
    pub fn calculate(self) -> ManiaPerformanceAttributes {
        let mut multiplier = 1.0;

        if self.mods.nf() {
            multiplier *= 0.75;
        }

        if self.mods.ez() {
            multiplier *= 0.5;
        }

        let difficulty_value = self.compute_difficulty_value();
        let pp = difficulty_value * multiplier;

        ManiaPerformanceAttributes {
            difficulty: self.attrs,
            pp,
            pp_difficulty: difficulty_value,
        }
    }

    fn compute_difficulty_value(&self) -> f64 {
        // * Star rating to pp curve
        8.0 * f64::powf(f64::max(self.attrs.stars - 0.15, 0.05), 2.2)
             // * From 80% accuracy, 1/20th of total pp is awarded per additional 1% accuracy
             * f64::max(0.0, 5.0 * self.calculate_custom_accuracy() - 4.0)
             // * Length bonus, capped at 1500 notes
             * (1.0 + 0.1 * f64::min(1.0, self.total_hits() / 1500.0))
    }

    const fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }

    fn calculate_custom_accuracy(&self) -> f64 {
        let ManiaScoreState {
            n320,
            n300,
            n200,
            n100,
            n50,
            misses: _,
        } = &self.state;

        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        custom_accuracy(*n320, *n300, *n200, *n100, *n50, total_hits)
    }
}

pub(super) fn custom_accuracy(
    n320: u32,
    n300: u32,
    n200: u32,
    n100: u32,
    n50: u32,
    total_hits: u32,
) -> f64 {
    let numerator = n320 * 32 + n300 * 30 + n200 * 20 + n100 * 10 + n50 * 5;
    let denominator = total_hits * 32;

    f64::from(numerator) / f64::from(denominator)
}
