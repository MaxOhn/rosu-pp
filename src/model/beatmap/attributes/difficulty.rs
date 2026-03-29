use rosu_map::section::general::GameMode;
use rosu_mods::GameMod;

use crate::{GameMods, model::beatmap::attributes::attribute::BeatmapAttribute};

#[derive(Clone, Debug, PartialEq)]
pub struct BeatmapDifficulty {
    pub ar: BeatmapAttribute,
    pub cs: BeatmapAttribute,
    pub hp: BeatmapAttribute,
    pub od: BeatmapAttribute,
}

impl BeatmapDifficulty {
    pub const DEFAULT: Self = Self {
        ar: BeatmapAttribute::None,
        cs: BeatmapAttribute::None,
        hp: BeatmapAttribute::None,
        od: BeatmapAttribute::None,
    };

    pub fn apply_mods(&mut self, mods: &GameMods, mode: GameMode) {
        // First we *set* values
        if let GameMods::Lazer(mods) = mods {
            macro_rules! set_if_some {
                ( $attr:ident = $opt:expr ) => {
                    if let Some(value) = $opt {
                        self.$attr.try_set(value as f32);
                    }
                };
            }

            for m in mods.iter() {
                let (hp, od) = match m {
                    GameMod::DifficultyAdjustCatch(da) => {
                        set_if_some!(ar = da.approach_rate);
                        set_if_some!(cs = da.circle_size);

                        (da.drain_rate, da.overall_difficulty)
                    }
                    GameMod::DifficultyAdjustMania(da) => (da.drain_rate, da.overall_difficulty),
                    GameMod::DifficultyAdjustOsu(da) => {
                        set_if_some!(ar = da.approach_rate);
                        set_if_some!(cs = da.circle_size);

                        (da.drain_rate, da.overall_difficulty)
                    }
                    GameMod::DifficultyAdjustTaiko(da) => {
                        // Ignoring slider multiplier

                        (da.drain_rate, da.overall_difficulty)
                    }
                    _ => continue,
                };

                set_if_some!(hp = hp);
                set_if_some!(od = od);
            }
        }

        // Then we *adjust* values
        if mods.ez() {
            const ADJUST_RATIO: f32 = 0.5;

            self.ar.try_mutate(|ar| *ar *= ADJUST_RATIO);
            self.cs.try_mutate(|cs| *cs *= ADJUST_RATIO);
            self.hp.try_mutate(|hp| *hp *= ADJUST_RATIO);

            match mode {
                GameMode::Osu => self.od.try_mutate(|od| *od *= ADJUST_RATIO),
                // Ignoring slider multiplier
                GameMode::Taiko => self.od.try_mutate(|od| *od *= ADJUST_RATIO),
                GameMode::Catch => self.od.try_mutate(|od| *od *= ADJUST_RATIO),
                GameMode::Mania => {}
            }
        } else if mods.hr() {
            const ADJUST_RATIO: f32 = 1.4;

            self.hp
                .try_mutate(|hp| *hp = f32::min(*hp * ADJUST_RATIO, 10.0));

            match mode {
                GameMode::Osu => {
                    self.od
                        .try_mutate(|od| *od = f32::min(*od * ADJUST_RATIO, 10.0));
                    // * CS uses a custom 1.3 ratio.
                    self.cs.try_mutate(|cs| *cs = f32::min(*cs * 1.3, 10.0));
                    self.ar
                        .try_mutate(|ar| *ar = f32::min(*ar * ADJUST_RATIO, 10.0));
                }
                // Ignoring slider multiplier
                GameMode::Taiko => self
                    .od
                    .try_mutate(|od| *od = f32::min(*od * ADJUST_RATIO, 10.0)),
                GameMode::Catch => {
                    self.od
                        .try_mutate(|od| *od = f32::min(*od * ADJUST_RATIO, 10.0));
                    // * CS uses a custom 1.3 ratio.
                    self.cs.try_mutate(|cs| *cs = f32::min(*cs * 1.3, 10.0));
                    self.ar
                        .try_mutate(|ar| *ar = f32::min(*ar * ADJUST_RATIO, 10.0));
                }
                GameMode::Mania => {}
            }
        }
    }
}
