use crate::{
    GameMods,
    catch::{CatchDifficultyAttributes, CatchPerformanceAttributes, CatchScoreState},
};

pub(super) struct CatchPerformanceCalculator<'mods> {
    attrs: CatchDifficultyAttributes,
    mods: &'mods GameMods,
    state: CatchScoreState,
}

impl<'a> CatchPerformanceCalculator<'a> {
    pub const fn new(
        attrs: CatchDifficultyAttributes,
        mods: &'a GameMods,
        state: CatchScoreState,
    ) -> Self {
        Self { attrs, mods, state }
    }
}

impl CatchPerformanceCalculator<'_> {
    pub fn calculate(self) -> CatchPerformanceAttributes {
        let attributes = &self.attrs;
        let stars = attributes.stars;
        let max_combo = attributes.max_combo();

        // Relying heavily on aim
        let mut pp = (5.0 * (stars / 0.0049).max(1.0) - 4.0).powf(2.0) / 100_000.0;

        let mut combo_hits = self.combo_hits();

        if combo_hits == 0 {
            combo_hits = max_combo;
        }

        // Longer maps are worth more
        let mut len_bonus = 0.95 + 0.3 * (f64::from(combo_hits) / 2500.0).min(1.0);

        if combo_hits > 2500 {
            len_bonus += (f64::from(combo_hits) / 2500.0).log10() * 0.475;
        }

        pp *= len_bonus;

        // Penalize misses exponentially
        pp *= 0.97_f64.powf(f64::from(self.state.misses));

        // Combo scaling
        if self.state.max_combo > 0 {
            pp *= (f64::from(self.state.max_combo).powf(0.8) / f64::from(max_combo).powf(0.8))
                .min(1.0);
        }

        // AR scaling
        let ar = attributes.ar;
        let mut ar_factor = 1.0;
        if ar > 9.0 {
            ar_factor += 0.1 * (ar - 9.0) + f64::from(u8::from(ar > 10.0)) * 0.1 * (ar - 10.0);
        } else if ar < 8.0 {
            ar_factor += 0.025 * (8.0 - ar);
        }
        pp *= ar_factor;

        // HD bonus
        if self.mods.hd() {
            if ar <= 10.0 {
                pp *= 1.05 + 0.075 * (10.0 - ar);
            } else if ar > 10.0 {
                pp *= 1.01 + 0.04 * (11.0 - ar.min(11.0));
            }
        }

        // FL bonus
        if self.mods.fl() {
            pp *= 1.35 * len_bonus;
        }

        // Accuracy scaling
        pp *= self.state.accuracy().powf(5.5);

        // NF penalty
        if self.mods.nf() {
            pp *= (1.0 - 0.02 * f64::from(self.state.misses)).max(0.9);
        }

        CatchPerformanceAttributes {
            difficulty: self.attrs,
            pp,
        }
    }

    const fn combo_hits(&self) -> u32 {
        self.state.fruits + self.state.droplets + self.state.misses
    }
}
